//! 健康度页面：展示仓库评分、阶段、风险等级和评分解释。

use ratatui::prelude::Line;

use crate::analytics::{BusFactorReport, HealthScore, RiskLevel};

pub fn health_rows(score: &HealthScore, bus_factor: &BusFactorReport) -> Vec<String> {
    vec![
        format!("健康度评分: {}/100", score.overall_score),
        format!(
            "仓库阶段: {}",
            repository_stage_label(score.repository_stage)
        ),
        format!("Bus Factor: {}", bus_factor.bus_factor),
        format!("风险等级: {}", risk_level_label(bus_factor.risk_level)),
        format!("提交活跃度: {}/100", score.activity_score),
        format!("贡献者分布: {}/100", score.contributor_score),
        format!("Bus Factor 分数: {}/100", score.bus_factor_score),
        format!("文件热点分数: {}/100", score.hotspot_score),
        health_explanation(score),
    ]
}

pub fn health_lines(
    score: Option<&HealthScore>,
    bus_factor: Option<&BusFactorReport>,
) -> Vec<Line<'static>> {
    match (score, bus_factor) {
        (Some(score), Some(bus_factor)) => health_rows(score, bus_factor)
            .into_iter()
            .map(Line::from)
            .collect(),
        _ => vec![Line::from("健康度评分尚未加载")],
    }
}

fn health_explanation(score: &HealthScore) -> String {
    if score.repository_stage == crate::analytics::RepositoryStage::EarlyStage {
        return "评分说明: Early Stage Repository，提交样本较少，活跃度、贡献者分布、Bus Factor 与文件热点均采用保守评分"
            .to_owned();
    }

    format!(
        "评分说明: 提交活跃度{}，贡献者分布{}，Bus Factor {}，文件热点{}",
        explain_score(score.activity_score),
        explain_score(score.contributor_score),
        explain_score(score.bus_factor_score),
        explain_score(score.hotspot_score)
    )
}

fn explain_score(score: u8) -> &'static str {
    match score {
        80..=100 => "较好",
        50..=79 => "一般",
        1..=49 => "较弱",
        _ => "缺失",
    }
}

fn repository_stage_label(repository_stage: crate::analytics::RepositoryStage) -> &'static str {
    match repository_stage {
        crate::analytics::RepositoryStage::EarlyStage => "Early Stage Repository",
        crate::analytics::RepositoryStage::Established => "Established Repository",
    }
}

fn risk_level_label(risk_level: RiskLevel) -> &'static str {
    match risk_level {
        RiskLevel::Low => "低",
        RiskLevel::Medium => "中",
        RiskLevel::High => "高",
    }
}
