//! Git 仓库访问层：封装 git2::Repository 并提供统计与分析入口。

use std::path::{Path, PathBuf};

use git2::{BranchType, ErrorCode, Repository};

use crate::analytics::FileHotspot;
use crate::analytics::HealthScore;
use crate::analytics::RiskReport;
use crate::analytics::{BusFactorAnalyzer, BusFactorReport};
use crate::analytics::{
    ContributorAnalyzer, HealthAnalyzer, HotspotAnalyzer, RiskAnalyzer, TimelineAnalyzer,
};
use crate::git::Analyzer;
use crate::models::{ContributorStats, RepositorySummary, TimelineEntry};
use crate::utils::Result;

pub struct GitRepository {
    repo: Repository,
}

impl GitRepository {
    pub fn open(path: &Path) -> Result<Self> {
        Ok(Self {
            repo: Repository::open(path)?,
        })
    }

    pub fn open_current_dir() -> Result<Self> {
        let current_dir = std::env::current_dir()?;
        Self::open(&current_dir)
    }

    pub fn repository_name(&self) -> Result<String> {
        let path = self.repository_path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.trim().is_empty())
            .map_or_else(|| "repository".to_owned(), ToOwned::to_owned);

        Ok(name)
    }

    pub fn commit_count(&self) -> Result<usize> {
        if self.repo.is_empty()? {
            return Ok(0);
        }

        let mut revwalk = self.repo.revwalk()?;
        if !push_head_if_present(&mut revwalk)? {
            return Ok(0);
        }

        let mut count = 0;
        for oid in revwalk {
            oid?;
            count += 1;
        }

        Ok(count)
    }

    pub fn branch_count(&self) -> Result<usize> {
        let mut count = 0;

        for branch in self.repo.branches(Some(BranchType::Local))? {
            branch?;
            count += 1;
        }

        Ok(count)
    }

    pub fn tag_count(&self) -> Result<usize> {
        Ok(self.repo.tag_names(None)?.len())
    }

    pub fn contributor_count(&self) -> Result<usize> {
        Ok(self.contributors()?.len())
    }

    pub fn contributors(&self) -> Result<Vec<ContributorStats>> {
        ContributorAnalyzer.analyze(self)
    }

    pub fn recent_commits(&self, limit: usize) -> Result<Vec<TimelineEntry>> {
        TimelineAnalyzer::new(limit).analyze(self)
    }

    pub fn hotspots(&self) -> Result<Vec<FileHotspot>> {
        HotspotAnalyzer.analyze(self)
    }

    pub fn bus_factor(&self) -> Result<BusFactorReport> {
        BusFactorAnalyzer.analyze(self)
    }

    pub fn health_score(&self) -> Result<HealthScore> {
        HealthAnalyzer.analyze(self)
    }

    pub fn risk_report(&self) -> Result<RiskReport> {
        RiskAnalyzer.analyze(self)
    }

    pub fn summary(&self) -> Result<RepositorySummary> {
        let mut summary = RepositorySummary::new(self.repository_name()?, self.repository_path());

        summary.total_commits = self.commit_count()?;
        summary.total_branches = self.branch_count()?;
        summary.total_tags = self.tag_count()?;
        summary.total_contributors = self.contributor_count()?;

        Ok(summary)
    }

    pub(crate) fn repository_path(&self) -> PathBuf {
        if let Some(workdir) = self.repo.workdir() {
            return workdir.to_path_buf();
        }

        if let Some(parent) = self.repo.path().parent() {
            return parent.to_path_buf();
        }

        self.repo.path().to_path_buf()
    }

    pub(crate) fn raw_repository(&self) -> &Repository {
        &self.repo
    }
}

pub(crate) fn push_head_if_present(revwalk: &mut git2::Revwalk<'_>) -> Result<bool> {
    match revwalk.push_head() {
        Ok(()) => Ok(true),
        Err(error) if is_empty_history_error(&error) => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn is_empty_history_error(error: &git2::Error) -> bool {
    matches!(error.code(), ErrorCode::UnbornBranch | ErrorCode::NotFound)
}
