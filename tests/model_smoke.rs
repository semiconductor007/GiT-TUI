//! 模型冒烟测试：验证核心数据模型的基础构造和统计方法。

use chrono::DateTime;
use gitinsight_rs::models::{
    ChangeKind, CommitInfo, ContributorStats, FileChange, RepositorySummary,
};

#[test]
fn repository_summary_starts_with_zero_counts() {
    let summary = RepositorySummary::new("demo", "D:/repos/demo");

    assert_eq!(summary.name, "demo");
    assert_eq!(summary.total_commits, 0);
    assert_eq!(summary.total_contributors, 0);
}

#[test]
fn commit_with_changes_updates_line_totals() {
    let timestamp = DateTime::from_timestamp(0, 0).expect("unix epoch should be valid");
    let changes = vec![
        FileChange::new("src/main.rs", ChangeKind::Modified, 10, 2),
        FileChange::new("src/lib.rs", ChangeKind::Added, 20, 0),
    ];

    let commit = CommitInfo::new(
        "abcdef123456",
        "Tom",
        "tom@example.com",
        timestamp,
        "initial model",
    )
    .with_changes(changes);

    assert_eq!(commit.short_id, "abcdef12");
    assert_eq!(commit.lines_added, 30);
    assert_eq!(commit.lines_deleted, 2);
    assert_eq!(commit.changed_lines(), 32);
}

#[test]
fn contributor_records_commit_totals() {
    let mut contributor = ContributorStats::new("Alice", "alice@example.com");
    let first_commit = DateTime::from_timestamp(0, 0).expect("unix epoch should be valid");
    let second_commit = DateTime::from_timestamp(60, 0).expect("timestamp should be valid");

    contributor.record_commit(first_commit, 7, 3);
    contributor.record_commit(second_commit, 5, 1);

    assert_eq!(contributor.commit_count, 2);
    assert_eq!(contributor.lines_added, 12);
    assert_eq!(contributor.lines_deleted, 4);
    assert_eq!(contributor.changed_lines(), 16);
    contributor.set_ownership(8);
    assert_eq!(contributor.ownership_percent, 25.0);
    assert_eq!(contributor.first_commit, Some(first_commit));
    assert_eq!(contributor.last_commit, Some(second_commit));
}
