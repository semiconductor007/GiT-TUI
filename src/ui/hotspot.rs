use ratatui::prelude::Line;

use crate::analytics::FileHotspot;

pub const DEFAULT_HOTSPOT_TOP_N: usize = 20;

pub fn hotspot_rows(hotspots: &[FileHotspot], top_n: usize) -> Vec<String> {
    hotspots.iter().take(top_n).map(hotspot_row).collect()
}

pub fn hotspot_row(hotspot: &FileHotspot) -> String {
    format!("{:<48} {:>6} changes", hotspot.path, hotspot.change_count)
}

pub fn hotspot_lines(
    hotspots: &[FileHotspot],
    selected_row: usize,
    top_n: usize,
) -> Vec<Line<'static>> {
    let rows = hotspot_rows(hotspots, top_n);
    visible_rows(&rows, selected_row, 12)
        .into_iter()
        .map(Line::from)
        .collect()
}

fn visible_rows(rows: &[String], selected_row: usize, height: usize) -> Vec<String> {
    if rows.is_empty() {
        return vec!["No hotspots found".to_owned()];
    }

    let start = selected_row.min(rows.len().saturating_sub(1));
    rows.iter().skip(start).take(height).cloned().collect()
}
