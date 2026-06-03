use ratatui::prelude::Line;

use crate::models::ContributorStats;

pub fn contributor_rows(contributors: &[ContributorStats]) -> Vec<String> {
    contributors.iter().map(contributor_row).collect()
}

pub fn contributor_row(stats: &ContributorStats) -> String {
    format!(
        "{:<20} {:>6} commits {:>4} active days",
        stats.name, stats.commit_count, stats.active_days
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
        return vec!["No contributors found".to_owned()];
    }

    let start = selected_row.min(rows.len().saturating_sub(1));
    rows.iter().skip(start).take(height).cloned().collect()
}
