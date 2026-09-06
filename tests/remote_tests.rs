mod common;

use common::TestRepo;

const GITHUB_TOKEN_CONFIG: &str = r#"
[default_human_author]
name = "Test Human"
email = "human@example.com"

[default_robot_author]
name = "Test Robot"

[github]
token = "ghp_fake_token_for_test"
"#;

const NO_TOKEN_CONFIG: &str = r#"
[default_human_author]
name = "Test Human"
email = "human@example.com"

[default_robot_author]
name = "Test Robot"
"#;

fn write_config_file(config_dir: &tempfile::TempDir, contents: &str) {
    let cfg_dir = config_dir.path().join("doppelganger");
    std::fs::create_dir_all(&cfg_dir).expect("create config dir");
    std::fs::write(cfg_dir.join("config.toml"), contents).expect("write config");
}

#[tokio::test]
async fn sync_without_github_token_errors() {
    let repo = TestRepo::new();
    repo.set_origin("https://github.com/testowner/testrepo.git");

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

    write_config_file(&repo.config_dir, GITHUB_TOKEN_CONFIG);

    let output = repo
        .dg_command()
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
    repo.set_origin("https://github.com/testowner/testrepo.git");

    write_config_file(&repo.config_dir, GITHUB_TOKEN_CONFIG);

    let output = repo
        .dg_command()
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
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panicked"),
        "binary must fail with a clean error, not a panic, got: {stderr}"
    );
    assert!(
        stderr.contains("error:"),
        "stderr should carry the CLI error prefix, got: {stderr}"
    );
}

#[tokio::test]
async fn sync_with_unsupported_remote_errors() {
    let repo = TestRepo::new_with_commit();
    repo.set_origin("https://bitbucket.org/testowner/testrepo.git");

    write_config_file(&repo.config_dir, GITHUB_TOKEN_CONFIG);

    let output = repo
        .dg_command()
        .arg("issue")
        .arg("sync")
        .arg("42")
        .output()
        .expect("command failed to execute");
    assert!(
        !output.status.success(),
        "should fail with unsupported remote host"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("GitHub") && stderr.contains("GitLab"),
        "stderr should name the supported forges, got: {stderr}"
    );
}

#[tokio::test]
async fn sync_from_gitlab_without_token_errors() {
    let repo = TestRepo::new_with_commit();
    repo.set_origin("https://gitlab.com/testowner/testrepo.git");

    write_config_file(&repo.config_dir, NO_TOKEN_CONFIG);

    let output = repo
        .dg_command()
        .arg("issue")
        .arg("sync")
        .arg("42")
        .output()
        .expect("command failed to execute");
    assert!(
        !output.status.success(),
        "should fail without a [gitlab] section"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("gitlab"),
        "stderr should mention the gitlab token config, got: {stderr}"
    );
}

#[tokio::test]
async fn sync_dispatch_follows_remote_url_not_config() {
    let repo = TestRepo::new_with_commit();
    repo.set_origin("https://github.com/testowner/testrepo.git");

    let mut both_config = NO_TOKEN_CONFIG.to_string();
    both_config.push_str("\n[gitlab]\ntoken = \"glpat_decoy_token\"\n");
    write_config_file(&repo.config_dir, &both_config);

    let output = repo
        .dg_command()
        .arg("issue")
        .arg("sync")
        .arg("42")
        .output()
        .expect("command failed to execute");
    assert!(
        !output.status.success(),
        "github remote with only [gitlab] configured should dispatch to GitHub and fail on its token"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("github"),
        "stderr should mention the [github] section, proving URL-based dispatch, got: {stderr}"
    );
}
