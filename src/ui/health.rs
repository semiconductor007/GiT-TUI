use ratatui::prelude::Line;

use crate::analytics::{BusFactorReport, HealthScore};

pub fn health_rows(score: &HealthScore, bus_factor: &BusFactorReport) -> Vec<String> {
    vec![
        format!("Health Score: {}/100", score.overall_score),
        format!("Bus Factor: {}", bus_factor.bus_factor),
        format!("Risk Level: {:?}", bus_factor.risk_level),
        format!("Activity Score: {}/100", score.activity_score),
        format!("Contributor Score: {}/100", score.contributor_score),
        format!("Bus Factor Score: {}/100", score.bus_factor_score),
        format!("Hotspot Score: {}/100", score.hotspot_score),
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
        _ => vec![Line::from("Health score is not loaded")],
    }
}

fn health_explanation(score: &HealthScore) -> String {
    format!(
        "Explanation: activity {}, contributors {}, bus factor {}, hotspots {}",
        explain_score(score.activity_score),
        explain_score(score.contributor_score),
        explain_score(score.bus_factor_score),
        explain_score(score.hotspot_score)
    )
}

fn explain_score(score: u8) -> &'static str {
    match score {
        80..=100 => "strong",
        50..=79 => "moderate",
        1..=49 => "weak",
        _ => "missing",
    }
}
