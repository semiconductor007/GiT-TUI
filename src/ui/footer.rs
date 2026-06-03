//! 底部帮助栏：统一展示全局快捷键，并高亮当前页面对应入口。

use ratatui::layout::Rect;
use ratatui::prelude::{Color, Frame, Line, Modifier, Span, Style};
use ratatui::widgets::Paragraph;

use crate::app::state::{AppState, Tab};

const WIDE_WIDTH: u16 = 96;
const MEDIUM_WIDTH: u16 = 65;

#[derive(Debug, Clone, Copy)]
struct FooterTab {
    tab: Tab,
    wide_label: &'static str,
    medium_label: &'static str,
}

const FOOTER_TABS: [FooterTab; 6] = [
    FooterTab {
        tab: Tab::Overview,
        wide_label: "[1]概览",
        medium_label: "[1]概",
    },
    FooterTab {
        tab: Tab::Contributors,
        wide_label: "[2]贡献者",
        medium_label: "[2]贡",
    },
    FooterTab {
        tab: Tab::Timeline,
        wide_label: "[3]时间线",
        medium_label: "[3]线",
    },
    FooterTab {
        tab: Tab::Hotspots,
        wide_label: "[4]热点",
        medium_label: "[4]热",
    },
    FooterTab {
        tab: Tab::Health,
        wide_label: "[5]健康度",
        medium_label: "[5]健",
    },
    FooterTab {
        tab: Tab::Risk,
        wide_label: "[6]风险报告",
        medium_label: "[6]险",
    },
];

pub fn render_footer(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let footer = Paragraph::new(footer_lines(state.active_tab, area.width));
    frame.render_widget(footer, area);
}

pub fn footer_lines(active_tab: Tab, width: u16) -> Vec<Line<'static>> {
    vec![
        Line::from(footer_tab_spans(active_tab, width)),
        Line::from(footer_help_text(width)),
    ]
}

pub fn footer_tab_spans(active_tab: Tab, width: u16) -> Vec<Span<'static>> {
    if width < MEDIUM_WIDTH {
        return vec![
            active_tab_span(active_tab, active_tab.title()),
            Span::raw("  数字切页 ..."),
        ];
    }

    let use_medium_labels = width < WIDE_WIDTH;
    let mut spans = Vec::new();
    for (index, footer_tab) in FOOTER_TABS.iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw("  "));
        }

        let label = if use_medium_labels {
            footer_tab.medium_label
        } else {
            footer_tab.wide_label
        };

        if footer_tab.tab == active_tab {
            spans.push(active_tab_span(active_tab, label));
        } else {
            spans.push(Span::raw(label));
        }
    }

    spans
}

fn active_tab_span<'a>(active_tab: Tab, label: &'a str) -> Span<'a> {
    Span::styled(
        active_label(active_tab, label),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )
}

fn active_label(active_tab: Tab, label: &str) -> String {
    if label.starts_with('[') {
        label.to_owned()
    } else {
        format!("[{}]{label}", tab_number(active_tab))
    }
}

fn footer_help_text(width: u16) -> &'static str {
    if width < MEDIUM_WIDTH {
        "←/→切页  ↑/↓滚动  Home/End  q退出"
    } else {
        "← → 切换页面   ↑ ↓ 滚动内容   Home 顶部   End 底部   q 退出"
    }
}

fn tab_number(tab: Tab) -> char {
    match tab {
        Tab::Overview => '1',
        Tab::Contributors => '2',
        Tab::Timeline => '3',
        Tab::Hotspots => '4',
        Tab::Health => '5',
        Tab::Risk => '6',
    }
}
