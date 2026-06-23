mod common;

use common::TestRepo;

fn setup_with_issue(repo: &TestRepo) {
    repo.dg_command()
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

    let output = repo
        .dg_command()
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

    let output = repo
        .dg_command()
        .arg("branch")
        .arg("create")
        .arg("1")
        .arg("first")
        .output()
        .expect("command failed to execute");
    assert!(output.status.success(), "first create should succeed");

    let output = repo
        .dg_command()
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
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("already exists"),
        "stderr should report the duplicate branch, got: {stderr}"
    );
}

#[tokio::test]
async fn branch_overwrite_creates_when_no_record() {
    let repo = TestRepo::new_with_commit();
    setup_with_issue(&repo);

    let output = repo
        .dg_command()
        .arg("branch")
        .arg("create")
        .arg("1")
        .arg("fresh via overwrite")
        .arg("--overwrite")
        .output()
        .expect("command failed to execute");
    assert!(
        output.status.success(),
        "overwrite-create should succeed when no record exists: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = std::str::from_utf8(&output.stdout).expect("valid utf8");
    let json: serde_json::Value = serde_json::from_str(stdout).expect("valid JSON");
    assert_eq!(
        json.get("description").and_then(|v| v.as_str()),
        Some("fresh via overwrite")
    );
}

#[tokio::test]
async fn branch_create_with_overwrite() {
    let repo = TestRepo::new_with_commit();
    setup_with_issue(&repo);

    let output = repo
        .dg_command()
        .arg("branch")
        .arg("create")
        .arg("1")
        .arg("original")
        .output()
        .expect("command failed to execute");
    assert!(output.status.success(), "first create should succeed");

    let output = repo
        .dg_command()
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

    let output = repo
        .dg_command()
        .arg("branch")
        .arg("create")
        .arg("1")
        .arg("test branch")
        .output()
        .expect("command failed to execute");
    assert!(output.status.success(), "branch create should succeed");

    let output = repo
        .dg_command()
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
    let output = repo
        .dg_command()
        .arg("branch")
        .arg("read")
        .output()
        .expect("command failed to execute");
    assert!(
        !output.status.success(),
        "should fail when no branch record exists: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("branch create"),
        "stderr should prompt the user to create a branch, got: {stderr}"
    );
}

#[tokio::test]
async fn branch_comment_no_record() {
    let repo = TestRepo::new_with_commit();
    let output = repo
        .dg_command()
        .arg("branch")
        .arg("comment")
        .arg("orphan comment")
        .output()
        .expect("command failed to execute");
    assert!(
        !output.status.success(),
        "comment should fail when no branch record exists: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("branch create"),
        "stderr should prompt the user to create a branch, got: {stderr}"
    );
}

#[tokio::test]
async fn branch_comment() {
    let repo = TestRepo::new_with_commit();
    setup_with_issue(&repo);

    let output = repo
        .dg_command()
        .arg("branch")
        .arg("create")
        .arg("1")
        .arg("branch desc")
        .output()
        .expect("command failed to execute");
    assert!(output.status.success(), "branch create should succeed");

    let output = repo
        .dg_command()
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

    let output = repo
        .dg_command()
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
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found"),
        "stderr should report the missing issue, got: {stderr}"
    );
}

#[tokio::test]
async fn branch_read_hides_hidden_comments_by_default() {
    let repo = TestRepo::new_with_commit();

    // Create issue and branch
    repo.dg_command()
        .arg("issue")
        .arg("create")
        .arg("test issue")
        .output()
        .expect("issue create failed");

    repo.dg_command()
        .arg("branch")
        .arg("create")
        .arg("1")
        .arg("branch desc")
        .output()
        .expect("branch create failed");

    // Add two comments
    repo.dg_command()
        .arg("branch")
        .arg("comment")
        .arg("visible comment")
        .output()
        .expect("comment 1 failed");

    repo.dg_command()
        .arg("branch")
        .arg("comment")
        .arg("to hide")
        .output()
        .expect("comment 2 failed");

    // Get branch_id and hide second comment via DB
    {
        use doppelganger::db::{Database, branch, comment};
        let db_path = repo
            .path
            .join(".doppelganger.db")
            .to_str()
            .expect("db path")
            .to_string();
        let db = Database::open(&db_path).await.expect("open db");

        let git_repo = git2::Repository::open(&repo.path).expect("open repo");
        let head = git_repo.head().expect("head");
        let branch_name = head.shorthand().expect("shorthand").to_string();

        let br = branch::get_by_name(db.conn(), &branch_name)
            .await
            .expect("get branch");
        let comments = comment::list_branch_comments(db.conn(), br.branch_id, true)
            .await
            .expect("list comments");
        let to_hide = comments
            .iter()
            .find(|c| c.content == "to hide")
            .expect("find comment to hide");
        comment::set_branch_comment_hidden(db.conn(), to_hide.branch_comment_id, true)
            .await
            .expect("hide comment");
        db.checkpoint().await.expect("checkpoint");
    }

    // Default read: hidden excluded
    let output = repo
        .dg_command()
        .arg("branch")
        .arg("read")
        .output()
        .expect("branch read failed");
    assert!(
        output.status.success(),
        "branch read should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = std::str::from_utf8(&output.stdout).expect("valid utf8");
    let json: serde_json::Value = serde_json::from_str(stdout).expect("valid JSON");
    let comments = json
        .get("comments")
        .and_then(|v| v.as_array())
        .expect("comments");
    assert_eq!(comments.len(), 1, "should have only visible comment");
    assert_eq!(
        comments[0].get("content").and_then(|v| v.as_str()),
        Some("visible comment")
    );

    // --hidden read: both included
    let output = repo
        .dg_command()
        .arg("branch")
        .arg("read")
        .arg("--hidden")
        .output()
        .expect("branch read --hidden failed");
    assert!(
        output.status.success(),
        "branch read --hidden should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = std::str::from_utf8(&output.stdout).expect("valid utf8");
    let json: serde_json::Value = serde_json::from_str(stdout).expect("valid JSON");
    let comments = json
        .get("comments")
        .and_then(|v| v.as_array())
        .expect("comments");
    assert_eq!(comments.len(), 2, "should have both comments with --hidden");
}

#[tokio::test]
async fn branch_archive_succeeds() {
    let repo = TestRepo::new_with_commit();
    setup_with_issue(&repo);

    repo.dg_command()
        .arg("branch")
        .arg("create")
        .arg("1")
        .arg("my branch")
        .output()
        .expect("branch create failed");

    let output = repo
        .dg_command()
        .arg("branch")
        .arg("archive")
        .output()
        .expect("command failed to execute");
    assert!(
        output.status.success(),
        "archive should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = std::str::from_utf8(&output.stdout).expect("valid utf8");
    let json: serde_json::Value = serde_json::from_str(stdout).expect("valid JSON");
    assert_eq!(
        json.get("description").and_then(|v| v.as_str()),
        Some("my branch")
    );
    assert!(
        json.get("archived_at").and_then(|v| v.as_str()).is_some(),
        "archived_at should be set in JSON output"
    );
}

#[tokio::test]
async fn branch_archive_no_branch_fails() {
    let repo = TestRepo::new_with_commit();

    let output = repo
        .dg_command()
        .arg("branch")
        .arg("archive")
        .output()
        .expect("command failed to execute");
    assert!(
        !output.status.success(),
        "archive should fail when no branch record exists: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("branch create"),
        "stderr should prompt the user to create a branch, got: {stderr}"
    );
}

#[tokio::test]
async fn branch_read_after_archive_fails() {
    let repo = TestRepo::new_with_commit();
    setup_with_issue(&repo);

    repo.dg_command()
        .arg("branch")
        .arg("create")
        .arg("1")
        .arg("my branch")
        .output()
        .expect("branch create failed");

    repo.dg_command()
        .arg("branch")
        .arg("archive")
        .output()
        .expect("archive failed");

    let output = repo
        .dg_command()
        .arg("branch")
        .arg("read")
        .output()
        .expect("command failed to execute");
    assert!(
        !output.status.success(),
        "read should fail after archive: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("branch create"),
        "stderr should prompt the user to create a branch, got: {stderr}"
    );
}

#[tokio::test]
async fn branch_comment_after_archive_fails() {
    let repo = TestRepo::new_with_commit();
    setup_with_issue(&repo);

    repo.dg_command()
        .arg("branch")
        .arg("create")
        .arg("1")
        .arg("my branch")
        .output()
        .expect("branch create failed");

    repo.dg_command()
        .arg("branch")
        .arg("archive")
        .output()
        .expect("archive failed");

    let output = repo
        .dg_command()
        .arg("branch")
        .arg("comment")
        .arg("hello")
        .output()
        .expect("command failed to execute");
    assert!(
        !output.status.success(),
        "comment should fail after archive: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("branch create"),
        "stderr should prompt the user to create a branch, got: {stderr}"
    );
}

#[tokio::test]
async fn branch_read_shows_archived_at_field() {
    let repo = TestRepo::new_with_commit();
    setup_with_issue(&repo);

    repo.dg_command()
        .arg("branch")
        .arg("create")
        .arg("1")
        .arg("my branch")
        .output()
        .expect("branch create failed");

    let output = repo
        .dg_command()
        .arg("branch")
        .arg("read")
        .output()
        .expect("branch read failed");
    assert!(output.status.success());

    let stdout = std::str::from_utf8(&output.stdout).expect("valid utf8");
    let json: serde_json::Value = serde_json::from_str(stdout).expect("valid JSON");
    let branch = json.get("branch").expect("branch field");
    assert!(
        branch.get("archived_at").is_some(),
        "branch JSON should have archived_at field"
    );
    assert!(
        branch.get("archived_at").and_then(|v| v.as_str()).is_none(),
        "archived_at should be null for active branch"
    );
}
