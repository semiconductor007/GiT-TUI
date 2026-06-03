//! 贡献者分析测试：验证提交统计、活跃天数、Ownership 和排序。

use std::error::Error;
use std::path::Path;

use git2::{ErrorCode, Oid, Repository, Signature, Time};
use gitinsight_rs::git::GitRepository;
use gitinsight_rs::models::ContributorStats;
use tempfile::TempDir;

const DAY_ONE: i64 = 1_735_689_600;
const DAY_THREE: i64 = 1_735_862_400;

#[test]
fn contributor_counting() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;
    let repo = Repository::open(&repo_path)?;

    commit_file(
        &repo,
        &repo_path,
        "a.txt",
        "a",
        "Tom",
        "tom@example.com",
        DAY_ONE,
    )?;
    commit_file(
        &repo,
        &repo_path,
        "b.txt",
        "b",
        "Tom",
        "tom@example.com",
        DAY_ONE + 60,
    )?;
    commit_file(
        &repo,
        &repo_path,
        "c.txt",
        "c",
        "Alice",
        "alice@example.com",
        DAY_ONE + 120,
    )?;

    let contributors = analyze_contributors(&repo_path)?;
    let tom = contributor_by_email(&contributors, "tom@example.com")?;
    let alice = contributor_by_email(&contributors, "alice@example.com")?;

    assert_eq!(tom.name, "Tom");
    assert_eq!(tom.commit_count, 2);
    assert_eq!(alice.name, "Alice");
    assert_eq!(alice.commit_count, 1);
    assert!((tom.ownership_percent - 66.666).abs() < 0.01);
    assert!((alice.ownership_percent - 33.333).abs() < 0.01);
    assert_eq!(tom.lines_added, 0);
    assert_eq!(tom.lines_deleted, 0);

    Ok(())
}

#[test]
fn empty_repository_has_no_contributors() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;

    let contributors = analyze_contributors(&repo_path)?;

    assert!(contributors.is_empty());

    Ok(())
}

#[test]
fn same_email_is_grouped_as_one_contributor() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;
    let repo = Repository::open(&repo_path)?;

    commit_file(
        &repo,
        &repo_path,
        "a.txt",
        "a",
        "Tom",
        "tom@example.com",
        DAY_ONE,
    )?;
    commit_file(
        &repo,
        &repo_path,
        "b.txt",
        "b",
        "Thomas",
        "tom@example.com",
        DAY_ONE + 60,
    )?;

    let contributors = analyze_contributors(&repo_path)?;

    assert_eq!(contributors.len(), 1);
    assert_eq!(contributors[0].email, "tom@example.com");
    assert_eq!(contributors[0].commit_count, 2);
    assert!((contributors[0].ownership_percent - 100.0).abs() < f64::EPSILON);

    Ok(())
}

#[test]
fn active_days_counts_unique_dates() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;
    let repo = Repository::open(&repo_path)?;

    commit_file(
        &repo,
        &repo_path,
        "a.txt",
        "a",
        "Tom",
        "tom@example.com",
        DAY_ONE,
    )?;
    commit_file(
        &repo,
        &repo_path,
        "b.txt",
        "b",
        "Tom",
        "tom@example.com",
        DAY_ONE + 60,
    )?;
    commit_file(
        &repo,
        &repo_path,
        "c.txt",
        "c",
        "Tom",
        "tom@example.com",
        DAY_THREE,
    )?;

    let contributors = analyze_contributors(&repo_path)?;
    let tom = contributor_by_email(&contributors, "tom@example.com")?;

    assert_eq!(tom.commit_count, 3);
    assert_eq!(tom.active_days, 2);

    Ok(())
}

#[test]
fn ownership_percentages_sum_to_one_hundred() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;
    let repo = Repository::open(&repo_path)?;

    commit_file(
        &repo,
        &repo_path,
        "a.txt",
        "a",
        "Tom",
        "tom@example.com",
        DAY_ONE,
    )?;
    commit_file(
        &repo,
        &repo_path,
        "b.txt",
        "b",
        "Alice",
        "alice@example.com",
        DAY_ONE + 60,
    )?;
    commit_file(
        &repo,
        &repo_path,
        "c.txt",
        "c",
        "Bob",
        "bob@example.com",
        DAY_ONE + 120,
    )?;
    commit_file(
        &repo,
        &repo_path,
        "d.txt",
        "d",
        "Bob",
        "bob@example.com",
        DAY_ONE + 180,
    )?;

    let contributors = analyze_contributors(&repo_path)?;
    let total_ownership = contributors
        .iter()
        .map(|contributor| contributor.ownership_percent)
        .sum::<f64>();
    let bob = contributor_by_email(&contributors, "bob@example.com")?;

    assert!((total_ownership - 100.0).abs() < 0.01);
    assert!((bob.ownership_percent - 50.0).abs() < f64::EPSILON);

    Ok(())
}

#[test]
fn first_last_commit_tracks_range() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;
    let repo = Repository::open(&repo_path)?;

    commit_file(
        &repo,
        &repo_path,
        "a.txt",
        "a",
        "Tom",
        "tom@example.com",
        DAY_ONE,
    )?;
    commit_file(
        &repo,
        &repo_path,
        "b.txt",
        "b",
        "Tom",
        "tom@example.com",
        DAY_THREE,
    )?;

    let contributors = analyze_contributors(&repo_path)?;
    let tom = contributor_by_email(&contributors, "tom@example.com")?;
    let first_commit = tom.first_commit.ok_or("first commit should exist")?;
    let last_commit = tom.last_commit.ok_or("last commit should exist")?;

    assert!(first_commit < last_commit);
    assert_eq!(first_commit.timestamp(), DAY_ONE);
    assert_eq!(last_commit.timestamp(), DAY_THREE);

    Ok(())
}

#[test]
fn sorting_orders_by_commit_count_desc() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;
    let repo = Repository::open(&repo_path)?;

    commit_file(
        &repo,
        &repo_path,
        "a.txt",
        "a",
        "Bob",
        "bob@example.com",
        DAY_ONE,
    )?;
    commit_file(
        &repo,
        &repo_path,
        "b.txt",
        "b",
        "Tom",
        "tom@example.com",
        DAY_ONE + 60,
    )?;
    commit_file(
        &repo,
        &repo_path,
        "c.txt",
        "c",
        "Tom",
        "tom@example.com",
        DAY_ONE + 120,
    )?;
    commit_file(
        &repo,
        &repo_path,
        "d.txt",
        "d",
        "Alice",
        "alice@example.com",
        DAY_ONE + 180,
    )?;
    commit_file(
        &repo,
        &repo_path,
        "e.txt",
        "e",
        "Alice",
        "alice@example.com",
        DAY_ONE + 240,
    )?;
    commit_file(
        &repo,
        &repo_path,
        "f.txt",
        "f",
        "Alice",
        "alice@example.com",
        DAY_ONE + 300,
    )?;

    let contributors = analyze_contributors(&repo_path)?;
    let counts = contributors
        .iter()
        .map(|contributor| (contributor.email.as_str(), contributor.commit_count))
        .collect::<Vec<_>>();

    assert_eq!(
        counts,
        vec![
            ("alice@example.com", 3),
            ("tom@example.com", 2),
            ("bob@example.com", 1)
        ]
    );

    Ok(())
}

fn init_temp_repository() -> Result<(TempDir, std::path::PathBuf), Box<dyn Error>> {
    let temp_dir = tempfile::tempdir()?;
    Repository::init(temp_dir.path())?;
    let repo_path = temp_dir.path().to_path_buf();

    Ok((temp_dir, repo_path))
}

fn analyze_contributors(repo_path: &Path) -> Result<Vec<ContributorStats>, Box<dyn Error>> {
    let repository = GitRepository::open(repo_path)?;
    Ok(repository.contributors()?)
}

fn contributor_by_email<'a>(
    contributors: &'a [ContributorStats],
    email: &str,
) -> Result<&'a ContributorStats, Box<dyn Error>> {
    contributors
        .iter()
        .find(|contributor| contributor.email == email)
        .ok_or_else(|| format!("missing contributor {email}").into())
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
        "test commit",
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
