//! 健康度分析：综合活跃度、贡献者分布、Bus Factor 和文件热点评分。

use serde::{Deserialize, Serialize};

use crate::analytics::RiskLevel;
use crate::git::{Analyzer, GitRepository};
use crate::utils::Result;

const EARLY_STAGE_COMMIT_THRESHOLD: usize = 5;
const EARLY_STAGE_ACTIVITY_CAP: u8 = 35;
const EARLY_STAGE_DISTRIBUTION_CAP: u8 = 60;
const EARLY_STAGE_HOTSPOT_CAP: u8 = 35;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum RepositoryStage {
    #[default]
    EarlyStage,
    Established,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthScore {
    pub repository_stage: RepositoryStage,
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
        Self::with_stage(
            RepositoryStage::Established,
            activity_score,
            contributor_score,
            bus_factor_score,
            hotspot_score,
        )
    }

    pub fn with_stage(
        repository_stage: RepositoryStage,
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
            repository_stage,
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

impl HealthAnalyzer {
    pub fn from_analysis(
        total_commits: usize,
        contributors: &[crate::models::ContributorStats],
        bus_factor: &crate::analytics::BusFactorReport,
        hotspots: &[crate::analytics::FileHotspot],
    ) -> HealthScore {
        let repository_stage = repository_stage(total_commits);
        let mut activity_score = score_commit_activity(total_commits);
        let mut contributor_score = score_contributors(contributors.len());
        let mut bus_factor_score = score_bus_factor(bus_factor.risk_level, bus_factor.bus_factor);
        let mut hotspot_score = score_hotspots(
            hotspots
                .iter()
                .map(|hotspot| hotspot.change_count)
                .collect::<Vec<_>>()
                .as_slice(),
        );

        if repository_stage == RepositoryStage::EarlyStage {
            apply_early_stage_caps(
                &mut activity_score,
                &mut contributor_score,
                &mut bus_factor_score,
                &mut hotspot_score,
            );
        }

        HealthScore::with_stage(
            repository_stage,
            activity_score,
            contributor_score,
            bus_factor_score,
            hotspot_score,
        )
    }
}

impl Analyzer for HealthAnalyzer {
    type Output = HealthScore;

    fn analyze(&self, repo: &GitRepository) -> Result<Self::Output> {
        super::validate_repository(repo)?;

        let summary = repo.summary()?;
        let contributors = repo.contributors()?;
        let bus_factor = repo.bus_factor()?;
        let hotspots = repo.hotspots()?;

        Ok(Self::from_analysis(
            summary.total_commits,
            &contributors,
            &bus_factor,
            &hotspots,
        ))
    }
}

fn repository_stage(total_commits: usize) -> RepositoryStage {
    if total_commits < EARLY_STAGE_COMMIT_THRESHOLD {
        RepositoryStage::EarlyStage
    } else {
        RepositoryStage::Established
    }
}

fn apply_early_stage_caps(
    activity_score: &mut u8,
    contributor_score: &mut u8,
    bus_factor_score: &mut u8,
    hotspot_score: &mut u8,
) {
    *activity_score = (*activity_score).min(EARLY_STAGE_ACTIVITY_CAP);
    *contributor_score = (*contributor_score).min(EARLY_STAGE_DISTRIBUTION_CAP);
    *bus_factor_score = (*bus_factor_score).min(EARLY_STAGE_DISTRIBUTION_CAP);
    *hotspot_score = (*hotspot_score).min(EARLY_STAGE_HOTSPOT_CAP);
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
