use std::collections::HashMap;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::git::repository::push_head_if_present;
use crate::git::{Analyzer, GitRepository};
use crate::utils::Result;
use crate::utils::time::unix_seconds_to_utc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileHotspot {
    pub path: String,
    pub change_count: usize,
    pub last_modified: Option<DateTime<Utc>>,
}

impl FileHotspot {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            change_count: 0,
            last_modified: None,
        }
    }

    pub fn record_change(&mut self, commit_time: DateTime<Utc>) {
        self.change_count = self.change_count.saturating_add(1);
        self.last_modified = Some(match self.last_modified {
            Some(last_modified) => last_modified.max(commit_time),
            None => commit_time,
        });
    }
}

#[derive(Debug, Clone, Default)]
pub struct HotspotAnalyzer;

impl Analyzer for HotspotAnalyzer {
    type Output = Vec<FileHotspot>;

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

        let mut hotspots = HashMap::<String, FileHotspot>::new();
        for oid in revwalk {
            let commit = raw_repo.find_commit(oid?)?;
            let commit_tree = commit.tree()?;
            let commit_time = unix_seconds_to_utc(commit.time().seconds())?;

            if commit.parent_count() == 0 {
                let diff = raw_repo.diff_tree_to_tree(None, Some(&commit_tree), None)?;
                record_diff_paths(&diff, commit_time, &mut hotspots);
                continue;
            }

            for parent in commit.parents() {
                let parent_tree = parent.tree()?;
                let diff =
                    raw_repo.diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), None)?;
                record_diff_paths(&diff, commit_time, &mut hotspots);
            }
        }

        let mut hotspots = hotspots.into_values().collect::<Vec<_>>();
        hotspots.sort_by(|left, right| {
            right
                .change_count
                .cmp(&left.change_count)
                .then_with(|| left.path.cmp(&right.path))
        });

        Ok(hotspots)
    }
}

fn record_diff_paths(
    diff: &git2::Diff<'_>,
    commit_time: DateTime<Utc>,
    hotspots: &mut HashMap<String, FileHotspot>,
) {
    for delta in diff.deltas() {
        if let Some(path) = changed_path(&delta) {
            let path = normalize_path(path);
            hotspots
                .entry(path.clone())
                .or_insert_with(|| FileHotspot::new(path))
                .record_change(commit_time);
        }
    }
}

fn changed_path<'a>(delta: &'a git2::DiffDelta<'a>) -> Option<&'a Path> {
    delta.new_file().path().or_else(|| delta.old_file().path())
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
