//! 时间线分析：提取最近提交记录、短提交 ID、作者、时间和提交信息。

use crate::git::repository::push_head_if_present;
use crate::git::{Analyzer, GitRepository};
use crate::models::TimelineEntry;
use crate::utils::Result;
use crate::utils::time::unix_seconds_to_utc;

pub const DEFAULT_TIMELINE_LIMIT: usize = 50;
const EMPTY_MESSAGE: &str = "<无提交信息>";
const SHORT_COMMIT_ID_LEN: usize = 8;

#[derive(Debug, Clone)]
pub struct TimelineAnalyzer {
    limit: usize,
}

impl TimelineAnalyzer {
    pub fn new(limit: usize) -> Self {
        Self { limit }
    }

    pub fn limit(&self) -> usize {
        self.limit
    }
}

impl Default for TimelineAnalyzer {
    fn default() -> Self {
        Self::new(DEFAULT_TIMELINE_LIMIT)
    }
}

impl Analyzer for TimelineAnalyzer {
    type Output = Vec<TimelineEntry>;

    fn analyze(&self, repo: &GitRepository) -> Result<Self::Output> {
        super::validate_repository(repo)?;

        if self.limit == 0 {
            return Ok(Vec::new());
        }

        let raw_repo = repo.raw_repository();
        if raw_repo.is_empty()? {
            return Ok(Vec::new());
        }

        let mut revwalk = raw_repo.revwalk()?;
        if !push_head_if_present(&mut revwalk)? {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();
        for oid in revwalk {
            let commit = raw_repo.find_commit(oid?)?;
            let author = commit.author();
            let commit_time = unix_seconds_to_utc(commit.time().seconds())?;
            let message = normalize_message(commit.message()?);

            entries.push(TimelineEntry::new(
                short_commit_id(&commit.id().to_string()),
                author.name()?.trim(),
                author.email()?.trim(),
                message,
                commit_time,
            ));
        }

        entries.sort_by(|left, right| {
            right
                .commit_time
                .cmp(&left.commit_time)
                .then_with(|| left.commit_id.cmp(&right.commit_id))
        });
        entries.truncate(self.limit);

        Ok(entries)
    }
}

fn short_commit_id(full_id: &str) -> String {
    full_id.chars().take(SHORT_COMMIT_ID_LEN).collect()
}

fn normalize_message(message: &str) -> String {
    let message = message.trim();

    if !message.is_empty() {
        message.to_owned()
    } else {
        EMPTY_MESSAGE.to_owned()
    }
}
