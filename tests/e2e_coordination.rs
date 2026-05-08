mod common;

use common::cli::{BrWorkspace, extract_json_payload, run_br};
use serde_json::{Value, json};
use std::fs;
use toon_rust::try_decode as parse_toon;

fn seed_coordination_workspace(workspace: &BrWorkspace) {
    let init = run_br(workspace, ["init"], "init");
    assert!(init.status.success(), "init failed: {}", init.stderr);

    let fresh = json!({
        "id": "bd-fresh",
        "title": "Fresh in-progress claim",
        "status": "in_progress",
        "priority": 1,
        "issue_type": "task",
        "assignee": "TopazFox",
        "created_at": "2099-01-01T00:00:00Z",
        "created_by": "tester",
        "updated_at": "2099-01-01T00:00:00Z",
        "labels": ["coordination"],
        "ephemeral": false,
        "pinned": false,
        "is_template": false,
        "dependencies": [],
        "comments": [
            {
                "id": 1,
                "issue_id": "bd-fresh",
                "author": "TopazFox",
                "text": "fresh claim note",
                "created_at": "2099-01-01T00:00:00Z"
            }
        ]
    });
    let stale = json!({
        "id": "bd-stale",
        "title": "Stale \u{1b}[31m in-progress claim",
        "status": "in_progress",
        "priority": 0,
        "issue_type": "bug",
        "assignee": "AmberLion",
        "created_at": "2020-01-01T00:00:00Z",
        "created_by": "tester",
        "updated_at": "2020-01-01T00:00:00Z",
        "labels": ["coordination", "stale"],
        "ephemeral": false,
        "pinned": false,
        "is_template": false,
        "dependencies": [],
        "comments": [
            {
                "id": 2,
                "issue_id": "bd-stale",
                "author": "AmberLion",
                "text": "old \u{1b}[31m stale claim note",
                "created_at": "2020-01-01T00:00:00Z"
            }
        ]
    });
    let body = format!("{fresh}\n{stale}\n");
    fs::write(workspace.root.join(".beads/issues.jsonl"), body).expect("write seed JSONL");

    let import = run_br(
        workspace,
        ["sync", "--import-only", "--json"],
        "import_seed",
    );
    assert!(
        import.status.success(),
        "import failed: stdout={} stderr={}",
        import.stdout,
        import.stderr
    );
}

fn coordination_json(workspace: &BrWorkspace, args: &[&str], label: &str) -> Value {
    let result = run_br(workspace, args, label);
    assert!(
        result.status.success(),
        "coordination status failed: stdout={} stderr={}",
        result.stdout,
        result.stderr
    );
    serde_json::from_str(&extract_json_payload(&result.stdout)).expect("coordination json")
}

#[test]
fn coordination_status_json_reports_fresh_and_stale_claims() {
    let _log = common::test_log("coordination_status_json_reports_fresh_and_stale_claims");
    let workspace = BrWorkspace::new();
    seed_coordination_workspace(&workspace);

    let json = coordination_json(
        &workspace,
        &[
            "coordination",
            "status",
            "--json",
            "--owner-kind",
            "swarm-agent",
        ],
        "coordination_json",
    );

    assert_eq!(json["schema_version"], "br.coordination.v1");
    assert_eq!(json["summary"]["total_claims"], 2);
    assert_eq!(json["summary"]["workspace"]["in_progress"], 2);
    let claims = json["claims"].as_array().expect("claims array");
    let fresh = claims
        .iter()
        .find(|claim| claim["issue"]["id"] == "bd-fresh")
        .expect("fresh claim");
    let stale = claims
        .iter()
        .find(|claim| claim["issue"]["id"] == "bd-stale")
        .expect("stale claim");

    assert_eq!(fresh["assessment"]["classification"], "fresh");
    assert_eq!(fresh["issue"]["labels"], json!(["coordination"]));
    assert_eq!(
        fresh["issue"]["latest_comments"][0]["text"],
        "fresh claim note"
    );
    assert_eq!(stale["assessment"]["classification"], "no_mail_snapshot");
    assert_eq!(stale["assessment"]["recommended_action"], "inspect_mail");
}

#[test]
fn coordination_status_text_is_concise_and_sanitized() {
    let _log = common::test_log("coordination_status_text_is_concise_and_sanitized");
    let workspace = BrWorkspace::new();
    seed_coordination_workspace(&workspace);

    let result = run_br(
        &workspace,
        ["coordination", "status", "--owner-kind", "swarm-agent"],
        "coordination_text",
    );

    assert!(result.status.success(), "text failed: {}", result.stderr);
    assert!(
        result
            .stdout
            .contains("Coordination status (2 in-progress claims):")
    );
    assert!(result.stdout.contains("bd-stale"));
    assert!(result.stdout.contains("classification: no_mail_snapshot"));
    assert!(result.stdout.contains("next_action: inspect_mail"));
    assert!(!result.stdout.contains('\u{1b}'));
    assert!(result.stdout.contains(r"\u{1b}[31m"));
}

#[test]
fn coordination_status_toon_decodes() {
    let _log = common::test_log("coordination_status_toon_decodes");
    let workspace = BrWorkspace::new();
    seed_coordination_workspace(&workspace);

    let result = run_br(
        &workspace,
        ["coordination", "status", "--format", "toon"],
        "coordination_toon",
    );

    assert!(result.status.success(), "toon failed: {}", result.stderr);
    let decoded = parse_toon(result.stdout.trim(), None).expect("valid TOON");
    let json = Value::from(decoded);
    assert_eq!(json["schema_version"], "br.coordination.v1");
    assert_eq!(json["claims"].as_array().expect("claims").len(), 2);
}
