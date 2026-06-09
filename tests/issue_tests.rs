mod common;

use common::TestRepo;

#[tokio::test]
async fn issue_create_via_arg() {
    let repo = TestRepo::new();
    let output = repo
        .dg_command()
        .arg("issue")
        .arg("create")
        .arg("My first issue")
        .output()
        .expect("command failed to execute");
    assert!(
        output.status.success(),
        "exit ok: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = std::str::from_utf8(&output.stdout).expect("valid utf8 output");
    let json: serde_json::Value = serde_json::from_str(stdout).expect("valid JSON");

    assert!(json.get("issue_id").is_some(), "should have issue_id");
    assert_eq!(
        json.get("description").and_then(|v| v.as_str()),
        Some("My first issue")
    );
    assert!(
        json.get("name").map(|v| v.is_null()).unwrap_or(false),
        "name should be null when --name is not provided"
    );
    assert_eq!(
        json.get("author").and_then(|v| v.as_str()),
        Some("Test Robot"),
        "default author should be the robot profile"
    );
    assert!(json.get("created_at").is_some(), "should have created_at");
    assert!(json.get("updated_at").is_some(), "should have updated_at");
    assert!(
        json.get("author_id").is_none(),
        "author_id must not be exposed in output"
    );
}

#[tokio::test]
async fn issue_create_via_stdin() {
    let repo = TestRepo::new();
    let output = repo
        .dg_command()
        .arg("issue")
        .arg("create")
        .write_stdin("Issue via stdin\n")
        .output()
        .expect("command failed to execute");
    assert!(
        output.status.success(),
        "exit ok: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = std::str::from_utf8(&output.stdout).expect("valid utf8");
    let json: serde_json::Value = serde_json::from_str(stdout).expect("valid JSON");
    assert_eq!(
        json.get("description").and_then(|v| v.as_str()),
        Some("Issue via stdin")
    );
}

#[tokio::test]
async fn issue_create_empty_fails() {
    let repo = TestRepo::new();
    let output = repo
        .dg_command()
        .arg("issue")
        .arg("create")
        .arg("")
        .output()
        .expect("command failed to execute");
    assert!(!output.status.success(), "should fail on empty input");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("empty"),
        "stderr should explain the empty-content error, got: {stderr}"
    );
}

#[tokio::test]
async fn issue_create_empty_stdin_fails() {
    let repo = TestRepo::new();
    let output = repo
        .dg_command()
        .arg("issue")
        .arg("create")
        .write_stdin("   \n")
        .output()
        .expect("command failed to execute");
    assert!(
        !output.status.success(),
        "should fail on whitespace-only stdin"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("empty"),
        "stderr should explain the empty-content error, got: {stderr}"
    );
}

#[tokio::test]
async fn issue_read() {
    let repo = TestRepo::new();

    repo.dg_command()
        .arg("issue")
        .arg("create")
        .arg("Test issue for reading")
        .output()
        .expect("create issue failed");

    let output = repo
        .dg_command()
        .arg("issue")
        .arg("read")
        .arg("1")
        .output()
        .expect("command failed to execute");
    assert!(
        output.status.success(),
        "exit ok: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = std::str::from_utf8(&output.stdout).expect("valid utf8");
    let json: serde_json::Value = serde_json::from_str(stdout).expect("valid JSON");

    assert!(json.get("issue").is_some(), "should have 'issue' field");
    assert!(
        json.get("comments").is_some(),
        "should have 'comments' field"
    );

    let issue = &json["issue"];
    assert_eq!(
        issue.get("description").and_then(|v| v.as_str()),
        Some("Test issue for reading")
    );
}

#[tokio::test]
async fn issue_read_not_found() {
    let repo = TestRepo::new();
    let output = repo
        .dg_command()
        .arg("issue")
        .arg("read")
        .arg("999")
        .output()
        .expect("command failed to execute");
    assert!(!output.status.success(), "should fail on nonexistent issue");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found"),
        "stderr should report the missing issue, got: {stderr}"
    );
}

#[tokio::test]
async fn issue_comment() {
    let repo = TestRepo::new();

    repo.dg_command()
        .arg("issue")
        .arg("create")
        .arg("Issue to comment on")
        .output()
        .expect("create issue failed");

    let output = repo
        .dg_command()
        .arg("issue")
        .arg("comment")
        .arg("1")
        .arg("a comment")
        .output()
        .expect("command failed to execute");
    assert!(
        output.status.success(),
        "exit ok: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = std::str::from_utf8(&output.stdout).expect("valid utf8");
    let json: serde_json::Value = serde_json::from_str(stdout).expect("valid JSON");
    assert_eq!(
        json.get("content").and_then(|v| v.as_str()),
        Some("a comment")
    );
}

#[tokio::test]
async fn issue_comment_via_stdin() {
    let repo = TestRepo::new();

    repo.dg_command()
        .arg("issue")
        .arg("create")
        .arg("Issue for stdin comment")
        .output()
        .expect("create issue failed");

    let output = repo
        .dg_command()
        .arg("issue")
        .arg("comment")
        .arg("1")
        .write_stdin("stdin comment content\n")
        .output()
        .expect("command failed to execute");
    assert!(
        output.status.success(),
        "exit ok: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = std::str::from_utf8(&output.stdout).expect("valid utf8");
    let json: serde_json::Value = serde_json::from_str(stdout).expect("valid JSON");
    assert_eq!(
        json.get("content").and_then(|v| v.as_str()),
        Some("stdin comment content")
    );
}

#[tokio::test]
async fn no_git_repo_fails() {
    let tmp = TestRepo::new_no_git();
    let output = tmp
        .dg_command()
        .arg("issue")
        .arg("create")
        .arg("x")
        .output()
        .expect("command failed to execute");
    assert!(!output.status.success(), "should fail outside git repo");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("repository"),
        "stderr should report the missing git repository, got: {stderr}"
    );
}

#[tokio::test]
async fn issue_create_with_name_flag() {
    let repo = TestRepo::new();
    let output = repo
        .dg_command()
        .arg("issue")
        .arg("create")
        .arg("The full description body")
        .arg("--name")
        .arg("Short title")
        .output()
        .expect("command failed to execute");
    assert!(
        output.status.success(),
        "exit ok: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = std::str::from_utf8(&output.stdout).expect("valid utf8");
    let json: serde_json::Value = serde_json::from_str(stdout).expect("valid JSON");
    assert_eq!(
        json.get("name").and_then(|v| v.as_str()),
        Some("Short title")
    );
    assert_eq!(
        json.get("description").and_then(|v| v.as_str()),
        Some("The full description body")
    );
}

#[tokio::test]
async fn issue_create_with_human_flag_uses_human_author() {
    let repo = TestRepo::new();
    let output = repo
        .dg_command()
        .arg("--human")
        .arg("issue")
        .arg("create")
        .arg("human-authored issue")
        .output()
        .expect("command failed to execute");
    assert!(
        output.status.success(),
        "exit ok: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = std::str::from_utf8(&output.stdout).expect("valid utf8");
    let json: serde_json::Value = serde_json::from_str(stdout).expect("valid JSON");
    assert_eq!(
        json.get("author").and_then(|v| v.as_str()),
        Some("Test Human"),
        "--human should resolve to the human profile"
    );
}

#[tokio::test]
async fn issue_create_with_author_flag_uses_named_profile() {
    let repo = TestRepo::new();
    let output = repo
        .dg_command()
        .arg("--author")
        .arg("extra")
        .arg("issue")
        .arg("create")
        .arg("extra-profile issue")
        .output()
        .expect("command failed to execute");
    assert!(
        output.status.success(),
        "exit ok: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = std::str::from_utf8(&output.stdout).expect("valid utf8");
    let json: serde_json::Value = serde_json::from_str(stdout).expect("valid JSON");
    assert_eq!(
        json.get("author").and_then(|v| v.as_str()),
        Some("Test Extra"),
        "--author extra should resolve to the extra profile"
    );
}

#[tokio::test]
async fn human_and_author_flags_are_mutually_exclusive() {
    let repo = TestRepo::new();
    let output = repo
        .dg_command()
        .arg("--human")
        .arg("--author")
        .arg("extra")
        .arg("issue")
        .arg("create")
        .arg("conflict")
        .output()
        .expect("command failed to execute");
    assert!(
        !output.status.success(),
        "clap should reject --human and --author together"
    );
}

#[tokio::test]
async fn missing_required_profile_field_exits_with_error() {
    let repo = TestRepo::new();

    let broken_config_dir = tempfile::tempdir().expect("config temp dir");
    let cfg_dir = broken_config_dir.path().join("doppelganger");
    std::fs::create_dir_all(&cfg_dir).expect("create config dir");
    std::fs::write(
        cfg_dir.join("config.toml"),
        r#"
[default_human_author]
name = "Only Human"
email = "human@example.com"
"#,
    )
    .expect("write broken config");

    let output = repo
        .dg_command()
        .env("DOPPELGANGER_CONFIG_DIR", broken_config_dir.path())
        .arg("issue")
        .arg("create")
        .arg("should fail")
        .output()
        .expect("command failed to execute");

    assert!(
        !output.status.success(),
        "should fail with missing required field, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("default_robot_author"),
        "stderr should name the missing field, got: {stderr}"
    );
}

#[tokio::test]
async fn first_run_writes_sample_and_exits_zero() {
    let repo = TestRepo::new();
    let empty_config_dir = tempfile::tempdir().expect("empty config dir");

    let output = repo
        .dg_command()
        .env("DOPPELGANGER_CONFIG_DIR", empty_config_dir.path())
        .arg("issue")
        .arg("create")
        .arg("should not run")
        .output()
        .expect("command failed to execute");

    assert!(
        output.status.success(),
        "first-run should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "stdout must be empty on first run"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("config"),
        "stderr should mention config, got: {stderr}"
    );

    let config_file = empty_config_dir
        .path()
        .join("doppelganger")
        .join("config.toml");
    assert!(config_file.exists(), "sample config.toml should be written");
}
