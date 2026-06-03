//! Bus Factor 测试：验证关键贡献者数量和风险等级判断。

use std::error::Error;
use std::path::Path;

use git2::{ErrorCode, Oid, Repository, Signature, Time};
use gitinsight_rs::analytics::RiskLevel;
use gitinsight_rs::git::GitRepository;
use tempfile::TempDir;

const BASE_TIME: i64 = 1_735_689_600;

#[test]
fn single_dominant_author() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;
    let repo = Repository::open(&repo_path)?;

    create_commits_for_author(&repo, &repo_path, "Tom", "tom@example.com", 6)?;
    create_commits_for_author(&repo, &repo_path, "Alice", "alice@example.com", 1)?;
    create_commits_for_author(&repo, &repo_path, "Bob", "bob@example.com", 1)?;

    let report = GitRepository::open(&repo_path)?.bus_factor()?;

    assert_eq!(report.bus_factor, 1);
    assert_eq!(report.top_contributors, vec!["Tom"]);
    assert_eq!(report.risk_level, RiskLevel::High);

    Ok(())
}

#[test]
fn empty_repository_has_zero_bus_factor() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;

    let report = GitRepository::open(&repo_path)?.bus_factor()?;

    assert_eq!(report.bus_factor, 0);
    assert!(report.top_contributors.is_empty());
    assert_eq!(report.risk_level, RiskLevel::High);

    Ok(())
}

#[test]
fn balanced_contributors() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;
    let repo = Repository::open(&repo_path)?;

    create_commits_for_author(&repo, &repo_path, "Alice", "alice@example.com", 2)?;
    create_commits_for_author(&repo, &repo_path, "Bob", "bob@example.com", 2)?;
    create_commits_for_author(&repo, &repo_path, "Tom", "tom@example.com", 2)?;
    create_commits_for_author(&repo, &repo_path, "Zoe", "zoe@example.com", 2)?;
    create_commits_for_author(&repo, &repo_path, "Mia", "mia@example.com", 2)?;
    create_commits_for_author(&repo, &repo_path, "Eve", "eve@example.com", 2)?;

    let report = GitRepository::open(&repo_path)?.bus_factor()?;

    assert_eq!(report.bus_factor, 3);
    assert_eq!(report.risk_level, RiskLevel::Low);
    assert_eq!(report.top_contributors.len(), 3);

    Ok(())
}

#[test]
fn risk_level_detection() -> Result<(), Box<dyn Error>> {
    let high_risk = repository_with_author_counts(&[("Tom", "tom@example.com", 3)])?;
    let medium_risk = repository_with_author_counts(&[
        ("Tom", "tom@example.com", 2),
        ("Alice", "alice@example.com", 2),
        ("Bob", "bob@example.com", 1),
    ])?;
    let low_risk = repository_with_author_counts(&[
        ("Alice", "alice@example.com", 2),
        ("Bob", "bob@example.com", 2),
        ("Tom", "tom@example.com", 2),
        ("Zoe", "zoe@example.com", 2),
        ("Mia", "mia@example.com", 2),
        ("Eve", "eve@example.com", 2),
    ])?;

    assert_eq!(high_risk.bus_factor, 1);
    assert_eq!(high_risk.risk_level, RiskLevel::High);
    assert_eq!(medium_risk.bus_factor, 2);
    assert_eq!(medium_risk.risk_level, RiskLevel::Medium);
    assert_eq!(low_risk.bus_factor, 3);
    assert_eq!(low_risk.risk_level, RiskLevel::Low);

    Ok(())
}

fn repository_with_author_counts(
    author_counts: &[(&str, &str, usize)],
) -> Result<gitinsight_rs::analytics::BusFactorReport, Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;
    let repo = Repository::open(&repo_path)?;

    for (name, email, count) in author_counts {
        create_commits_for_author(&repo, &repo_path, name, email, *count)?;
    }

    Ok(GitRepository::open(&repo_path)?.bus_factor()?)
}

fn init_temp_repository() -> Result<(TempDir, std::path::PathBuf), Box<dyn Error>> {
    let temp_dir = tempfile::tempdir()?;
    Repository::init(temp_dir.path())?;
    let repo_path = temp_dir.path().to_path_buf();

    Ok((temp_dir, repo_path))
}

fn create_commits_for_author(
    repo: &Repository,
    repo_path: &Path,
    author_name: &str,
    author_email: &str,
    count: usize,
) -> Result<(), Box<dyn Error>> {
    for index in 0..count {
        let safe_email = author_email.replace(['@', '.'], "_");
        commit_file(
            repo,
            repo_path,
            &format!("{safe_email}-{index}.txt"),
            &format!("{author_name} commit {index}"),
            author_name,
            author_email,
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
    author_name: &str,
    author_email: &str,
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
    let signature = Signature::new(author_name, author_email, &time)?;
    let parents = current_head_commit(repo)?;
    let parent_refs = parents.iter().collect::<Vec<_>>();

    let commit_id = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "bus factor test commit",
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
