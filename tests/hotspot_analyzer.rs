use std::error::Error;
use std::path::Path;

use git2::{ErrorCode, Oid, Repository, Signature, Time};
use gitinsight_rs::analytics::FileHotspot;
use gitinsight_rs::git::GitRepository;
use tempfile::TempDir;

const BASE_TIME: i64 = 1_735_689_600;

#[test]
fn hotspot_counting() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;
    let repo = Repository::open(&repo_path)?;

    commit_file(
        &repo,
        &repo_path,
        "src/main.rs",
        "fn main() {}\n",
        BASE_TIME,
    )?;
    commit_file(
        &repo,
        &repo_path,
        "src/main.rs",
        "fn main() { println!(\"hi\"); }\n",
        BASE_TIME + 60,
    )?;
    commit_file(
        &repo,
        &repo_path,
        "src/lib.rs",
        "pub fn lib() {}\n",
        BASE_TIME + 120,
    )?;
    commit_file(
        &repo,
        &repo_path,
        "src/main.rs",
        "fn main() { println!(\"hello\"); }\n",
        BASE_TIME + 180,
    )?;

    let hotspots = analyze_hotspots(&repo_path)?;
    let main_rs = hotspot_by_path(&hotspots, "src/main.rs")?;
    let lib_rs = hotspot_by_path(&hotspots, "src/lib.rs")?;

    assert_eq!(main_rs.change_count, 3);
    assert_eq!(
        main_rs
            .last_modified
            .expect("last modified should exist")
            .timestamp(),
        BASE_TIME + 180
    );
    assert_eq!(lib_rs.change_count, 1);

    Ok(())
}

#[test]
fn hotspot_sorting() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;
    let repo = Repository::open(&repo_path)?;

    commit_file(&repo, &repo_path, "a.txt", "a1", BASE_TIME)?;
    commit_file(&repo, &repo_path, "b.txt", "b1", BASE_TIME + 60)?;
    commit_file(&repo, &repo_path, "a.txt", "a2", BASE_TIME + 120)?;
    commit_file(&repo, &repo_path, "c.txt", "c1", BASE_TIME + 180)?;
    commit_file(&repo, &repo_path, "b.txt", "b2", BASE_TIME + 240)?;
    commit_file(&repo, &repo_path, "a.txt", "a3", BASE_TIME + 300)?;

    let hotspots = analyze_hotspots(&repo_path)?;
    let ordered = hotspots
        .iter()
        .map(|hotspot| (hotspot.path.as_str(), hotspot.change_count))
        .collect::<Vec<_>>();

    assert_eq!(ordered, vec![("a.txt", 3), ("b.txt", 2), ("c.txt", 1)]);

    Ok(())
}

#[test]
fn empty_repository() -> Result<(), Box<dyn Error>> {
    let (_temp_dir, repo_path) = init_temp_repository()?;

    let hotspots = analyze_hotspots(&repo_path)?;

    assert!(hotspots.is_empty());

    Ok(())
}

fn init_temp_repository() -> Result<(TempDir, std::path::PathBuf), Box<dyn Error>> {
    let temp_dir = tempfile::tempdir()?;
    Repository::init(temp_dir.path())?;
    let repo_path = temp_dir.path().to_path_buf();

    Ok((temp_dir, repo_path))
}

fn analyze_hotspots(repo_path: &Path) -> Result<Vec<FileHotspot>, Box<dyn Error>> {
    let repository = GitRepository::open(repo_path)?;
    Ok(repository.hotspots()?)
}

fn hotspot_by_path<'a>(
    hotspots: &'a [FileHotspot],
    path: &str,
) -> Result<&'a FileHotspot, Box<dyn Error>> {
    hotspots
        .iter()
        .find(|hotspot| hotspot.path == path)
        .ok_or_else(|| format!("missing hotspot {path}").into())
}

fn commit_file(
    repo: &Repository,
    workdir: &Path,
    relative_path: &str,
    contents: &str,
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
    let signature = Signature::new("Hotspot Tester", "hotspot@example.com", &time)?;
    let parents = current_head_commit(repo)?;
    let parent_refs = parents.iter().collect::<Vec<_>>();

    let commit_id = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "hotspot test commit",
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
