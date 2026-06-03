//! 提交模型：描述单次提交及其文件变更和代码行数统计。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileChange {
    pub path: String,
    pub change_kind: ChangeKind,
    pub lines_added: usize,
    pub lines_deleted: usize,
}

impl FileChange {
    pub fn new(
        path: impl Into<String>,
        change_kind: ChangeKind,
        lines_added: usize,
        lines_deleted: usize,
    ) -> Self {
        Self {
            path: path.into(),
            change_kind,
            lines_added,
            lines_deleted,
        }
    }

    pub fn changed_lines(&self) -> usize {
        self.lines_added + self.lines_deleted
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitInfo {
    pub id: String,
    pub short_id: String,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub changes: Vec<FileChange>,
    pub lines_added: usize,
    pub lines_deleted: usize,
}

impl CommitInfo {
    pub fn new(
        id: impl Into<String>,
        author_name: impl Into<String>,
        author_email: impl Into<String>,
        timestamp: DateTime<Utc>,
        message: impl Into<String>,
    ) -> Self {
        let id = id.into();
        let short_id = id.chars().take(8).collect();

        Self {
            id,
            short_id,
            author_name: author_name.into(),
            author_email: author_email.into(),
            timestamp,
            message: message.into(),
            changes: Vec::new(),
            lines_added: 0,
            lines_deleted: 0,
        }
    }

    pub fn with_changes(mut self, changes: Vec<FileChange>) -> Self {
        self.lines_added = changes.iter().map(|change| change.lines_added).sum();
        self.lines_deleted = changes.iter().map(|change| change.lines_deleted).sum();
        self.changes = changes;
        self
    }

    pub fn changed_lines(&self) -> usize {
        self.lines_added + self.lines_deleted
    }
}
