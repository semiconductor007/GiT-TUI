//! 贡献者分析：按邮箱聚合提交数量、活跃天数和 Ownership 百分比。

use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;

use crate::git::repository::push_head_if_present;
use crate::git::{Analyzer, GitRepository};
use crate::models::ContributorStats;
use crate::utils::Result;
use crate::utils::time::unix_seconds_to_utc;

#[derive(Debug, Clone, Default)]
pub struct ContributorAnalyzer;

#[derive(Debug, Clone, Default)]
struct ContributorAccumulator {
    stats: ContributorStats,
    active_dates: HashSet<NaiveDate>,
}

impl ContributorAccumulator {
    fn new(name: String, email: String) -> Self {
        Self {
            stats: ContributorStats::new(name, email),
            active_dates: HashSet::new(),
        }
    }

    fn record_commit(&mut self, timestamp: chrono::DateTime<chrono::Utc>) {
        self.stats.record_commit(timestamp, 0, 0);
        self.active_dates.insert(timestamp.date_naive());
        self.stats.active_days = self.active_dates.len();
    }

    fn into_stats(mut self) -> ContributorStats {
        self.stats.active_days = self.active_dates.len();
        self.stats
    }
}

impl Analyzer for ContributorAnalyzer {
    type Output = Vec<ContributorStats>;

    fn analyze(&self, repo: &GitRepository) -> Result<Self::Output> {
        super::validate_repository(repo)?;

        let raw_repo = repo.raw_repository();
        if raw_repo.is_empty()? {
            return Ok(Vec::new());
        }

        let mut revwalk = raw_repo.revwalk()?;
        if !push_head_if_present(&mut revwalk)? {
            return Ok(Vec::new());
        }

        let mut contributors = HashMap::<String, ContributorAccumulator>::new();
        for oid in revwalk {
            let commit = raw_repo.find_commit(oid?)?;
            let author = commit.author();
            let email = author.email()?.trim();

            if email.is_empty() {
                continue;
            }

            let name = author.name()?.trim();
            let timestamp = unix_seconds_to_utc(commit.time().seconds())?;
            contributors
                .entry(email.to_owned())
                .or_insert_with(|| ContributorAccumulator::new(name.to_owned(), email.to_owned()))
                .record_commit(timestamp);
        }

        let mut stats = contributors
            .into_values()
            .map(ContributorAccumulator::into_stats)
            .collect::<Vec<_>>();
        let total_commits = stats
            .iter()
            .map(|contributor| contributor.commit_count)
            .sum::<usize>();
        for contributor in &mut stats {
            contributor.set_ownership(total_commits);
        }
        stats.sort_by(|left, right| {
            right
                .commit_count
                .cmp(&left.commit_count)
                .then_with(|| left.email.cmp(&right.email))
        });

        Ok(stats)
    }
}
