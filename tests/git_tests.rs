mod common;

use std::sync::OnceLock;

use common::TestRepo;
use doppelganger::git::{current_branch, discover_repo, remote_url};
use tempfile::TempDir;

/// Point libgit2's global/system/XDG config search paths at an empty directory
/// so tests never resolve against the developer's real git config. Done exactly
/// once for the whole test binary via `OnceLock`, which serializes all callers
/// until the search paths are stable — so no concurrent config access races with
/// the (process-global) `set_search_path` mutation.
fn isolate_git_config() {
    static EMPTY_CONFIG_DIR: OnceLock<TempDir> = OnceLock::new();
    EMPTY_CONFIG_DIR.get_or_init(|| {
        let dir = tempfile::tempdir().expect("temp config dir");
        let path = dir.path().to_str().expect("utf-8 config path");
        for level in [
            git2::ConfigLevel::System,
            git2::ConfigLevel::Global,
            git2::ConfigLevel::XDG,
            git2::ConfigLevel::ProgramData,
        ] {
            // SAFETY: invoked exactly once via OnceLock before any test performs
            // git config access, so libgit2's search path is mutated with no
            // concurrent reads/writes in flight.
            unsafe {
                git2::opts::set_search_path(level, path).expect("set git search path");
            }
        }
        dir
    });
}

#[tokio::test]
async fn discover_repo_finds_git() {
    isolate_git_config();
    let result = discover_repo();
    assert!(
        result.is_ok(),
        "discover_repo should succeed in this project's git root"
    );
}

#[tokio::test]
async fn discover_repo_fails_outside_git() {
    isolate_git_config();
    let tmp = tempfile::tempdir().expect("temp dir");
    let old_cwd = std::env::current_dir().expect("current dir");
    std::env::set_current_dir(tmp.path()).expect("change dir");

    struct CwdGuard(std::path::PathBuf);
    impl Drop for CwdGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.0);
        }
    }
    let _guard = CwdGuard(old_cwd);

    let result = discover_repo();
    assert!(result.is_err(), "should fail outside a git repo");
    match result {
        Err(doppelganger::error::Error::NoRepository) => {}
        Err(e) => panic!("expected NoRepository error, got: {e}"),
        Ok(_) => unreachable!(),
    }
}

#[tokio::test]
async fn current_branch_returns_name() {
    isolate_git_config();
    let repo_dir = TestRepo::new_with_commit();
    let repo = git2::Repository::open(&repo_dir.path).expect("open repo");

    let branch = current_branch(&repo).expect("current branch should work");
    assert!(
        !branch.is_empty(),
        "current_branch should return a non-empty branch name, got: {branch:?}"
    );
}

#[tokio::test]
async fn current_branch_detached_head_errors() {
    isolate_git_config();
    let repo_dir = TestRepo::new_with_commit();
    let repo = git2::Repository::open(&repo_dir.path).expect("open repo");

    let head_commit = repo
        .head()
        .expect("head")
        .peel_to_commit()
        .expect("peel to commit");
    repo.set_head_detached(head_commit.id())
        .expect("detach head");

    let result = current_branch(&repo);
    match result {
        Err(doppelganger::error::Error::DetachedHead) => {}
        Err(e) => panic!("expected DetachedHead, got: {e}"),
        Ok(name) => panic!("expected DetachedHead, got branch name: {name}"),
    }
}

#[tokio::test]
async fn remote_url_returns_origin_url() {
    isolate_git_config();
    let repo_dir = TestRepo::new_with_commit();
    let repo = git2::Repository::open(&repo_dir.path).expect("open repo");
    repo.remote("origin", "https://github.com/owner/repo.git")
        .expect("add remote");
    let url = remote_url(&repo).expect("get remote url");
    assert_eq!(url, "https://github.com/owner/repo.git");
}

#[tokio::test]
async fn remote_url_errors_without_origin() {
    isolate_git_config();
    let repo_dir = TestRepo::new_with_commit();
    let repo = git2::Repository::open(&repo_dir.path).expect("open repo");
    let result = remote_url(&repo);
    assert!(result.is_err(), "should fail without origin remote");
}
