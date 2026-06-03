//! 健康度分析测试：验证评分范围、早期仓库和健康/不健康场景。

use std::error::Error;
use std::path::Path;

use git2::{ErrorCode, Oid, Repository, Signature, Time};
use gitinsight_rs::analytics::{HealthScore, RepositoryStage};
use gitinsight_rs::git::GitRepository;
use tempfile::TempDir;

const BASE_TIME: i64 = 1_735_689_600;

#[test]
fn score_range() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;

    let score = health_score(&repo_path)?;

    assert_score_range(score.overall_score);
    assert_score_range(score.activity_score);
    assert_score_range(score.contributor_score);
    assert_score_range(score.bus_factor_score);
    assert_score_range(score.hotspot_score);

    Ok(())
}

#[test]
fn empty_repository_has_zero_health_score() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;

    let score = health_score(&repo_path)?;

    assert_eq!(score.overall_score, 0);
    assert_eq!(score.repository_stage, RepositoryStage::EarlyStage);
    assert_eq!(score.activity_score, 0);
    assert_eq!(score.contributor_score, 0);
    assert_eq!(score.bus_factor_score, 0);
    assert_eq!(score.hotspot_score, 0);

    Ok(())
}

#[test]
fn early_stage_repository_uses_conservative_scores() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;
    let repo = Repository::open(&repo_path)?;

    commit_files(
        &repo,
        &repo_path,
        &[
            ("src/main.rs", "fn main() {}\n"),
            ("src/lib.rs", "pub fn lib() {}\n"),
            ("src/app.rs", "pub fn app() {}\n"),
            ("src/git.rs", "pub fn git() {}\n"),
            ("src/ui.rs", "pub fn ui() {}\n"),
        ],
        "Tom",
        "tom@example.com",
        BASE_TIME,
    )?;

    let score = health_score(&repo_path)?;

    assert_eq!(score.repository_stage, RepositoryStage::EarlyStage);
    assert_eq!(score.activity_score, 35);
    assert_eq!(score.contributor_score, 35);
    assert_eq!(score.bus_factor_score, 35);
    assert_eq!(score.hotspot_score, 35);
    assert_eq!(score.overall_score, 35);

    Ok(())
}

#[test]
fn healthy_repository() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;
    let repo = Repository::open(&repo_path)?;

    let authors = [
        ("Alice", "alice@example.com"),
        ("Bob", "bob@example.com"),
        ("Eve", "eve@example.com"),
        ("Mia", "mia@example.com"),
        ("Tom", "tom@example.com"),
        ("Zoe", "zoe@example.com"),
    ];

    for index in 0..12 {
        let (author_name, author_email) = authors[index % authors.len()];
        commit_file(
            &repo,
            &repo_path,
            &format!("src/file-{}.rs", index % 6),
            &format!("pub const VALUE: usize = {index};\n"),
            author_name,
            author_email,
            BASE_TIME + index as i64 * 60,
        )?;
    }

    let score = health_score(&repo_path)?;

    assert_eq!(score.repository_stage, RepositoryStage::Established);
    assert_eq!(score.activity_score, 100);
    assert_eq!(score.contributor_score, 100);
    assert_eq!(score.bus_factor_score, 100);
    assert_eq!(score.hotspot_score, 100);
    assert!(score.overall_score >= 90);

    Ok(())
}

#[test]
fn unhealthy_repository() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;
    let repo = Repository::open(&repo_path)?;

    for index in 0..5 {
        commit_file(
            &repo,
            &repo_path,
            "src/main.rs",
            &format!("fn main() {{ println!(\"{index}\"); }}\n"),
            "Tom",
            "tom@example.com",
            BASE_TIME + index as i64 * 60,
        )?;
    }

    let score = health_score(&repo_path)?;

    assert_eq!(score.contributor_score, 35);
    assert_eq!(score.bus_factor_score, 35);
    assert_eq!(score.hotspot_score, 25);
    assert!(score.overall_score < 50);

    Ok(())
}

fn assert_score_range(score: u8) {
    assert!(score <= 100);
}

fn health_score(repo_path: &Path) -> Result<HealthScore, Box<dyn Error>> {
    let repository = GitRepository::open(repo_path)?;
    Ok(repository.health_score()?)
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
        "health test commit",
        &tree,
        &parent_refs,
    )?;

    Ok(commit_id)
}

fn commit_files(
    repo: &Repository,
    workdir: &Path,
    files: &[(&str, &str)],
    author_name: &str,
    author_email: &str,
    timestamp: i64,
) -> Result<Oid, Box<dyn Error>> {
    for (relative_path, contents) in files {
        let file_path = workdir.join(relative_path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(file_path, contents)?;
    }

    let mut index = repo.index()?;
    for (relative_path, _) in files {
        index.add_path(Path::new(relative_path))?;
    }
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
        "initial import",
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
