//! Footer 测试：验证底部快捷键栏文本、窄屏缩短和当前页面高亮。

use gitinsight_rs::app::state::Tab;
use gitinsight_rs::ui::footer::footer_tab_spans;
use ratatui::prelude::{Color, Modifier, Style};

#[test]
fn footer_highlights_active_tab() {
    let spans = footer_tab_spans(Tab::Hotspots, 120);
    let expected_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);

    let active_span = spans
        .iter()
        .find(|span| span.content.as_ref() == "[4]热点")
        .expect("active hotspot tab should be visible");

    assert_eq!(active_span.style, expected_style);
}

#[test]
fn footer_uses_compact_text_on_narrow_width() {
    let spans = footer_tab_spans(Tab::Risk, 40);
    let text = spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert!(text.contains("[6]风险报告"));
    assert!(text.contains("数字切页"));
}
