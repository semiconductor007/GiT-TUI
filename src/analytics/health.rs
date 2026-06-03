use serde::{Deserialize, Serialize};

use crate::analytics::RiskLevel;
use crate::git::{Analyzer, GitRepository};
use crate::utils::Result;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthScore {
    pub overall_score: u8,
    pub activity_score: u8,
    pub contributor_score: u8,
    pub bus_factor_score: u8,
    pub hotspot_score: u8,
}

impl HealthScore {
    pub fn new(
        activity_score: u8,
        contributor_score: u8,
        bus_factor_score: u8,
        hotspot_score: u8,
    ) -> Self {
        let overall_score = average_score([
            activity_score,
            contributor_score,
            bus_factor_score,
            hotspot_score,
        ]);

        Self {
            overall_score,
            activity_score,
            contributor_score,
            bus_factor_score,
            hotspot_score,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct HealthAnalyzer;

impl Analyzer for HealthAnalyzer {
    type Output = HealthScore;

    fn analyze(&self, repo: &GitRepository) -> Result<Self::Output> {
        super::validate_repository(repo)?;

        let summary = repo.summary()?;
        let contributors = repo.contributors()?;
        let bus_factor = repo.bus_factor()?;
        let hotspots = repo.hotspots()?;

        Ok(HealthScore::new(
            score_commit_activity(summary.total_commits),
            score_contributors(contributors.len()),
            score_bus_factor(bus_factor.risk_level, bus_factor.bus_factor),
            score_hotspots(
                hotspots
                    .iter()
                    .map(|hotspot| hotspot.change_count)
                    .collect::<Vec<_>>()
                    .as_slice(),
            ),
        ))
    }
}

fn average_score(scores: [u8; 4]) -> u8 {
    let total = scores.iter().map(|score| u16::from(*score)).sum::<u16>();
    (total / scores.len() as u16) as u8
}

fn score_commit_activity(total_commits: usize) -> u8 {
    match total_commits {
        0 => 0,
        1..=2 => 40,
        3..=9 => 70,
        _ => 100,
    }
}

fn score_contributors(total_contributors: usize) -> u8 {
    match total_contributors {
        0 => 0,
        1 => 35,
        2 => 70,
        _ => 100,
    }
}

fn score_bus_factor(risk_level: RiskLevel, bus_factor: usize) -> u8 {
    match risk_level {
        RiskLevel::Low => 100,
        RiskLevel::Medium => 70,
        RiskLevel::High if bus_factor == 0 => 0,
        RiskLevel::High => 35,
    }
}

fn score_hotspots(change_counts: &[usize]) -> u8 {
    let total_changes = change_counts.iter().sum::<usize>();
    if total_changes == 0 {
        return 0;
    }

    let top_changes = change_counts.iter().copied().max().unwrap_or(0);
    let concentration = top_changes as f64 / total_changes as f64;

    if concentration <= 0.4 {
        100
    } else if concentration <= 0.6 {
        75
    } else if concentration <= 0.8 {
        50
    } else {
        25
    }
}
