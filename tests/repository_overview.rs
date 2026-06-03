use std::error::Error;
use std::path::Path;

use git2::{ErrorCode, Oid, Repository, Signature};
use gitinsight_rs::git::GitRepository;
use tempfile::TempDir;

#[test]
fn repository_open() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;

    let repository = GitRepository::open(&repo_path)?;

    assert!(!repository.repository_name()?.trim().is_empty());

    Ok(())
}

#[test]
fn empty_repository_summary_has_zero_history_counts() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;

    let repository = GitRepository::open(&repo_path)?;
    let summary = repository.summary()?;

    assert!(!summary.name.trim().is_empty());
    assert_eq!(summary.total_commits, 0);
    assert_eq!(summary.total_branches, 0);
    assert_eq!(summary.total_tags, 0);
    assert_eq!(summary.total_contributors, 0);

    Ok(())
}

#[test]
fn summary_generation() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;
    let repo = Repository::open(&repo_path)?;

    commit_file(
        &repo,
        &repo_path,
        "README.md",
        "# Demo\n",
        "Alice",
        "alice@example.com",
        "add readme",
    )?;
    let second_commit = commit_file(
        &repo,
        &repo_path,
        "src/lib.rs",
        "pub fn demo() {}\n",
        "Bob",
        "bob@example.com",
        "add library",
    )?;

    let target = repo.find_object(second_commit, None)?;
    let tagger = Signature::now("Tagger", "tagger@example.com")?;
    repo.tag("v0.1.0", &target, &tagger, "first test tag", false)?;

    let repository = GitRepository::open(&repo_path)?;
    let summary = repository.summary()?;

    assert!(!summary.name.trim().is_empty());
    assert_eq!(summary.total_commits, 2);
    assert_eq!(summary.total_branches, 1);
    assert_eq!(summary.total_tags, 1);
    assert_eq!(summary.total_contributors, 2);
    assert_eq!(summary.total_files, 0);
    assert_eq!(summary.total_loc, 0);

    Ok(())
}

fn init_temp_repository() -> Result<(TempDir, std::path::PathBuf), Box<dyn Error>> {
    let temp_dir = tempfile::tempdir()?;
    Repository::init(temp_dir.path())?;
    let repo_path = temp_dir.path().to_path_buf();

    Ok((temp_dir, repo_path))
}

fn commit_file(
    repo: &Repository,
    workdir: &Path,
    relative_path: &str,
    contents: &str,
    author_name: &str,
    author_email: &str,
    message: &str,
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
    let signature = Signature::now(author_name, author_email)?;
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
