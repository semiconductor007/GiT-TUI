use ratatui::prelude::Line;

use crate::models::TimelineEntry;
use crate::utils::time::format_utc_time;

pub fn timeline_rows(commits: &[TimelineEntry]) -> Vec<String> {
    commits.iter().map(timeline_row).collect()
}

pub fn timeline_row(commit: &TimelineEntry) -> String {
    format!(
        "{:<8} {:<18} {:<16} {}",
        commit.commit_id,
        commit.author_name,
        format_utc_time(&commit.commit_time),
        commit.message
    )
}

pub fn timeline_lines(commits: &[TimelineEntry], selected_row: usize) -> Vec<Line<'static>> {
    let rows = timeline_rows(commits);
    visible_rows(&rows, selected_row, 12)
        .into_iter()
        .map(Line::from)
        .collect()
}

fn visible_rows(rows: &[String], selected_row: usize, height: usize) -> Vec<String> {
    if rows.is_empty() {
        return vec!["No commits found".to_owned()];
    }

    let start = selected_row.min(rows.len().saturating_sub(1));
    rows.iter().skip(start).take(height).cloned().collect()
}
