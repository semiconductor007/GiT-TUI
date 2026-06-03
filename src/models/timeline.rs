//! 时间线模型：保存 TUI 时间线页面展示的单条提交记录。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimelineEntry {
    pub commit_id: String,
    pub author_name: String,
    pub author_email: String,
    pub message: String,
    pub commit_time: DateTime<Utc>,
}

impl TimelineEntry {
    pub fn new(
        commit_id: impl Into<String>,
        author_name: impl Into<String>,
        author_email: impl Into<String>,
        message: impl Into<String>,
        commit_time: DateTime<Utc>,
    ) -> Self {
        Self {
            commit_id: commit_id.into(),
            author_name: author_name.into(),
            author_email: author_email.into(),
            message: message.into(),
            commit_time,
        }
    }
}
