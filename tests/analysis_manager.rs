//! 集成测试：验证 AnalysisManager 能正确聚合并发分析结果。

use std::error::Error;
use std::path::Path;

use git2::{ErrorCode, Oid, Repository, Signature, Time};
use gitinsight_rs::analytics::AnalysisManager;
use gitinsight_rs::git::GitRepository;
use tempfile::TempDir;

const BASE_TIME: i64 = 1_735_689_600;

#[test]
fn analysis_manager_loads_core_datasets() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;
    let repo = Repository::open(&repo_path)?;

    commit_file(
        &repo,
        &repo_path,
        "src/main.rs",
        "fn main() {}\n",
        "Alice",
        "alice@example.com",
        BASE_TIME,
    )?;
    commit_file(
        &repo,
        &repo_path,
        "src/lib.rs",
        "pub fn lib() {}\n",
        "Bob",
        "bob@example.com",
        BASE_TIME + 60,
    )?;
    commit_file(
        &repo,
        &repo_path,
        "src/main.rs",
        "fn main() { println!(\"hi\"); }\n",
        "Alice",
        "alice@example.com",
        BASE_TIME + 120,
    )?;

    let repository = GitRepository::open(&repo_path)?;
    let snapshot = AnalysisManager::new(50).analyze(&repository)?;

    assert_eq!(snapshot.summary.total_commits, 3);
    assert_eq!(snapshot.contributors.len(), 2);
    assert_eq!(snapshot.timeline.len(), 3);
    assert_eq!(snapshot.hotspots[0].path, "src/main.rs");
    assert!(snapshot.health_score.overall_score <= 100);
    assert!(snapshot.bus_factor.bus_factor >= 1);

    Ok(())
}

#[test]
fn analysis_manager_respects_timeline_limit() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;
    let repo = Repository::open(&repo_path)?;

    for index in 0..5 {
        commit_file(
            &repo,
            &repo_path,
            &format!("file-{index}.txt"),
            &format!("content {index}\n"),
            "Alice",
            "alice@example.com",
            BASE_TIME + index as i64 * 60,
        )?;
    }

    let repository = GitRepository::open(&repo_path)?;
    let snapshot = AnalysisManager::new(2).analyze(&repository)?;

    assert_eq!(snapshot.summary.total_commits, 5);
    assert_eq!(snapshot.timeline.len(), 2);

    Ok(())
}

#[test]
fn analysis_manager_matches_direct_repository_analysis() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;
    let repo = Repository::open(&repo_path)?;

    for index in 0..6 {
        let (author_name, author_email) = if index % 2 == 0 {
            ("Alice", "alice@example.com")
        } else {
            ("Bob", "bob@example.com")
        };
        commit_file(
            &repo,
            &repo_path,
            &format!("src/file-{}.rs", index % 3),
            &format!("pub const VALUE: usize = {index};\n"),
            author_name,
            author_email,
            BASE_TIME + index as i64 * 60,
        )?;
    }

    let repository = GitRepository::open(&repo_path)?;
    let snapshot = AnalysisManager::default().analyze(&repository)?;

    assert_eq!(snapshot.summary, repository.summary()?);
    assert_eq!(snapshot.contributors, repository.contributors()?);
    assert_eq!(snapshot.bus_factor, repository.bus_factor()?);
    assert_eq!(snapshot.health_score, repository.health_score()?);
    assert_eq!(snapshot.risk_report, repository.risk_report()?);

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
        "analysis manager test commit",
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
