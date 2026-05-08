//! Read-only swarm coordination diagnostics.

use crate::cli::{
    CoordinationOwnerKindArg, CoordinationStatusArgs, OutputFormat,
    resolve_output_format_basic_with_outer_mode,
};
use crate::config;
use crate::coordination::{
    ClaimAssessmentInput, ClaimOwnerKind, CoordinationClaimRow, CoordinationComment,
    CoordinationIssueRow, CoordinationStatusOutput, CoordinationWorkspaceCounts,
    ReservationEvidence, assess_claim,
};
use crate::error::{BeadsError, Result};
use crate::format::{sanitize_terminal_inline, truncate_title};
use crate::model::{Comment, Issue, Status};
use crate::output::{OutputContext, OutputMode};
use crate::storage::{ListFilters, ReadyFilters, ReadySortPolicy, SqliteStorage};
use chrono::{DateTime, Utc};
use std::path::Path;

/// Execute `br coordination status`.
///
/// # Errors
///
/// Returns an error if the workspace cannot be opened or read.
pub fn execute_status(
    args: &CoordinationStatusArgs,
    cli: &config::CliOverrides,
    outer_ctx: &OutputContext,
) -> Result<()> {
    let beads_dir = config::discover_beads_dir_with_cli(cli)?;
    execute_status_inner(args, cli, outer_ctx, &beads_dir, None, None)
}

/// Execute `br coordination status` using the caller's pre-opened storage.
///
/// # Errors
///
/// Returns an error if coordination rows cannot be loaded.
pub fn execute_status_with_storage_ctx(
    args: &CoordinationStatusArgs,
    cli: &config::CliOverrides,
    outer_ctx: &OutputContext,
    beads_dir: &Path,
    storage_ctx: &config::OpenStorageResult,
) -> Result<()> {
    execute_status_inner(args, cli, outer_ctx, beads_dir, None, Some(storage_ctx))
}

fn execute_status_inner(
    args: &CoordinationStatusArgs,
    cli: &config::CliOverrides,
    outer_ctx: &OutputContext,
    beads_dir: &Path,
    preloaded_storage: Option<&SqliteStorage>,
    preloaded_storage_ctx: Option<&config::OpenStorageResult>,
) -> Result<()> {
    let owned_storage_ctx = if preloaded_storage.is_some() || preloaded_storage_ctx.is_some() {
        None
    } else {
        Some(config::open_storage_with_cli(beads_dir, cli)?)
    };
    let storage = preloaded_storage
        .or_else(|| preloaded_storage_ctx.map(|ctx| &ctx.storage))
        .or_else(|| owned_storage_ctx.as_ref().map(|ctx| &ctx.storage))
        .ok_or_else(|| BeadsError::internal("coordination status missing open storage handle"))?;

    let output_format = resolve_output_format_basic_with_outer_mode(
        args.format,
        outer_ctx.inherited_output_mode(),
        args.robot,
    );
    let quiet = cli.quiet.unwrap_or(false);
    let ctx = OutputContext::from_output_format(output_format, quiet, true);
    if matches!(ctx.mode(), OutputMode::Quiet) {
        return Ok(());
    }

    let output = build_coordination_status_output(
        storage,
        owner_kind_from_arg(args.owner_kind),
        args.comments,
        Utc::now(),
    )?;

    match output_format {
        OutputFormat::Json => ctx.json_pretty(&output),
        OutputFormat::Toon => ctx.toon_with_stats(&output, args.stats),
        OutputFormat::Text | OutputFormat::Csv => print_text_output(&output),
    }

    Ok(())
}

fn build_coordination_status_output(
    storage: &SqliteStorage,
    owner_kind: ClaimOwnerKind,
    comment_limit: usize,
    generated_at: DateTime<Utc>,
) -> Result<CoordinationStatusOutput> {
    let filters = ListFilters {
        statuses: Some(vec![Status::InProgress]),
        include_deferred: true,
        sort: Some("updated_at".to_string()),
        ..ListFilters::default()
    };
    let issues = storage.list_issues(&filters)?;
    let issue_ids = issues
        .iter()
        .map(|issue| issue.id.clone())
        .collect::<Vec<_>>();
    let labels_by_issue = storage.get_labels_for_issues(&issue_ids)?;
    let (dependency_counts, dependent_counts) =
        storage.count_relation_counts_for_issues(&issue_ids)?;
    let comments_by_issue = storage.get_comments_for_issues(&issue_ids)?;

    let claims: Vec<CoordinationClaimRow> = issues
        .into_iter()
        .map(|issue| {
            let issue_id = issue.id.clone();
            build_claim_row(
                issue,
                ClaimRowContext {
                    owner_kind,
                    generated_at,
                    labels: labels_by_issue.get(&issue_id).cloned().unwrap_or_default(),
                    dependency_count: dependency_counts.get(&issue_id).copied().unwrap_or(0),
                    dependent_count: dependent_counts.get(&issue_id).copied().unwrap_or(0),
                    comments: comments_by_issue
                        .get(&issue_id)
                        .map(Vec::as_slice)
                        .unwrap_or(&[]),
                    comment_limit,
                },
            )
        })
        .collect();
    let workspace = workspace_counts(storage, claims.len())?;

    Ok(CoordinationStatusOutput::new(
        generated_at,
        workspace,
        claims,
    ))
}

struct ClaimRowContext<'a> {
    owner_kind: ClaimOwnerKind,
    generated_at: DateTime<Utc>,
    labels: Vec<String>,
    dependency_count: usize,
    dependent_count: usize,
    comments: &'a [Comment],
    comment_limit: usize,
}

fn build_claim_row(issue: Issue, context: ClaimRowContext<'_>) -> CoordinationClaimRow {
    let latest_comments = latest_comments(context.comments, context.comment_limit);
    let assessment = assess_claim(ClaimAssessmentInput {
        assignee: issue.assignee.clone(),
        updated_at: issue.updated_at,
        now: context.generated_at,
        owner_kind: context.owner_kind,
        reservation: ReservationEvidence::NoSnapshot,
    });
    let issue = CoordinationIssueRow {
        id: issue.id,
        title: issue.title,
        status: issue.status,
        priority: issue.priority,
        issue_type: issue.issue_type,
        labels: context.labels,
        dependency_count: context.dependency_count,
        dependent_count: context.dependent_count,
        latest_comments,
    };

    CoordinationClaimRow { issue, assessment }
}

fn latest_comments(comments: &[Comment], limit: usize) -> Vec<CoordinationComment> {
    comments
        .iter()
        .rev()
        .take(limit)
        .map(CoordinationComment::from)
        .collect()
}

fn workspace_counts(
    storage: &SqliteStorage,
    in_progress_count: usize,
) -> Result<CoordinationWorkspaceCounts> {
    Ok(CoordinationWorkspaceCounts {
        open: status_count(storage, &Status::Open)?,
        ready: storage
            .get_ready_issues_for_command_output(
                &ReadyFilters::default(),
                ReadySortPolicy::Priority,
            )?
            .len(),
        blocked: storage.get_blocked_ids()?.len(),
        in_progress: in_progress_count,
        deferred: status_count(storage, &Status::Deferred)?,
        closed: status_count(storage, &Status::Closed)?,
    })
}

fn status_count(storage: &SqliteStorage, status: &Status) -> Result<usize> {
    storage.count_issues_with_filters(&ListFilters {
        statuses: Some(vec![status.clone()]),
        include_closed: matches!(status, &Status::Closed | &Status::Tombstone),
        include_deferred: matches!(status, &Status::Deferred),
        ..ListFilters::default()
    })
}

const fn owner_kind_from_arg(arg: CoordinationOwnerKindArg) -> ClaimOwnerKind {
    match arg {
        CoordinationOwnerKindArg::SwarmAgent => ClaimOwnerKind::SwarmAgent,
        CoordinationOwnerKindArg::Human => ClaimOwnerKind::Human,
        CoordinationOwnerKindArg::Unknown => ClaimOwnerKind::Unknown,
    }
}

fn print_text_output(output: &CoordinationStatusOutput) {
    println!(
        "Coordination status ({} in-progress claim{}):",
        output.summary.total_claims,
        if output.summary.total_claims == 1 {
            ""
        } else {
            "s"
        }
    );
    println!(
        "Workspace: open {} | ready {} | blocked {} | deferred {} | closed {}",
        output.summary.workspace.open,
        output.summary.workspace.ready,
        output.summary.workspace.blocked,
        output.summary.workspace.deferred,
        output.summary.workspace.closed
    );

    if output.claims.is_empty() {
        println!("No in-progress claims.");
        return;
    }

    for claim in &output.claims {
        let issue_id = sanitize_terminal_inline(&claim.issue.id);
        println!(
            "- {} [{} {}] {}",
            issue_id,
            claim.issue.priority,
            claim.issue.issue_type,
            truncate_title(&claim.issue.title, 72)
        );
        println!(
            "  assignee: {} | age: {}m | classification: {} | next_action: {}",
            claim
                .assessment
                .assignee
                .as_deref()
                .map(sanitize_terminal_inline)
                .unwrap_or_else(|| "(unassigned)".into()),
            claim.assessment.updated_age_minutes,
            claim.assessment.classification.as_str(),
            claim.assessment.recommended_action.as_str()
        );
        println!(
            "  deps: {} | dependents: {} | labels: {}",
            claim.issue.dependency_count,
            claim.issue.dependent_count,
            text_labels(&claim.issue.labels)
        );
        if let Some(comment) = claim.issue.latest_comments.first() {
            println!(
                "  latest_comment: {}: {}",
                sanitize_terminal_inline(&comment.author),
                truncate_title(&comment.text, 96)
            );
        }
    }
}

fn text_labels(labels: &[String]) -> String {
    if labels.is_empty() {
        return "(none)".to_string();
    }

    labels
        .iter()
        .map(|label| sanitize_terminal_inline(label).into_owned())
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::{ClaimRowContext, build_claim_row, latest_comments, owner_kind_from_arg};
    use crate::cli::CoordinationOwnerKindArg;
    use crate::coordination::{ClaimClassification, ClaimOwnerKind, RecommendedAction};
    use crate::model::{Comment, Issue, IssueType, Priority, Status};
    use chrono::{Duration, TimeZone, Utc};

    fn now() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 8, 9, 0, 0)
            .single()
            .expect("valid timestamp")
    }

    fn issue(updated_at: chrono::DateTime<Utc>, assignee: Option<&str>) -> Issue {
        Issue {
            id: "bd-claim".to_string(),
            title: "Claimed issue".to_string(),
            status: Status::InProgress,
            priority: Priority(1),
            issue_type: IssueType::Task,
            assignee: assignee.map(str::to_string),
            updated_at,
            ..Issue::default()
        }
    }

    #[test]
    fn build_claim_row_uses_shared_no_snapshot_classification() {
        let row = build_claim_row(
            issue(now() - Duration::hours(2), Some("TopazFox")),
            ClaimRowContext {
                owner_kind: ClaimOwnerKind::SwarmAgent,
                generated_at: now(),
                labels: vec!["coordination".to_string()],
                dependency_count: 2,
                dependent_count: 3,
                comments: &[],
                comment_limit: 2,
            },
        );

        assert_eq!(row.issue.id, "bd-claim");
        assert_eq!(row.issue.labels, ["coordination"]);
        assert_eq!(row.issue.dependency_count, 2);
        assert_eq!(row.issue.dependent_count, 3);
        assert_eq!(
            row.assessment.classification,
            ClaimClassification::NoMailSnapshot
        );
        assert_eq!(
            row.assessment.recommended_action,
            RecommendedAction::InspectMail
        );
    }

    #[test]
    fn latest_comments_are_newest_first_and_bounded() {
        let comments = vec![
            Comment {
                id: 1,
                issue_id: "bd-claim".to_string(),
                author: "a".to_string(),
                body: "old".to_string(),
                created_at: now() - Duration::hours(2),
            },
            Comment {
                id: 2,
                issue_id: "bd-claim".to_string(),
                author: "b".to_string(),
                body: "new".to_string(),
                created_at: now(),
            },
        ];

        let latest = latest_comments(&comments, 1);

        assert_eq!(latest.len(), 1);
        assert_eq!(latest[0].text, "new");
    }

    #[test]
    fn owner_kind_arg_maps_to_coordination_policy() {
        assert_eq!(
            owner_kind_from_arg(CoordinationOwnerKindArg::Human),
            ClaimOwnerKind::Human
        );
        assert_eq!(
            owner_kind_from_arg(CoordinationOwnerKindArg::Unknown),
            ClaimOwnerKind::Unknown
        );
    }
}
