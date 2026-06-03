//! Bus Factor 分析：识别关键贡献者集中度和维护风险。

use serde::{Deserialize, Serialize};

use crate::git::{Analyzer, GitRepository};
use crate::models::ContributorStats;
use crate::utils::Result;

const BUS_FACTOR_THRESHOLD: f64 = 0.5;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    #[default]
    High,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct BusFactorReport {
    pub bus_factor: usize,
    pub top_contributors: Vec<String>,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Default)]
pub struct BusFactorAnalyzer;

impl BusFactorAnalyzer {
    pub fn from_contributors(contributors: &[ContributorStats]) -> BusFactorReport {
        calculate_bus_factor(contributors)
    }
}

impl Analyzer for BusFactorAnalyzer {
    type Output = BusFactorReport;

    fn analyze(&self, repo: &GitRepository) -> Result<Self::Output> {
        super::validate_repository(repo)?;

        let contributors = repo.contributors()?;
        Ok(Self::from_contributors(&contributors))
    }
}

fn calculate_bus_factor(contributors: &[ContributorStats]) -> BusFactorReport {
    let total_commits = contributors
        .iter()
        .map(|contributor| contributor.commit_count)
        .sum::<usize>();

    if total_commits == 0 {
        return BusFactorReport {
            bus_factor: 0,
            top_contributors: Vec::new(),
            risk_level: RiskLevel::High,
        };
    }

    let mut sorted = contributors.to_vec();
    sorted.sort_by(|left, right| {
        right
            .commit_count
            .cmp(&left.commit_count)
            .then_with(|| left.email.cmp(&right.email))
    });

    let mut cumulative_commits = 0;
    let mut top_contributors = Vec::new();
    for contributor in sorted {
        cumulative_commits += contributor.commit_count;
        top_contributors.push(contributor.name);

        let contribution_ratio = cumulative_commits as f64 / total_commits as f64;
        if contribution_ratio >= BUS_FACTOR_THRESHOLD {
            break;
        }
    }

    let bus_factor = top_contributors.len();
    BusFactorReport {
        bus_factor,
        top_contributors,
        risk_level: risk_level_for(bus_factor),
    }
}

fn risk_level_for(bus_factor: usize) -> RiskLevel {
    match bus_factor {
        0 | 1 => RiskLevel::High,
        2 => RiskLevel::Medium,
        _ => RiskLevel::Low,
    }
}
