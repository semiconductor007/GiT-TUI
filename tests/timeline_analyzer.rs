use std::error::Error;
use std::path::Path;

use git2::{ErrorCode, Oid, Repository, Signature, Time};
use gitinsight_rs::git::GitRepository;
use gitinsight_rs::models::TimelineEntry;
use tempfile::TempDir;

const BASE_TIME: i64 = 1_735_689_600;

#[test]
fn recent_commit_count() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;
    let repo = Repository::open(&repo_path)?;

    create_commits(&repo, &repo_path, 4)?;

    let entries = recent_commits(&repo_path, 50)?;

    assert_eq!(entries.len(), 4);

    Ok(())
}

#[test]
fn ordering_places_newest_commit_first() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;
    let repo = Repository::open(&repo_path)?;

    commit_file(&repo, &repo_path, "old.txt", "old", "old commit", BASE_TIME)?;
    commit_file(
        &repo,
        &repo_path,
        "new.txt",
        "new",
        "new commit",
        BASE_TIME + 600,
    )?;

    let entries = recent_commits(&repo_path, 50)?;

    assert_eq!(entries[0].message, "new commit");
    assert_eq!(entries[1].message, "old commit");
    assert!(entries[0].commit_time > entries[1].commit_time);

    Ok(())
}

#[test]
fn limit_respected() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;
    let repo = Repository::open(&repo_path)?;

    create_commits(&repo, &repo_path, 10)?;

    let entries = recent_commits(&repo_path, 3)?;

    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].message, "commit 9");
    assert_eq!(entries[2].message, "commit 7");

    Ok(())
}

#[test]
fn empty_message_uses_placeholder() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;
    let repo = Repository::open(&repo_path)?;

    commit_file(&repo, &repo_path, "empty.txt", "empty", "", BASE_TIME)?;

    let entries = recent_commits(&repo_path, 50)?;

    assert_eq!(entries[0].message, "<no message>");

    Ok(())
}

#[test]
fn short_commit_id_has_eight_characters() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;
    let repo = Repository::open(&repo_path)?;

    commit_file(&repo, &repo_path, "id.txt", "id", "check id", BASE_TIME)?;

    let entries = recent_commits(&repo_path, 50)?;

    assert_eq!(entries[0].commit_id.len(), 8);

    Ok(())
}

fn init_temp_repository() -> Result<(TempDir, std::path::PathBuf), Box<dyn Error>> {
    let temp_dir = tempfile::tempdir()?;
    Repository::init(temp_dir.path())?;
    let repo_path = temp_dir.path().to_path_buf();

    Ok((temp_dir, repo_path))
}

fn recent_commits(repo_path: &Path, limit: usize) -> Result<Vec<TimelineEntry>, Box<dyn Error>> {
    let repository = GitRepository::open(repo_path)?;
    Ok(repository.recent_commits(limit)?)
}

fn create_commits(repo: &Repository, repo_path: &Path, count: usize) -> Result<(), Box<dyn Error>> {
    for index in 0..count {
        commit_file(
            repo,
            repo_path,
            &format!("file-{index}.txt"),
            &format!("content {index}"),
            &format!("commit {index}"),
            BASE_TIME + index as i64 * 60,
        )?;
    }

    Ok(())
}

fn commit_file(
    repo: &Repository,
    workdir: &Path,
    relative_path: &str,
    contents: &str,
    message: &str,
    timestamp: i64,
) -> Result<Oid, Box<dyn Error>> {
    let file_path = workdir.join(relative_path);
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(file_path, contents)?;

    let mut index = repo.index()?;
    index.add_path(Path::new(relative_path))?;
    index.write()?;
    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;
    let time = Time::new(timestamp, 0);
    let signature = Signature::new("Timeline Tester", "timeline@example.com", &time)?;
    let parents = current_head_commit(repo)?;
    let parent_refs = parents.iter().collect::<Vec<_>>();

    let commit_id = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &parent_refs,
    )?;

    Ok(commit_id)
}

fn current_head_commit(repo: &Repository) -> Result<Vec<git2::Commit<'_>>, Box<dyn Error>> {
    match repo.head() {
        Ok(head) => {
            if let Some(oid) = head.target() {
                Ok(vec![repo.find_commit(oid)?])
            } else {
                Ok(Vec::new())
            }
        }
        Err(error) if matches!(error.code(), ErrorCode::UnbornBranch | ErrorCode::NotFound) => {
            Ok(Vec::new())
        }
        Err(error) => Err(error.into()),
    }
}
