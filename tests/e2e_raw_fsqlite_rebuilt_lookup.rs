mod common;

use common::cli::{BrWorkspace, run_br};
use fsqlite::Connection;
use fsqlite_types::SqliteValue;
use serde_json::Value;

fn extract_json(run_stdout: &str) -> Value {
    let payload = common::cli::extract_json_payload(run_stdout);
    serde_json::from_str(&payload).expect("valid cli json payload")
}

fn scan_issue_ids(conn: &Connection) -> Vec<String> {
    conn.query("SELECT id FROM issues ORDER BY rowid")
        .unwrap()
        .into_iter()
        .filter_map(|row| {
            row.values()
                .first()
                .and_then(SqliteValue::as_text)
                .map(ToOwned::to_owned)
        })
        .collect()
}

fn keyed_issue_ids(conn: &Connection, id: &str) -> Vec<String> {
    conn.query_with_params(
        "SELECT id FROM issues WHERE id = ?",
        &[SqliteValue::from(id)],
    )
    .unwrap()
    .into_iter()
    .filter_map(|row| {
        row.values()
            .first()
            .and_then(SqliteValue::as_text)
            .map(ToOwned::to_owned)
    })
    .collect()
}

fn keyed_issue_row(conn: &Connection, id: &str) -> Option<String> {
    conn.query_row_with_params(
        "SELECT id FROM issues WHERE id = ?",
        &[SqliteValue::from(id)],
    )
    .ok()
    .and_then(|row| {
        row.values()
            .first()
            .and_then(SqliteValue::as_text)
            .map(ToOwned::to_owned)
    })
}

#[test]
fn e2e_raw_fsqlite_keyed_lookup_matches_full_scan_after_alt_rebuild() {
    const LOOP_COUNT: usize = 20;

    let _log = common::test_log("e2e_raw_fsqlite_keyed_lookup_matches_full_scan_after_alt_rebuild");
    let workspace = BrWorkspace::new();

    let init = run_br(&workspace, ["init", "--prefix", "raw"], "init_raw_workspace");
    assert!(init.status.success(), "init failed: {}", init.stderr);

    for title in ["seed A", "seed B"] {
        let create = run_br(
            &workspace,
            [
                "create",
                "--title",
                title,
                "--type",
                "task",
                "--priority",
                "2",
                "--json",
            ],
            &format!("create_{title}"),
        );
        assert!(
            create.status.success(),
            "seed create failed for {title}: stdout={} stderr={}",
            create.stdout,
            create.stderr
        );
    }

    let flush = run_br(&workspace, ["sync", "--flush-only"], "flush_before_alt_rebuild");
    assert!(flush.status.success(), "flush failed: {}", flush.stderr);

    let alt_db = workspace.root.join(".beads").join("beads.raw-rebuilt.db");
    let rebuild = run_br(
        &workspace,
        [
            "--db",
            alt_db.to_str().expect("alt db path"),
            "sync",
            "--import-only",
            "--rebuild",
            "--json",
            "--no-auto-import",
            "--no-auto-flush",
        ],
        "rebuild_alt_db",
    );
    assert!(
        rebuild.status.success(),
        "alt rebuild failed: stdout={} stderr={}",
        rebuild.stdout,
        rebuild.stderr
    );

    for i in 0..LOOP_COUNT {
        let title = format!("raw lookup loop {i}");
        let create = run_br(
            &workspace,
            [
                "--db",
                alt_db.to_str().expect("alt db path"),
                "create",
                "--title",
                &title,
                "--type",
                "task",
                "--priority",
                "2",
                "--json",
            ],
            &format!("loop_create_{i}"),
        );
        assert!(
            create.status.success(),
            "create failed on loop {i}: stdout={} stderr={}",
            create.stdout,
            create.stderr
        );
        let create_json = extract_json(&create.stdout);
        let fresh_id = create_json["id"]
            .as_str()
            .expect("fresh id")
            .to_string();

        let conn = Connection::open(alt_db.to_string_lossy().into_owned()).unwrap();

        let scanned = scan_issue_ids(&conn);
        assert!(
            scanned.iter().any(|existing| existing == &fresh_id),
            "full scan could not find freshly created id {fresh_id} on loop {i}"
        );

        let keyed = keyed_issue_ids(&conn, &fresh_id);
        assert_eq!(
            keyed,
            vec![fresh_id.clone()],
            "raw fsqlite keyed query_with_params diverged from full scan for {fresh_id} on loop {i}; scanned_tail={:?}",
            &scanned[scanned.len().saturating_sub(10)..]
        );

        let keyed_row = keyed_issue_row(&conn, &fresh_id);
        assert_eq!(
            keyed_row.as_deref(),
            Some(fresh_id.as_str()),
            "raw fsqlite query_row_with_params diverged for {fresh_id} on loop {i}; keyed={keyed:?}"
        );
    }
}
