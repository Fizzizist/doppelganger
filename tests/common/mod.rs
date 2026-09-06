#![allow(dead_code)]

use std::path::PathBuf;
use tempfile::TempDir;

use assert_cmd::Command;

const TEST_CONFIG: &str = r#"
[default_human_author]
name = "Test Human"
email = "human@example.com"

[default_robot_author]
name = "Test Robot"

[profiles.extra]
name = "Test Extra"
email = "extra@example.com"
"#;

pub struct TestRepo {
    pub dir: TempDir,
    pub path: PathBuf,
    pub config_dir: TempDir,
}

impl TestRepo {
    fn write_test_config(config_dir: &TempDir) {
        let cfg_dir = config_dir.path().join("doppelganger");
        std::fs::create_dir_all(&cfg_dir).expect("create config dir");
        std::fs::write(cfg_dir.join("config.toml"), TEST_CONFIG).expect("write test config");
    }

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

        let config_dir = tempfile::tempdir().expect("config temp dir");
        Self::write_test_config(&config_dir);

        TestRepo {
            dir,
            path,
            config_dir,
        }
    }

    pub fn new_no_email() -> Self {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().to_path_buf();
        let repo = git2::Repository::init(&path).expect("git init");
        let mut config = repo.config().expect("repo config");
        config
            .set_str("user.name", "Test User")
            .expect("set user.name");

        let config_dir = tempfile::tempdir().expect("config temp dir");
        Self::write_test_config(&config_dir);

        TestRepo {
            dir,
            path,
            config_dir,
        }
    }

    pub fn new_with_commit() -> Self {
        let repo = Self::new();
        let git_repo = git2::Repository::open(&repo.path).expect("open repo");
        make_initial_commit(&git_repo);
        repo
    }

    pub fn set_origin(&self, url: &str) {
        let git_repo = git2::Repository::open(&self.path).expect("open repo");
        git_repo.remote("origin", url).expect("add origin remote");
    }

    pub fn new_no_git() -> Self {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().to_path_buf();

        let config_dir = tempfile::tempdir().expect("config temp dir");
        Self::write_test_config(&config_dir);

        TestRepo {
            dir,
            path,
            config_dir,
        }
    }

    pub fn dg_command(&self) -> Command {
        let mut cmd = Command::cargo_bin("dg").expect("dg binary");
        cmd.current_dir(&self.path)
            .env("DOPPELGANGER_CONFIG_DIR", self.config_dir.path());
        cmd
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
