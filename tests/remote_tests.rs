mod common;

use common::TestRepo;

#[tokio::test]
async fn sync_without_github_token_errors() {
    let repo = TestRepo::new();
    let output = repo
        .dg_command()
        .arg("issue")
        .arg("sync")
        .arg("42")
        .output()
        .expect("command failed to execute");
    assert!(!output.status.success(), "should fail without GitHub token");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("GitHub token") || stderr.contains("github"),
        "stderr should mention GitHub token, got: {stderr}"
    );
}

#[tokio::test]
async fn sync_without_git_remote_errors() {
    let repo = TestRepo::new();

    let config_dir = tempfile::tempdir().expect("config temp dir");
    let cfg_dir = config_dir.path().join("doppelganger");
    std::fs::create_dir_all(&cfg_dir).expect("create config dir");
    std::fs::write(
        cfg_dir.join("config.toml"),
        r#"
[default_human_author]
name = "Test Human"
email = "human@example.com"

[default_robot_author]
name = "Test Robot"

[github]
token = "ghp_fake_token_for_test"
"#,
    )
    .expect("write config");

    let output = repo
        .dg_command()
        .env("DOPPELGANGER_CONFIG_DIR", config_dir.path())
        .arg("issue")
        .arg("sync")
        .arg("42")
        .output()
        .expect("command failed to execute");
    assert!(
        !output.status.success(),
        "should fail without git remote origin"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("remote") || stderr.contains("origin"),
        "stderr should mention missing remote, got: {stderr}"
    );
}

#[tokio::test]
async fn sync_overwrite_nonexistent_issue_errors() {
    let repo = TestRepo::new_with_commit();

    let config_dir = tempfile::tempdir().expect("config temp dir");
    let cfg_dir = config_dir.path().join("doppelganger");
    std::fs::create_dir_all(&cfg_dir).expect("create config dir");
    std::fs::write(
        cfg_dir.join("config.toml"),
        r#"
[default_human_author]
name = "Test Human"
email = "human@example.com"

[default_robot_author]
name = "Test Robot"

[github]
token = "ghp_fake_token_for_test"
"#,
    )
    .expect("write config");

    let git_repo = git2::Repository::open(&repo.path).expect("open repo");
    git_repo
        .remote("origin", "https://github.com/testowner/testrepo.git")
        .expect("add remote");

    let output = repo
        .dg_command()
        .env("DOPPELGANGER_CONFIG_DIR", config_dir.path())
        .arg("issue")
        .arg("sync")
        .arg("42")
        .arg("--overwrite")
        .arg("999")
        .output()
        .expect("command failed to execute");
    assert!(
        !output.status.success(),
        "should fail with nonexistent overwrite target"
    );
}

#[tokio::test]
async fn sync_with_non_github_remote_errors() {
    let repo = TestRepo::new_with_commit();

    let config_dir = tempfile::tempdir().expect("config temp dir");
    let cfg_dir = config_dir.path().join("doppelganger");
    std::fs::create_dir_all(&cfg_dir).expect("create config dir");
    std::fs::write(
        cfg_dir.join("config.toml"),
        r#"
[default_human_author]
name = "Test Human"
email = "human@example.com"

[default_robot_author]
name = "Test Robot"

[github]
token = "ghp_fake_token_for_test"
"#,
    )
    .expect("write config");

    let git_repo = git2::Repository::open(&repo.path).expect("open repo");
    git_repo
        .remote("origin", "https://gitlab.com/testowner/testrepo.git")
        .expect("add remote");

    let output = repo
        .dg_command()
        .env("DOPPELGANGER_CONFIG_DIR", config_dir.path())
        .arg("issue")
        .arg("sync")
        .arg("42")
        .output()
        .expect("command failed to execute");
    assert!(
        !output.status.success(),
        "should fail with non-GitHub remote"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("GitHub") || stderr.contains("remote"),
        "stderr should mention GitHub/remote issue, got: {stderr}"
    );
}
