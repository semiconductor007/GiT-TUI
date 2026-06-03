use crate::models::CommitInfo;

pub fn sort_by_time_desc(commits: &mut [CommitInfo]) {
    commits.sort_by(|left, right| right.timestamp.cmp(&left.timestamp));
}
