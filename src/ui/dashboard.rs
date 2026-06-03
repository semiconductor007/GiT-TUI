//! Dashboard 渲染：绘制标题、标签页、内容区域和统一底部快捷键栏。

use ratatui::prelude::{Constraint, Direction, Frame, Layout, Line, Modifier, Span, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};

use crate::app::state::{AppState, Tab};
use crate::models::RepositorySummary;
use crate::ui::contributors::contributor_lines;
use crate::ui::footer::render_footer;
use crate::ui::health::health_lines;
use crate::ui::hotspot::{DEFAULT_HOTSPOT_TOP_N, hotspot_lines};
use crate::ui::risk::risk_lines;
use crate::ui::timeline::timeline_lines;

pub fn overview_rows(summary: &RepositorySummary) -> Vec<(&'static str, String)> {
    vec![
        ("仓库名称", summary.name.clone()),
        ("提交总数", summary.total_commits.to_string()),
        ("本地分支", summary.total_branches.to_string()),
        ("标签数量", summary.total_tags.to_string()),
        ("贡献者数", summary.total_contributors.to_string()),
    ]
}

pub fn overview_lines(summary: &RepositorySummary) -> Vec<Line<'static>> {
    overview_rows(summary)
        .into_iter()
        .map(|(label, value)| Line::from(vec![Span::raw(format!("{label}: ")), Span::raw(value)]))
        .collect()
}

pub fn draw_dashboard(frame: &mut Frame<'_>, state: &AppState) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(2),
        ])
        .split(frame.area());

    render_title(frame, state, root[0]);
    render_tabs(frame, state, root[1]);
    render_content_shell(frame, state, root[2]);
    render_footer(frame, root[3], state);
}

fn render_title(frame: &mut Frame<'_>, state: &AppState, area: ratatui::layout::Rect) {
    let title = state
        .repository
        .as_ref()
        .map(|summary| format!("GitInsight-RS | {}", summary.name))
        .unwrap_or_else(|| "GitInsight-RS".to_owned());
    let block = Block::default().title(title).borders(Borders::ALL);

    frame.render_widget(block, area);
}

fn render_tabs(frame: &mut Frame<'_>, state: &AppState, area: ratatui::layout::Rect) {
    let titles = Tab::ALL
        .iter()
        .map(|tab| Line::from(tab.title()))
        .collect::<Vec<_>>();
    let selected = Tab::ALL
        .iter()
        .position(|tab| *tab == state.active_tab)
        .unwrap_or_default();
    let tabs = Tabs::new(titles)
        .select(selected)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(tabs, area);
}

fn render_content_shell(frame: &mut Frame<'_>, state: &AppState, area: ratatui::layout::Rect) {
    match state.active_tab {
        Tab::Overview => render_overview(frame, state, area),
        Tab::Contributors => render_contributors(frame, state, area),
        Tab::Timeline => render_timeline(frame, state, area),
        Tab::Hotspots => render_hotspots(frame, state, area),
        Tab::Health => render_health(frame, state, area),
        Tab::Risk => render_risk(frame, state, area),
    }
}

fn render_overview(frame: &mut Frame<'_>, state: &AppState, area: ratatui::layout::Rect) {
    let lines = state
        .repository
        .as_ref()
        .map(overview_lines)
        .unwrap_or_else(|| vec![Line::from("仓库概览尚未加载")]);
    let content =
        Paragraph::new(lines).block(Block::default().title("仓库概览").borders(Borders::ALL));

    frame.render_widget(content, area);
}

fn render_contributors(frame: &mut Frame<'_>, state: &AppState, area: ratatui::layout::Rect) {
    let title = format!("贡献者 | 按{}排序", state.contributor_sort_mode.title());
    let content = Paragraph::new(contributor_lines(&state.contributors, state.selected_row))
        .block(Block::default().title(title).borders(Borders::ALL));

    frame.render_widget(content, area);
}

fn render_timeline(frame: &mut Frame<'_>, state: &AppState, area: ratatui::layout::Rect) {
    let content = Paragraph::new(timeline_lines(&state.timeline, state.selected_row)).block(
        Block::default()
            .title("提交时间线 | 最左列为提交ID，即Git哈希前8位")
            .borders(Borders::ALL),
    );

    frame.render_widget(content, area);
}

fn render_hotspots(frame: &mut Frame<'_>, state: &AppState, area: ratatui::layout::Rect) {
    let title = format!("文件热点 | 前 {DEFAULT_HOTSPOT_TOP_N} 项");
    let content = Paragraph::new(hotspot_lines(
        &state.hotspots,
        state.selected_row,
        DEFAULT_HOTSPOT_TOP_N,
    ))
    .block(Block::default().title(title).borders(Borders::ALL));

    frame.render_widget(content, area);
}

fn render_health(frame: &mut Frame<'_>, state: &AppState, area: ratatui::layout::Rect) {
    let content = Paragraph::new(health_lines(
        state.health_score.as_ref(),
        state.bus_factor.as_ref(),
    ))
    .block(Block::default().title("仓库健康度").borders(Borders::ALL));

    frame.render_widget(content, area);
}

fn render_risk(frame: &mut Frame<'_>, state: &AppState, area: ratatui::layout::Rect) {
    let content = Paragraph::new(risk_lines(state.risk_report.as_ref()))
        .block(Block::default().title("风险报告").borders(Borders::ALL));

    frame.render_widget(content, area);
}
