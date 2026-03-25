//! MCP (Model Context Protocol) server for beads_rust.
//!
//! Exposes the issue tracker as an MCP server so that AI agents can
//! query, create, and manage issues through the standard MCP protocol
//! instead of shelling out to the `br` CLI.
//!
//! This module is feature-gated behind `mcp` and is **not** included
//! in the default feature set.

mod prompts;
mod resources;
mod tools;

use crate::BIN_NAME;
use std::path::PathBuf;

use fastmcp_rust::McpError;

use crate::config;

/// Map any `Display` error into a flat `McpError::tool_error`.
///
/// Used by resources and prompts for non-structured error mapping.
/// Tools use the richer `beads_to_mcp` in `tools.rs` instead.
pub(super) fn to_mcp(err: impl std::fmt::Display) -> McpError {
    McpError::tool_error(err.to_string())
}

/// Shared configuration available to every MCP handler.
///
/// Storage is intentionally **not** held open: `fsqlite::Connection` uses
/// `Rc` internally and therefore cannot satisfy `Send + Sync`. Each handler
/// call reopens storage via [`BeadsState::open_storage_ctx`].
pub struct BeadsState {
    pub beads_dir: PathBuf,
    pub actor: String,
    pub overrides: config::CliOverrides,
}

impl BeadsState {
    /// Open a fresh storage context using the same config resolution and
    /// no-db/recovery policy as the CLI.
    ///
    /// # Errors
    ///
    /// Returns an error if storage cannot be opened.
    pub fn open_storage_ctx(&self) -> crate::Result<config::OpenStorageResult> {
        config::open_storage_with_cli(&self.beads_dir, &self.overrides)
    }

    /// Resolve the effective issue prefix for the current request.
    ///
    /// # Errors
    ///
    /// Returns an error if merged config loading fails.
    pub fn issue_prefix(&self, storage_ctx: &config::OpenStorageResult) -> crate::Result<String> {
        resolved_mcp_issue_prefix(storage_ctx, &self.overrides)
    }
}

/// CLI arguments for `br serve`.
#[derive(clap::Args, Debug, Clone)]
pub struct ServeArgs {
    /// Actor name for mutations (defaults to "mcp")
    #[arg(long, default_value = "mcp")]
    pub actor: String,
}

fn resolved_mcp_issue_prefix(
    storage_ctx: &config::OpenStorageResult,
    overrides: &config::CliOverrides,
) -> crate::Result<String> {
    let merged_layer = storage_ctx.load_config(overrides)?;
    Ok(config::id_config_from_layer(&merged_layer).prefix)
}

pub(super) fn persist_mcp_mutation(
    storage_ctx: &mut config::OpenStorageResult,
) -> crate::Result<()> {
    storage_ctx.flush_no_db_if_dirty()?;
    storage_ctx.auto_flush_if_enabled()?;
    Ok(())
}

/// Entry point: build and run the MCP server on stdio.
///
/// # Errors
///
/// Returns an error if the beads workspace is not initialised or storage
/// cannot be opened.
pub fn run_serve(args: &ServeArgs, overrides: &config::CliOverrides) -> crate::Result<()> {
    let beads_dir = config::discover_beads_dir_with_cli(overrides)?;
    let res = config::open_storage_with_cli(&beads_dir, overrides)?;
    let _ = resolved_mcp_issue_prefix(&res, overrides)?;

    // Eagerly drop the bootstrap connection; handlers will open their own.
    drop(res.storage);

    let state = std::sync::Arc::new(BeadsState {
        beads_dir,
        actor: args.actor.clone(),
        overrides: overrides.clone(),
    });

    let server = fastmcp_rust::Server::new(BIN_NAME, env!("CARGO_PKG_VERSION"))
        .instructions(
            "beads_rust (br) issue tracker MCP server.\n\n\
             Use tools to query, create, and manage issues. All mutations are \
             recorded with full audit trails.\n\n\
             Getting started:\n\
             1. Call project_overview to understand the project state\n\
             2. Read beads://schema for valid field values and bead anatomy guidance\n\
             3. Read beads://labels to discover existing labels\n\
             4. Use list_issues to find specific issues\n\n\
             Discovery resources: beads://project/info, beads://schema, \
             beads://labels, beads://issues/ready, beads://issues/blocked, \
             beads://issues/deferred, beads://issues/bottlenecks, \
             beads://graph/health, beads://events/recent\n\n\
             Guided workflows:\n\
             - 'triage' — backlog triage (blocked, unassigned, deferred)\n\
             - 'status_report' — project status report generation\n\
             - 'plan_next_work' — graph-aware work planning (bottlenecks, quick wins)\n\
             - 'polish_backlog' — review issue quality and dependency health",
        )
        // Tools (7 — at the ≤7 cluster ceiling)
        .tool(tools::ListIssuesTool::new(state.clone()))
        .tool(tools::ShowIssueTool::new(state.clone()))
        .tool(tools::CreateIssueTool::new(state.clone()))
        .tool(tools::UpdateIssueTool::new(state.clone()))
        .tool(tools::CloseIssueTool::new(state.clone()))
        .tool(tools::ManageDependenciesTool::new(state.clone()))
        .tool(tools::ProjectOverviewTool::new(state.clone()))
        // Resources (11)
        .resource(resources::ProjectInfoResource::new(state.clone()))
        .resource(resources::IssueResource::new(state.clone()))
        .resource(resources::SchemaResource)
        .resource(resources::LabelsResource::new(state.clone()))
        .resource(resources::ReadyIssuesResource::new(state.clone()))
        .resource(resources::BlockedIssuesResource::new(state.clone()))
        .resource(resources::InProgressResource::new(state.clone()))
        .resource(resources::EventsResource::new(state.clone()))
        .resource(resources::DeferredIssuesResource::new(state.clone()))
        .resource(resources::GraphHealthResource::new(state.clone()))
        .resource(resources::BottlenecksResource::new(state.clone()))
        // Prompts (4)
        .prompt(prompts::TriagePrompt::new(state.clone()))
        .prompt(prompts::StatusReportPrompt::new(state.clone()))
        .prompt(prompts::PlanNextWorkPrompt::new(state.clone()))
        .prompt(prompts::PolishBacklogPrompt::new(state))
        .build();

    server.run_stdio();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Issue, IssueType, Priority, Status};
    use crate::storage::SqliteStorage;
    use crate::sync::{ExportConfig, export_to_jsonl_with_policy, finalize_export};
    use chrono::Utc;
    use std::fs;
    use tempfile::TempDir;

    fn sample_issue(id: &str, title: &str) -> Issue {
        let now = Utc::now();
        Issue {
            id: id.to_string(),
            content_hash: None,
            title: title.to_string(),
            description: None,
            design: None,
            acceptance_criteria: None,
            notes: None,
            status: Status::Open,
            priority: Priority::MEDIUM,
            issue_type: IssueType::Task,
            assignee: None,
            owner: None,
            estimated_minutes: None,
            created_at: now,
            created_by: Some("tester".to_string()),
            updated_at: now,
            closed_at: None,
            close_reason: None,
            closed_by_session: None,
            due_at: None,
            defer_until: None,
            external_ref: None,
            source_system: None,
            source_repo: None,
            deleted_at: None,
            deleted_by: None,
            delete_reason: None,
            original_type: None,
            compaction_level: None,
            compacted_at: None,
            compacted_at_commit: None,
            original_size: None,
            sender: None,
            ephemeral: false,
            pinned: false,
            is_template: false,
            labels: vec![],
            dependencies: vec![],
            comments: vec![],
        }
    }

    #[test]
    fn resolved_mcp_issue_prefix_uses_merged_config_precedence() {
        let temp = TempDir::new().expect("tempdir");
        let beads_dir = temp.path().join(".beads");
        fs::create_dir_all(&beads_dir).expect("create beads dir");
        fs::write(
            beads_dir.join("config.yaml"),
            "issue_prefix: project-prefix\n",
        )
        .expect("write project config");

        let cli = config::CliOverrides::default();
        let mut storage_ctx = config::open_storage_with_cli(&beads_dir, &cli).expect("storage");
        storage_ctx
            .storage
            .set_config("issue_prefix", "db-prefix")
            .expect("set db prefix");

        let prefix = resolved_mcp_issue_prefix(&storage_ctx, &cli).expect("resolved prefix");
        assert_eq!(prefix, "project-prefix");
    }

    #[test]
    fn beads_state_open_storage_ctx_honors_no_db_jsonl_snapshot() {
        let temp = TempDir::new().expect("tempdir");
        let beads_dir = temp.path().join(".beads");
        fs::create_dir_all(&beads_dir).expect("create beads dir");
        fs::write(beads_dir.join("config.yaml"), "no-db: true\n").expect("write config");

        let mut storage = SqliteStorage::open_memory().expect("memory storage");
        let issue = sample_issue("proj-read01", "Read from JSONL snapshot");
        storage
            .create_issue(&issue, "tester")
            .expect("create issue");

        let jsonl_path = beads_dir.join("issues.jsonl");
        let (export_result, _) =
            export_to_jsonl_with_policy(&storage, &jsonl_path, &ExportConfig::default())
                .expect("export jsonl");
        finalize_export(
            &mut storage,
            &export_result,
            Some(&export_result.issue_hashes),
            &jsonl_path,
        )
        .expect("finalize export");

        let state = BeadsState {
            beads_dir: beads_dir.clone(),
            actor: "mcp".to_string(),
            overrides: config::CliOverrides::default(),
        };

        let storage_ctx = state.open_storage_ctx().expect("open storage ctx");
        assert!(storage_ctx.no_db);
        assert_eq!(storage_ctx.storage.count_all_issues().expect("count"), 1);
        assert_eq!(state.issue_prefix(&storage_ctx).expect("prefix"), "proj");
    }

    #[test]
    fn persist_mcp_mutation_flushes_no_db_changes() {
        let temp = TempDir::new().expect("tempdir");
        let beads_dir = temp.path().join(".beads");
        fs::create_dir_all(&beads_dir).expect("create beads dir");
        fs::write(beads_dir.join("config.yaml"), "no-db: true\n").expect("write config");
        fs::write(beads_dir.join("issues.jsonl"), "").expect("write empty jsonl");

        let state = BeadsState {
            beads_dir: beads_dir.clone(),
            actor: "mcp".to_string(),
            overrides: config::CliOverrides::default(),
        };

        let mut storage_ctx = state.open_storage_ctx().expect("open storage ctx");
        assert!(storage_ctx.no_db);

        let mut issue = sample_issue("proj-write01", "Persist no-db mutation");
        issue.content_hash = Some(issue.compute_content_hash());
        storage_ctx
            .storage
            .create_issue(&issue, "tester")
            .expect("create issue");

        persist_mcp_mutation(&mut storage_ctx).expect("persist mutation");

        let exported =
            fs::read_to_string(beads_dir.join("issues.jsonl")).expect("read exported jsonl");
        assert!(
            exported.contains("\"id\":\"proj-write01\""),
            "expected persisted issue in JSONL, got: {exported}"
        );
    }
}
