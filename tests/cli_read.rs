use common::{dg_command_in_repo, setup_repo};

mod common;

#[test]
fn test_read_issue() {
    let (tmp, _repo) = setup_repo();

    let issue_output = dg_command_in_repo(tmp.path())
        .args([
            "create",
            "issue",
            "--name",
            "Readable Issue",
            "--description",
            "Can read this",
        ])
        .output()
        .expect("run dg create issue");
    assert!(issue_output.status.success());

    let issue: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&issue_output.stdout)).expect("valid JSON");
    let issue_id = issue["issue_id"].as_i64().expect("issue_id");

    let read_output = dg_command_in_repo(tmp.path())
        .args(["read", "issue", "single", &issue_id.to_string()])
        .output()
        .expect("run dg read issue");

    assert!(
        read_output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&read_output.stderr)
    );

    let read_issue: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&read_output.stdout)).expect("valid JSON");
    assert_eq!(read_issue["issue_id"], issue_id);
    assert_eq!(read_issue["name"], "Readable Issue");
}

#[test]
fn test_read_issue_thread() {
    let (tmp, _repo) = setup_repo();

    let issue_output = dg_command_in_repo(tmp.path())
        .args([
            "create",
            "issue",
            "--name",
            "Threaded Issue",
            "--description",
            "Thread test",
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
            "Thread comment",
        ])
        .output()
        .expect("run dg create comment issue");
    assert!(comment_output.status.success());

    let thread_output = dg_command_in_repo(tmp.path())
        .args(["read", "issue", "thread", &issue_id.to_string()])
        .output()
        .expect("run dg read issue thread");

    assert!(
        thread_output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&thread_output.stderr)
    );

    let thread: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&thread_output.stdout)).expect("valid JSON");
    assert_eq!(thread["issue"]["issue_id"], issue_id);
    assert_eq!(
        thread["comments"].as_array().expect("comments array").len(),
        1
    );
}

#[test]
fn test_read_branch() {
    let (tmp, _repo) = setup_repo();

    let issue_output = dg_command_in_repo(tmp.path())
        .args([
            "create",
            "issue",
            "--name",
            "Branch Issue",
            "--description",
            "Branch test",
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
            "read-branch",
            "--description",
            "Branch desc",
        ])
        .output()
        .expect("run dg create branch");
    assert!(branch_output.status.success());

    let read_output = dg_command_in_repo(tmp.path())
        .args(["read", "branch", "single"])
        .output()
        .expect("run dg read branch");

    assert!(
        read_output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&read_output.stderr)
    );

    let read_branch: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&read_output.stdout)).expect("valid JSON");
    assert_eq!(read_branch["name"], "read-branch");
}

#[test]
fn test_read_branch_thread() {
    let (tmp, _repo) = setup_repo();

    let issue_output = dg_command_in_repo(tmp.path())
        .args([
            "create",
            "issue",
            "--name",
            "Thread Branch Issue",
            "--description",
            "Thread",
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
            "thread-branch",
            "--description",
            "Branch thread",
        ])
        .output()
        .expect("run dg create branch");
    assert!(branch_output.status.success());

    let comment_output = dg_command_in_repo(tmp.path())
        .args([
            "create",
            "comment",
            "branch",
            "--content",
            "Branch thread comment",
        ])
        .output()
        .expect("run dg create comment branch");
    assert!(comment_output.status.success());

    let thread_output = dg_command_in_repo(tmp.path())
        .args(["read", "branch", "thread"])
        .output()
        .expect("run dg read branch thread");

    assert!(
        thread_output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&thread_output.stderr)
    );

    let thread: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&thread_output.stdout)).expect("valid JSON");
    assert_eq!(thread["branch"]["name"], "thread-branch");
    assert_eq!(
        thread["comments"].as_array().expect("comments array").len(),
        1
    );
}
