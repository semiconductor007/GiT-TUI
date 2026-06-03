//! 贡献者页面：把贡献者统计数据格式化为终端可读列表。

use ratatui::prelude::Line;

use crate::models::ContributorStats;

pub fn contributor_rows(contributors: &[ContributorStats]) -> Vec<String> {
    contributors.iter().map(contributor_row).collect()
}

pub fn contributor_row(stats: &ContributorStats) -> String {
    format!(
        "{:<18} {:>5} 次提交 {:>4} 个活跃日  Ownership {:>5.1}%",
        stats.name, stats.commit_count, stats.active_days, stats.ownership_percent
    )
}

pub fn contributor_lines(
    contributors: &[ContributorStats],
    selected_row: usize,
) -> Vec<Line<'static>> {
    let rows = contributor_rows(contributors);
    visible_rows(&rows, selected_row, 12)
        .into_iter()
        .map(Line::from)
        .collect()
}

fn visible_rows(rows: &[String], selected_row: usize, height: usize) -> Vec<String> {
    if rows.is_empty() {
        return vec!["暂无贡献者数据".to_owned()];
    }

    let start = selected_row.min(rows.len().saturating_sub(1));
    rows.iter().skip(start).take(height).cloned().collect()
}
