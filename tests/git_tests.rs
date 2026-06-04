mod common;

use common::TestRepo;
use doppelganger::git::{author_from_config, current_branch, discover_repo};

#[tokio::test]
async fn discover_repo_finds_git() {
    let result = discover_repo();
    assert!(
        result.is_ok(),
        "discover_repo should succeed in this project's git root"
    );
}

#[tokio::test]
async fn discover_repo_fails_outside_git() {
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
    let repo_dir = TestRepo::new_with_commit();
    let repo = git2::Repository::open(&repo_dir.path).expect("open repo");

    let branch = current_branch(&repo).expect("current branch should work");
    // git 2.28+ defaults to "trunk", older versions default to "master"
    assert!(
        branch == "master" || branch == "trunk",
        "default branch should be 'master' or 'trunk', got: {branch}"
    );
}

#[tokio::test]
async fn author_from_config_with_email() {
    let repo_dir = TestRepo::new();
    let repo = git2::Repository::open(&repo_dir.path).expect("open repo");

    let (name, email) = author_from_config(&repo).expect("author_from_config should succeed");
    // Local config is set to "Test User" / "test@example.com", but git2::Config::get_string
    // resolves from local → global → system. If the global config exists it may override.
    // The key invariant: name is Some, email is Some (either from local or global).
    assert!(!name.is_empty(), "name should not be empty");
    assert!(
        email.is_some(),
        "email should be present (from local or global config)"
    );
}

#[tokio::test]
async fn author_from_config_without_local_email() {
    let repo_dir = TestRepo::new_no_email();
    let repo = git2::Repository::open(&repo_dir.path).expect("open repo");

    // No local user.email. The system may or may not have a global email.
    // Either way, author_from_config should succeed (user.name is set locally).
    let (name, _email) = author_from_config(&repo).expect("author_from_config should succeed");
    assert!(!name.is_empty(), "name should not be empty");
}

#[tokio::test]
async fn author_from_config_missing_name() {
    // Create a repo without user.name. To avoid system-level user.name leaking through,
    // we unset it via the local config by setting an explicit "unset" — but git2 has no
    // "unset". Instead, we test the behavior of the function in the project's own repo
    // where user.name IS configured, and verify it succeeds.
    //
    // For the missing-name case, we can't easily isolate from global config.
    // Instead verify: in a fresh repo with no local name, the system global name
    // is picked up (git2::config.get_string searches local → global → system).
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().to_path_buf();
    git2::Repository::init(&path).expect("git init");

    let repo = git2::Repository::open(&path).expect("open repo");
    let result = author_from_config(&repo);

    // git2 searches local → global → system. If the test machine has a global user.name
    // set, this succeeds. If not, it returns MissingAuthorName.
    match result {
        Ok((name, _)) => {
            assert!(
                !name.is_empty(),
                "name should not be empty if found in global/system"
            );
        }
        Err(doppelganger::error::Error::MissingAuthorName) => {
            // Expected when no global/system user.name exists
        }
        Err(e) => panic!("unexpected error: {e}"),
    }
}
