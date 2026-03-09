use beads_rust::model::{Issue, IssueType, Status, Priority, Dependency, DepType};
use beads_rust::storage::{SqliteStorage, ListFilters};
use chrono::Utc;

#[test]
fn test_parent_blocks_child_ready() {
    let mut storage = SqliteStorage::open_memory().unwrap();

    // Create a parent issue
    let parent = Issue {
        id: "bd-parent".to_string(),
        title: "Parent Epic".to_string(),
        description: None,
        issue_type: IssueType::Epic,
        status: Status::Open,
        priority: Priority::Medium,
        assignee: None,
        labels: vec![],
        parent_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        closed_at: None,
        defer_until: None,
        due_at: None,
        content_hash: None,
        owner: None,
        estimated_minutes: None,
        created_by: Some("tester".to_string()),
        source_system: None,
        source_repo: None,
        external_ref: None,
        deleted_at: None,
        deleted_by: None,
        delete_reason: None,
        original_type: None,
        compaction_level: 0,
        compacted_at: None,
        compacted_at_commit: None,
        original_size: 0,
        sender: None,
        ephemeral: false,
        pinned: false,
        is_template: false,
        design: None,
        acceptance_criteria: None,
        notes: None,
        closed_by_session: None,
    };
    storage.create_issue(&parent, "tester").unwrap();

    // Create a child issue
    let child = Issue {
        id: "bd-child".to_string(),
        title: "Child Task".to_string(),
        description: None,
        issue_type: IssueType::Task,
        status: Status::Open,
        priority: Priority::Medium,
        assignee: None,
        labels: vec![],
        parent_id: Some("bd-parent".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        closed_at: None,
        defer_until: None,
        due_at: None,
        content_hash: None,
        owner: None,
        estimated_minutes: None,
        created_by: Some("tester".to_string()),
        source_system: None,
        source_repo: None,
        external_ref: None,
        deleted_at: None,
        deleted_by: None,
        delete_reason: None,
        original_type: None,
        compaction_level: 0,
        compacted_at: None,
        compacted_at_commit: None,
        original_size: 0,
        sender: None,
        ephemeral: false,
        pinned: false,
        is_template: false,
        design: None,
        acceptance_criteria: None,
        notes: None,
        closed_by_session: None,
    };
    storage.create_issue(&child, "tester").unwrap();

    // The parent-child relationship is also reflected in the dependencies table
    let dep = Dependency {
        issue_id: "bd-child".to_string(),
        depends_on_id: "bd-parent".to_string(),
        dep_type: DepType::ParentChild,
        created_at: Utc::now(),
        created_by: Some("tester".to_string()),
        metadata: None,
        thread_id: None,
    };
    storage.add_dependency(&dep, "tester").unwrap();

    // Manually trigger cache rebuild
    storage.rebuild_blocked_cache(true).unwrap();

    // Check if child is blocked
    let is_blocked = storage.is_blocked("bd-child").unwrap();
    assert!(is_blocked, "Child should be blocked by open parent");

    // Check if child is in ready issues
    let ready = storage.get_ready_issues(&ListFilters::default()).unwrap();
    let is_ready = ready.iter().any(|i| i.id == "bd-child");
    assert!(!is_ready, "Child should not be ready when parent is open");
}
