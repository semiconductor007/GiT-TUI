//! 提交辅助函数：提供提交数据排序等通用工具。

use crate::models::CommitInfo;

pub fn sort_by_time_desc(commits: &mut [CommitInfo]) {
    commits.sort_by(|left, right| right.timestamp.cmp(&left.timestamp));
}
