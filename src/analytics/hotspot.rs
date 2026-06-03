//! 文件热点分析：结合修改次数和最近修改时间计算文件热度分数。

use std::collections::HashMap;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::git::repository::push_head_if_present;
use crate::git::{Analyzer, GitRepository};
use crate::utils::Result;
use crate::utils::time::unix_seconds_to_utc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileHotspot {
    pub path: String,
    pub change_count: usize,
    pub last_modified: DateTime<Utc>,
    pub score: f64,
}

impl FileHotspot {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            change_count: 0,
            last_modified: DateTime::<Utc>::UNIX_EPOCH,
            score: 0.0,
        }
    }

    pub fn record_change(&mut self, commit_time: DateTime<Utc>) {
        self.change_count = self.change_count.saturating_add(1);
        self.last_modified = self.last_modified.max(commit_time);
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
        assign_hotspot_scores(&mut hotspots);
        hotspots.sort_by(|left, right| {
            right
                .score
                .total_cmp(&left.score)
                .then_with(|| right.change_count.cmp(&left.change_count))
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

fn assign_hotspot_scores(hotspots: &mut [FileHotspot]) {
    let max_change_count = hotspots
        .iter()
        .map(|hotspot| hotspot.change_count)
        .max()
        .unwrap_or(0);
    let oldest_timestamp = hotspots
        .iter()
        .map(|hotspot| hotspot.last_modified.timestamp())
        .min()
        .unwrap_or(0);
    let newest_timestamp = hotspots
        .iter()
        .map(|hotspot| hotspot.last_modified.timestamp())
        .max()
        .unwrap_or(0);

    for hotspot in hotspots {
        let change_score = normalize_change_count(hotspot.change_count, max_change_count);
        let recency_score = normalize_recency(
            hotspot.last_modified.timestamp(),
            oldest_timestamp,
            newest_timestamp,
        );
        hotspot.score = 0.6 * change_score + 0.4 * recency_score;
    }
}

fn normalize_change_count(change_count: usize, max_change_count: usize) -> f64 {
    if max_change_count == 0 {
        0.0
    } else {
        change_count as f64 / max_change_count as f64
    }
}

fn normalize_recency(timestamp: i64, oldest_timestamp: i64, newest_timestamp: i64) -> f64 {
    let range = newest_timestamp.saturating_sub(oldest_timestamp);
    if range == 0 {
        1.0
    } else {
        timestamp.saturating_sub(oldest_timestamp) as f64 / range as f64
    }
}
