mod common;

use assert_cmd::Command;
use common::TestRepo;

fn dg_command() -> Command {
    Command::cargo_bin("dg").expect("dg binary")
}

fn setup_with_issue(repo: &TestRepo) {
    dg_command()
        .current_dir(&repo.path)
        .arg("issue")
        .arg("create")
        .arg("issue description")
        .output()
        .expect("create issue failed");
}

#[tokio::test]
async fn branch_create() {
    let repo = TestRepo::new_with_commit();
    setup_with_issue(&repo);

    let output = dg_command()
        .current_dir(&repo.path)
        .arg("branch")
        .arg("create")
        .arg("1")
        .arg("my branch")
        .output()
        .expect("command failed to execute");
    assert!(
        output.status.success(),
        "exit ok: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = std::str::from_utf8(&output.stdout).expect("valid utf8");
    let json: serde_json::Value = serde_json::from_str(stdout).expect("valid JSON");

    assert!(json.get("branch_id").is_some(), "should have branch_id");
    assert!(json.get("name").is_some(), "should have name");
    assert_eq!(
        json.get("description").and_then(|v| v.as_str()),
        Some("my branch")
    );
}

#[tokio::test]
async fn branch_create_duplicate_fails() {
    let repo = TestRepo::new_with_commit();
    setup_with_issue(&repo);

    let output = dg_command()
        .current_dir(&repo.path)
        .arg("branch")
        .arg("create")
        .arg("1")
        .arg("first")
        .output()
        .expect("command failed to execute");
    assert!(output.status.success(), "first create should succeed");

    let output = dg_command()
        .current_dir(&repo.path)
        .arg("branch")
        .arg("create")
        .arg("1")
        .arg("second")
        .output()
        .expect("command failed to execute");
    assert!(
        !output.status.success(),
        "duplicate create should fail: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn branch_create_with_overwrite() {
    let repo = TestRepo::new_with_commit();
    setup_with_issue(&repo);

    let output = dg_command()
        .current_dir(&repo.path)
        .arg("branch")
        .arg("create")
        .arg("1")
        .arg("original")
        .output()
        .expect("command failed to execute");
    assert!(output.status.success(), "first create should succeed");

    let output = dg_command()
        .current_dir(&repo.path)
        .arg("branch")
        .arg("create")
        .arg("1")
        .arg("updated desc")
        .arg("--overwrite")
        .output()
        .expect("command failed to execute");
    assert!(
        output.status.success(),
        "overwrite should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = std::str::from_utf8(&output.stdout).expect("valid utf8");
    let json: serde_json::Value = serde_json::from_str(stdout).expect("valid JSON");
    assert_eq!(
        json.get("description").and_then(|v| v.as_str()),
        Some("updated desc")
    );
}

#[tokio::test]
async fn branch_read() {
    let repo = TestRepo::new_with_commit();
    setup_with_issue(&repo);

    let output = dg_command()
        .current_dir(&repo.path)
        .arg("branch")
        .arg("create")
        .arg("1")
        .arg("test branch")
        .output()
        .expect("command failed to execute");
    assert!(output.status.success(), "branch create should succeed");

    let output = dg_command()
        .current_dir(&repo.path)
        .arg("branch")
        .arg("read")
        .output()
        .expect("command failed to execute");
    assert!(
        output.status.success(),
        "branch read should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = std::str::from_utf8(&output.stdout).expect("valid utf8");
    let json: serde_json::Value = serde_json::from_str(stdout).expect("valid JSON");

    assert!(json.get("branch").is_some(), "should have 'branch' field");
    assert!(
        json.get("comments").is_some(),
        "should have 'comments' field"
    );
}

#[tokio::test]
async fn branch_read_no_record() {
    let repo = TestRepo::new_with_commit();
    let output = dg_command()
        .current_dir(&repo.path)
        .arg("branch")
        .arg("read")
        .output()
        .expect("command failed to execute");
    assert!(
        !output.status.success(),
        "should fail when no branch record exists: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn branch_comment() {
    let repo = TestRepo::new_with_commit();
    setup_with_issue(&repo);

    let output = dg_command()
        .current_dir(&repo.path)
        .arg("branch")
        .arg("create")
        .arg("1")
        .arg("branch desc")
        .output()
        .expect("command failed to execute");
    assert!(output.status.success(), "branch create should succeed");

    let output = dg_command()
        .current_dir(&repo.path)
        .arg("branch")
        .arg("comment")
        .arg("hello")
        .output()
        .expect("command failed to execute");
    assert!(
        output.status.success(),
        "comment should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = std::str::from_utf8(&output.stdout).expect("valid utf8");
    let json: serde_json::Value = serde_json::from_str(stdout).expect("valid JSON");
    assert_eq!(json.get("content").and_then(|v| v.as_str()), Some("hello"));
}

#[tokio::test]
async fn branch_create_invalid_issue() {
    let repo = TestRepo::new_with_commit();

    let output = dg_command()
        .current_dir(&repo.path)
        .arg("branch")
        .arg("create")
        .arg("999")
        .arg("desc")
        .output()
        .expect("command failed to execute");
    assert!(
        !output.status.success(),
        "should fail for nonexistent issue: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
