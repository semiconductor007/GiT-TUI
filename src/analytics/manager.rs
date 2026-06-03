//! 分析调度器：并行加载核心分析数据并组装 AnalysisSnapshot。

use std::path::Path;

use rayon::join;

use crate::analytics::{
    BusFactorAnalyzer, BusFactorReport, DEFAULT_TIMELINE_LIMIT, FileHotspot, HealthAnalyzer,
    HealthScore, OverviewAnalyzer, RiskAnalyzer, RiskReport,
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
    pub risk_report: RiskReport,
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
        let repo_path = repo.repository_path();

        // 使用 Rayon 并行加载互不依赖的数据集；每个任务重新打开仓库，避免跨线程共享 git2::Repository。
        let (summary_result, (contributors_result, (timeline_result, hotspots_result))) = join(
            || load_summary(&repo_path),
            || {
                join(
                    || load_contributors(&repo_path),
                    || {
                        join(
                            || load_timeline(&repo_path, self.recent_commit_limit),
                            || load_hotspots(&repo_path),
                        )
                    },
                )
            },
        );

        let summary = summary_result?;
        let contributors = contributors_result?;
        let timeline = timeline_result?;
        let hotspots = hotspots_result?;
        let bus_factor = BusFactorAnalyzer::from_contributors(&contributors);
        let health_score = HealthAnalyzer::from_analysis(
            summary.total_commits,
            &contributors,
            &bus_factor,
            &hotspots,
        );
        let risk_report =
            RiskAnalyzer::from_analysis(&contributors, &bus_factor, &health_score, &hotspots);

        Ok(AnalysisSnapshot {
            summary,
            contributors,
            timeline,
            hotspots,
            health_score,
            bus_factor,
            risk_report,
        })
    }
}

impl Default for AnalysisManager {
    fn default() -> Self {
        Self::new(DEFAULT_TIMELINE_LIMIT)
    }
}

fn load_summary(repo_path: &Path) -> Result<RepositorySummary> {
    OverviewAnalyzer.analyze(&GitRepository::open(repo_path)?)
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
