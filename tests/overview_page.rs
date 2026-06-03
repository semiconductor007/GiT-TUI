use gitinsight_rs::models::RepositorySummary;
use gitinsight_rs::ui::dashboard::overview_rows;

#[test]
fn overview_rows_match_repository_summary() {
    let mut summary = RepositorySummary::new("demo", "D:/repos/demo");
    summary.total_commits = 12;
    summary.total_branches = 3;
    summary.total_tags = 2;
    summary.total_contributors = 4;
    summary.total_files = 99;
    summary.total_loc = 12345;

    let rows = overview_rows(&summary);

    assert_eq!(
        rows,
        vec![
            ("Repository Name", "demo".to_owned()),
            ("Commits", "12".to_owned()),
            ("Branches", "3".to_owned()),
            ("Tags", "2".to_owned()),
            ("Contributors", "4".to_owned()),
        ]
    );
}
