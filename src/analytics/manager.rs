use std::path::Path;

use rayon::join;

use crate::analytics::{
    BusFactorReport, DEFAULT_TIMELINE_LIMIT, FileHotspot, HealthScore, OverviewAnalyzer,
};
use crate::git::{Analyzer, GitRepository};
use crate::models::{ContributorStats, RepositorySummary, TimelineEntry};
use crate::utils::Result;

#[derive(Debug, Clone)]
pub struct AnalysisSnapshot {
    pub summary: RepositorySummary,
    pub contributors: Vec<ContributorStats>,
    pub timeline: Vec<TimelineEntry>,
    pub hotspots: Vec<FileHotspot>,
    pub health_score: HealthScore,
    pub bus_factor: BusFactorReport,
}

#[derive(Debug, Clone)]
pub struct AnalysisManager {
    recent_commit_limit: usize,
}

impl AnalysisManager {
    pub fn new(recent_commit_limit: usize) -> Self {
        Self {
            recent_commit_limit,
        }
    }

    pub fn recent_commit_limit(&self) -> usize {
        self.recent_commit_limit
    }

    pub fn analyze(&self, repo: &GitRepository) -> Result<AnalysisSnapshot> {
        let summary = OverviewAnalyzer.analyze(repo)?;
        let repo_path = repo.repository_path();

        let (contributors_result, (timeline_result, hotspots_result)) = join(
            || load_contributors(&repo_path),
            || {
                join(
                    || load_timeline(&repo_path, self.recent_commit_limit),
                    || load_hotspots(&repo_path),
                )
            },
        );

        let contributors = contributors_result?;
        let timeline = timeline_result?;
        let hotspots = hotspots_result?;
        let health_score = repo.health_score()?;
        let bus_factor = repo.bus_factor()?;

        Ok(AnalysisSnapshot {
            summary,
            contributors,
            timeline,
            hotspots,
            health_score,
            bus_factor,
        })
    }
}

impl Default for AnalysisManager {
    fn default() -> Self {
        Self::new(DEFAULT_TIMELINE_LIMIT)
    }
}

fn load_contributors(repo_path: &Path) -> Result<Vec<ContributorStats>> {
    GitRepository::open(repo_path)?.contributors()
}

fn load_timeline(repo_path: &Path, limit: usize) -> Result<Vec<TimelineEntry>> {
    GitRepository::open(repo_path)?.recent_commits(limit)
}

fn load_hotspots(repo_path: &Path) -> Result<Vec<FileHotspot>> {
    GitRepository::open(repo_path)?.hotspots()
}
