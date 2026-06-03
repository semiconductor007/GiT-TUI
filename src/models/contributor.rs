//! 贡献者模型：记录作者提交数、活跃天数、时间范围和 Ownership。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ContributorStats {
    pub name: String,
    pub email: String,
    pub commit_count: usize,
    pub active_days: usize,
    pub first_commit: Option<DateTime<Utc>>,
    pub last_commit: Option<DateTime<Utc>>,
    pub lines_added: usize,
    pub lines_deleted: usize,
    pub ownership_percent: f64,
}

impl ContributorStats {
    pub fn new(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            email: email.into(),
            ..Self::default()
        }
    }

    pub fn record_commit(
        &mut self,
        timestamp: DateTime<Utc>,
        lines_added: usize,
        lines_deleted: usize,
    ) {
        self.commit_count = self.commit_count.saturating_add(1);
        self.lines_added = self.lines_added.saturating_add(lines_added);
        self.lines_deleted = self.lines_deleted.saturating_add(lines_deleted);
        self.update_commit_range(timestamp);
    }

    pub fn changed_lines(&self) -> usize {
        self.lines_added + self.lines_deleted
    }

    pub fn set_ownership(&mut self, total_commits: usize) {
        self.ownership_percent = if total_commits == 0 {
            0.0
        } else {
            self.commit_count as f64 / total_commits as f64 * 100.0
        };
    }

    fn update_commit_range(&mut self, timestamp: DateTime<Utc>) {
        self.first_commit = Some(match self.first_commit {
            Some(first_commit) => first_commit.min(timestamp),
            None => timestamp,
        });
        self.last_commit = Some(match self.last_commit {
            Some(last_commit) => last_commit.max(timestamp),
            None => timestamp,
        });
    }
}
