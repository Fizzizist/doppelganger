mod common;

use assert_cmd::Command;
use common::TestRepo;

fn dg_command() -> Command {
    Command::cargo_bin("dg").expect("dg binary")
}

#[tokio::test]
async fn issue_create_via_arg() {
    let repo = TestRepo::new();
    let output = dg_command()
        .current_dir(&repo.path)
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
        Some("Test User"),
        "author should be the local git config user.name"
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
    let output = dg_command()
        .current_dir(&repo.path)
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
    let output = dg_command()
        .current_dir(&repo.path)
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
    let output = dg_command()
        .current_dir(&repo.path)
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

    // Create an issue first
    dg_command()
        .current_dir(&repo.path)
        .arg("issue")
        .arg("create")
        .arg("Test issue for reading")
        .output()
        .expect("create issue failed");

    // Read it back
    let output = dg_command()
        .current_dir(&repo.path)
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
    let output = dg_command()
        .current_dir(&repo.path)
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

    // Create an issue first
    dg_command()
        .current_dir(&repo.path)
        .arg("issue")
        .arg("create")
        .arg("Issue to comment on")
        .output()
        .expect("create issue failed");

    // Comment on it
    let output = dg_command()
        .current_dir(&repo.path)
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

    dg_command()
        .current_dir(&repo.path)
        .arg("issue")
        .arg("create")
        .arg("Issue for stdin comment")
        .output()
        .expect("create issue failed");

    let output = dg_command()
        .current_dir(&repo.path)
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
    let output = dg_command()
        .current_dir(&tmp.path)
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
    let output = dg_command()
        .current_dir(&repo.path)
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
