use common::{dg_command_in_repo, setup_repo};

mod common;

#[test]
fn test_create_issue_with_description_flag() {
    let (tmp, _repo) = setup_repo();
    let output = dg_command_in_repo(tmp.path())
        .args([
            "create",
            "issue",
            "--name",
            "Test Issue",
            "--description",
            "A test description",
        ])
        .output()
        .expect("run dg create issue");

    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON output");
    assert_eq!(parsed["name"], "Test Issue");
    assert_eq!(parsed["description"], "A test description");
}

#[test]
fn test_create_branch() {
    let (tmp, _repo) = setup_repo();

    let issue_output = dg_command_in_repo(tmp.path())
        .args(["create", "issue", "--name", "Bug", "--description", "A bug"])
        .output()
        .expect("run dg create issue");

    assert!(issue_output.status.success());
    let issue: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&issue_output.stdout)).expect("valid JSON");
    let issue_id = issue["issue_id"].as_i64().expect("issue_id");

    let branch_output = dg_command_in_repo(tmp.path())
        .args([
            "create",
            "branch",
            &issue_id.to_string(),
            "--name",
            "fix-bug",
            "--description",
            "Fix the bug",
        ])
        .output()
        .expect("run dg create branch");

    assert!(
        branch_output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&branch_output.stderr)
    );

    let branch: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&branch_output.stdout)).expect("valid JSON");
    assert_eq!(branch["name"], "fix-bug");
    assert_eq!(branch["issue_id"], issue_id);
}

#[test]
fn test_create_issue_comment() {
    let (tmp, _repo) = setup_repo();

    let issue_output = dg_command_in_repo(tmp.path())
        .args([
            "create",
            "issue",
            "--name",
            "Issue1",
            "--description",
            "Desc",
        ])
        .output()
        .expect("run dg create issue");
    assert!(issue_output.status.success());

    let issue: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&issue_output.stdout)).expect("valid JSON");
    let issue_id = issue["issue_id"].as_i64().expect("issue_id");

    let comment_output = dg_command_in_repo(tmp.path())
        .args([
            "create",
            "comment",
            "issue",
            &issue_id.to_string(),
            "--content",
            "First comment",
        ])
        .output()
        .expect("run dg create comment issue");

    assert!(
        comment_output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&comment_output.stderr)
    );

    let comment: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&comment_output.stdout)).expect("valid JSON");
    assert_eq!(comment["content"], "First comment");
    assert_eq!(comment["issue_id"], issue_id);
}

#[test]
fn test_create_branch_comment() {
    let (tmp, _repo) = setup_repo();

    let issue_output = dg_command_in_repo(tmp.path())
        .args([
            "create",
            "issue",
            "--name",
            "Issue1",
            "--description",
            "Desc",
        ])
        .output()
        .expect("run dg create issue");
    assert!(issue_output.status.success());

    let issue: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&issue_output.stdout)).expect("valid JSON");
    let issue_id = issue["issue_id"].as_i64().expect("issue_id");

    let branch_output = dg_command_in_repo(tmp.path())
        .args([
            "create",
            "branch",
            &issue_id.to_string(),
            "--name",
            "fix-issue",
            "--description",
            "Working on fix",
        ])
        .output()
        .expect("run dg create branch");
    assert!(branch_output.status.success());

    let comment_output = dg_command_in_repo(tmp.path())
        .args(["create", "comment", "branch", "--content", "Branch comment"])
        .output()
        .expect("run dg create comment branch");

    assert!(
        comment_output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&comment_output.stderr)
    );

    let comment: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&comment_output.stdout)).expect("valid JSON");
    assert_eq!(comment["content"], "Branch comment");
}
