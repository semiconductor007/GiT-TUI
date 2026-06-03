use gitinsight_rs::models::ContributorStats;
use gitinsight_rs::ui::contributors::{contributor_row, contributor_rows};

#[test]
fn contributor_row_contains_name_commit_count_and_active_days() {
    let mut contributor = ContributorStats::new("Alice", "alice@example.com");
    contributor.commit_count = 12;
    contributor.active_days = 5;

    let row = contributor_row(&contributor);

    assert!(row.contains("Alice"));
    assert!(row.contains("12 commits"));
    assert!(row.contains("5 active days"));
}

#[test]
fn contributor_rows_preserve_given_order() {
    let mut alice = ContributorStats::new("Alice", "alice@example.com");
    alice.commit_count = 3;
    let mut bob = ContributorStats::new("Bob", "bob@example.com");
    bob.commit_count = 1;

    let rows = contributor_rows(&[alice, bob]);

    assert!(rows[0].contains("Alice"));
    assert!(rows[1].contains("Bob"));
}
