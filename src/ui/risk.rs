//! 风险报告页面：展示综合风险等级和可解释风险原因。

use ratatui::prelude::Line;

use crate::analytics::{RiskLevel, RiskReport};

pub fn risk_rows(report: &RiskReport) -> Vec<String> {
    let mut rows = vec![format!("风险等级: {}", risk_level_label(report.risk_level))];
    rows.push("风险原因:".to_owned());
    rows.extend(report.reasons.iter().map(|reason| format!("- {reason}")));
    rows
}

pub fn risk_lines(report: Option<&RiskReport>) -> Vec<Line<'static>> {
    match report {
        Some(report) => risk_rows(report).into_iter().map(Line::from).collect(),
        None => vec![Line::from("风险报告尚未加载")],
    }
}

fn risk_level_label(risk_level: RiskLevel) -> &'static str {
    match risk_level {
        RiskLevel::Low => "低",
        RiskLevel::Medium => "中",
        RiskLevel::High => "高",
    }
}
