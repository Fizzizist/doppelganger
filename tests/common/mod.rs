#![allow(dead_code)]

use std::path::PathBuf;
use tempfile::TempDir;

pub struct TestRepo {
    pub dir: TempDir,
    pub path: PathBuf,
}

impl TestRepo {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().to_path_buf();

        let repo = git2::Repository::init(&path).expect("git init");

        let mut config = repo.config().expect("repo config");
        config
            .set_str("user.name", "Test User")
            .expect("set user.name");
        config
            .set_str("user.email", "test@example.com")
            .expect("set user.email");

        TestRepo { dir, path }
    }

    pub fn new_no_email() -> Self {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().to_path_buf();
        let repo = git2::Repository::init(&path).expect("git init");
        let mut config = repo.config().expect("repo config");
        config
            .set_str("user.name", "Test User")
            .expect("set user.name");
        TestRepo { dir, path }
    }

    pub fn new_with_commit() -> Self {
        let repo = Self::new();
        let git_repo = git2::Repository::open(&repo.path).expect("open repo");
        make_initial_commit(&git_repo);
        repo
    }

    pub fn new_no_git() -> Self {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().to_path_buf();
        // No git init — just a plain directory
        TestRepo { dir, path }
    }
}

pub fn make_initial_commit(repo: &git2::Repository) {
    let sig = git2::Signature::now("Test", "test@test.com").expect("sig");
    let tree_id = {
        let mut index = repo.index().expect("index");
        index.write_tree().expect("write tree")
    };
    let tree = repo.find_tree(tree_id).expect("find tree");
    repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
        .expect("commit");
}
