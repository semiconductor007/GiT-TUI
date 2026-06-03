//! 风险分析：基于健康度、Bus Factor、热点和 Ownership 生成可解释风险报告。

use serde::{Deserialize, Serialize};

use crate::analytics::{BusFactorReport, FileHotspot, HealthScore, RiskLevel};
use crate::git::{Analyzer, GitRepository};
use crate::models::ContributorStats;
use crate::utils::Result;

const HIGH_OWNERSHIP_PERCENT: f64 = 80.0;
const MEDIUM_OWNERSHIP_PERCENT: f64 = 50.0;
const HIGH_HOTSPOT_CONCENTRATION_PERCENT: f64 = 70.0;
const MEDIUM_HOTSPOT_CONCENTRATION_PERCENT: f64 = 50.0;
const HIGH_HEALTH_THRESHOLD: u8 = 60;
const MEDIUM_HEALTH_THRESHOLD: u8 = 75;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskReport {
    pub risk_level: RiskLevel,
    pub reasons: Vec<String>,
}

impl Default for RiskReport {
    fn default() -> Self {
        Self {
            risk_level: RiskLevel::Low,
            reasons: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RiskAnalyzer;

impl RiskAnalyzer {
    pub fn from_analysis(
        contributors: &[ContributorStats],
        bus_factor: &BusFactorReport,
        health_score: &HealthScore,
        hotspots: &[FileHotspot],
    ) -> RiskReport {
        let mut risk_level = RiskLevel::Low;
        let mut reasons = Vec::new();

        apply_bus_factor_risk(bus_factor, &mut risk_level, &mut reasons);
        apply_ownership_risk(contributors, &mut risk_level, &mut reasons);
        apply_hotspot_risk(hotspots, &mut risk_level, &mut reasons);
        apply_health_risk(health_score, &mut risk_level, &mut reasons);

        if reasons.is_empty() {
            reasons.push("No major repository risk signals detected".to_owned());
        }

        RiskReport {
            risk_level,
            reasons,
        }
    }
}

impl Analyzer for RiskAnalyzer {
    type Output = RiskReport;

    fn analyze(&self, repo: &GitRepository) -> Result<Self::Output> {
        super::validate_repository(repo)?;

        let contributors = repo.contributors()?;
        let bus_factor = repo.bus_factor()?;
        let health_score = repo.health_score()?;
        let hotspots = repo.hotspots()?;

        Ok(Self::from_analysis(
            &contributors,
            &bus_factor,
            &health_score,
            &hotspots,
        ))
    }
}

fn apply_bus_factor_risk(
    bus_factor: &BusFactorReport,
    risk_level: &mut RiskLevel,
    reasons: &mut Vec<String>,
) {
    match bus_factor.risk_level {
        RiskLevel::High => {
            promote_risk(risk_level, RiskLevel::High);
            reasons.push(format!("Bus Factor = {}", bus_factor.bus_factor));
        }
        RiskLevel::Medium => {
            promote_risk(risk_level, RiskLevel::Medium);
            reasons.push(format!("Bus Factor = {}", bus_factor.bus_factor));
        }
        RiskLevel::Low => {}
    }
}

fn apply_ownership_risk(
    contributors: &[ContributorStats],
    risk_level: &mut RiskLevel,
    reasons: &mut Vec<String>,
) {
    if let Some(top_contributor) = contributors.iter().max_by(|left, right| {
        left.ownership_percent
            .total_cmp(&right.ownership_percent)
            .then_with(|| right.email.cmp(&left.email))
    }) {
        if top_contributor.ownership_percent >= HIGH_OWNERSHIP_PERCENT {
            promote_risk(risk_level, RiskLevel::High);
            reasons.push(format!(
                "Single contributor owns {:.1}% of commits",
                top_contributor.ownership_percent
            ));
        } else if top_contributor.ownership_percent >= MEDIUM_OWNERSHIP_PERCENT {
            promote_risk(risk_level, RiskLevel::Medium);
            reasons.push(format!(
                "Leading contributor owns {:.1}% of commits",
                top_contributor.ownership_percent
            ));
        }
    }
}

fn apply_hotspot_risk(
    hotspots: &[FileHotspot],
    risk_level: &mut RiskLevel,
    reasons: &mut Vec<String>,
) {
    let concentration = top_hotspot_concentration(hotspots);
    if concentration >= HIGH_HOTSPOT_CONCENTRATION_PERCENT {
        promote_risk(risk_level, RiskLevel::High);
        reasons.push(format!(
            "Top 3 hotspot files account for {:.1}% of changes",
            concentration
        ));
    } else if concentration >= MEDIUM_HOTSPOT_CONCENTRATION_PERCENT {
        promote_risk(risk_level, RiskLevel::Medium);
        reasons.push(format!(
            "Top 3 hotspot files account for {:.1}% of changes",
            concentration
        ));
    }
}

fn apply_health_risk(
    health_score: &HealthScore,
    risk_level: &mut RiskLevel,
    reasons: &mut Vec<String>,
) {
    if health_score.overall_score < HIGH_HEALTH_THRESHOLD {
        promote_risk(risk_level, RiskLevel::High);
        reasons.push(format!(
            "Health score below {HIGH_HEALTH_THRESHOLD}: {}/100",
            health_score.overall_score
        ));
    } else if health_score.overall_score < MEDIUM_HEALTH_THRESHOLD {
        promote_risk(risk_level, RiskLevel::Medium);
        reasons.push(format!(
            "Health score below {MEDIUM_HEALTH_THRESHOLD}: {}/100",
            health_score.overall_score
        ));
    }
}

fn top_hotspot_concentration(hotspots: &[FileHotspot]) -> f64 {
    let total_changes = hotspots
        .iter()
        .map(|hotspot| hotspot.change_count)
        .sum::<usize>();
    if total_changes == 0 {
        return 0.0;
    }

    let mut change_counts = hotspots
        .iter()
        .map(|hotspot| hotspot.change_count)
        .collect::<Vec<_>>();
    change_counts.sort_by(|left, right| right.cmp(left));
    let top_three_changes = change_counts.into_iter().take(3).sum::<usize>();

    top_three_changes as f64 / total_changes as f64 * 100.0
}

fn promote_risk(current: &mut RiskLevel, candidate: RiskLevel) {
    if risk_rank(candidate) > risk_rank(*current) {
        *current = candidate;
    }
}

fn risk_rank(risk_level: RiskLevel) -> u8 {
    match risk_level {
        RiskLevel::Low => 0,
        RiskLevel::Medium => 1,
        RiskLevel::High => 2,
    }
}
