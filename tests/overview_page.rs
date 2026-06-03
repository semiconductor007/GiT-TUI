//! 概览页面测试：验证仓库摘要字段能正确转换为展示行。

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
            ("仓库名称", "demo".to_owned()),
            ("提交总数", "12".to_owned()),
            ("本地分支", "3".to_owned()),
            ("标签数量", "2".to_owned()),
            ("贡献者数", "4".to_owned()),
        ]
    );
}
