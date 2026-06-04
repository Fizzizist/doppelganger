mod common;

use doppelganger::db::{Database, author, branch, comment, issue};

#[tokio::test]
async fn migrate_is_idempotent() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    // migrate() is already called during open_in_memory(), call it again
    db.migrate().await.expect("second migrate should succeed");
}

#[tokio::test]
async fn author_find_or_create_with_email() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();

    let a1 = author::find_or_create(conn, "Alice", Some("alice@example.com"))
        .await
        .expect("create author");
    assert_eq!(a1.name, "Alice");
    assert_eq!(a1.email, Some("alice@example.com".to_string()));

    let a2 = author::find_or_create(conn, "Alice", Some("alice@example.com"))
        .await
        .expect("find existing author");
    assert_eq!(
        a1.author_id, a2.author_id,
        "same name/email should return same author"
    );
}

#[tokio::test]
async fn author_find_or_create_nullable_email() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();

    let a = author::find_or_create(conn, "Bob", None)
        .await
        .expect("create author without email");
    assert_eq!(a.name, "Bob");
    assert!(a.email.is_none(), "email should be None when not provided");
}

#[tokio::test]
async fn issue_create_and_get() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();

    let author = author::find_or_create(conn, "Author", Some("a@b.com"))
        .await
        .expect("create author");

    let long_desc = "This is a very long description that is definitely more than eighty characters long and should be truncated for the name field but kept in full for description";
    let created = issue::create(conn, long_desc, author.author_id)
        .await
        .expect("create issue");

    assert!(
        created.name.len() <= 80,
        "name should be truncated to 80 chars"
    );
    assert_eq!(
        created.description, long_desc,
        "description should be the full text"
    );
    assert_eq!(created.author, author.name);

    let fetched = issue::get_by_id(conn, created.issue_id)
        .await
        .expect("get issue by id");
    assert_eq!(fetched.issue_id, created.issue_id);
}

#[tokio::test]
async fn issue_get_not_found() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();

    let result = issue::get_by_id(conn, 999).await;
    assert!(result.is_err(), "getting nonexistent issue should error");
    match result.unwrap_err() {
        doppelganger::error::Error::IssueNotFound(id) => {
            assert_eq!(id, 999);
        }
        other => panic!("expected IssueNotFound, got {other:?}"),
    }
}

#[tokio::test]
async fn branch_create_and_get_by_name() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();

    let author = author::find_or_create(conn, "Dev", Some("dev@x.com"))
        .await
        .expect("create author");
    let iss = issue::create(conn, "Fix bug", author.author_id)
        .await
        .expect("create issue");

    let br = branch::create(conn, "feature-1", "desc", author.author_id, iss.issue_id)
        .await
        .expect("create branch");

    assert_eq!(br.name, "feature-1");
    assert_eq!(br.issue_id, iss.issue_id);

    let fetched = branch::get_by_name(conn, "feature-1")
        .await
        .expect("get branch by name");
    assert_eq!(fetched.branch_id, br.branch_id);
}

#[tokio::test]
async fn branch_create_duplicate_error() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();

    let author = author::find_or_create(conn, "Dev", Some("dev@x.com"))
        .await
        .expect("create author");
    let iss = issue::create(conn, "Fix bug", author.author_id)
        .await
        .expect("create issue");

    branch::create(conn, "feature-1", "first", author.author_id, iss.issue_id)
        .await
        .expect("first create succeeds");

    let result = branch::create(conn, "feature-1", "second", author.author_id, iss.issue_id).await;
    assert!(result.is_err(), "duplicate branch should fail");
    match result.unwrap_err() {
        doppelganger::error::Error::BranchAlreadyExists(name) => {
            assert_eq!(name, "feature-1");
        }
        other => panic!("expected BranchAlreadyExists, got {other:?}"),
    }
}

#[tokio::test]
async fn branch_overwrite_via_update() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();

    let author = author::find_or_create(conn, "Dev", Some("dev@x.com"))
        .await
        .expect("create author");
    let iss = issue::create(conn, "Fix bug", author.author_id)
        .await
        .expect("create issue");

    let br = branch::create(
        conn,
        "feature-1",
        "original desc",
        author.author_id,
        iss.issue_id,
    )
    .await
    .expect("create branch");
    assert_eq!(br.description, "original desc");

    let updated = branch::update_description(conn, "feature-1", "updated desc")
        .await
        .expect("update description");
    assert_eq!(updated.description, "updated desc");
}

#[tokio::test]
async fn branch_get_not_found() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();

    let result = branch::get_by_name(conn, "nonexistent").await;
    assert!(result.is_err(), "getting nonexistent branch should fail");
    match result.unwrap_err() {
        doppelganger::error::Error::BranchNotFound(name) => {
            assert_eq!(name, "nonexistent");
        }
        other => panic!("expected BranchNotFound, got {other:?}"),
    }
}

#[tokio::test]
async fn issue_comments_create_and_list() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();

    let author = author::find_or_create(conn, "Alice", Some("a@b.com"))
        .await
        .expect("create author");
    let iss = issue::create(conn, "Issue", author.author_id)
        .await
        .expect("create issue");

    comment::create_issue_comment(conn, iss.issue_id, "first comment", author.author_id)
        .await
        .expect("create first comment");
    comment::create_issue_comment(conn, iss.issue_id, "second comment", author.author_id)
        .await
        .expect("create second comment");

    let comments = comment::list_issue_comments(conn, iss.issue_id)
        .await
        .expect("list comments");

    assert_eq!(comments.len(), 2);
    assert_eq!(comments[0].content, "first comment");
    assert_eq!(comments[1].content, "second comment");
}

#[tokio::test]
async fn branch_comments_create_and_list() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();

    let author = author::find_or_create(conn, "Alice", Some("a@b.com"))
        .await
        .expect("create author");
    let iss = issue::create(conn, "Issue", author.author_id)
        .await
        .expect("create issue");
    let br = branch::create(conn, "feature", "desc", author.author_id, iss.issue_id)
        .await
        .expect("create branch");

    comment::create_branch_comment(conn, br.branch_id, "comment one", author.author_id)
        .await
        .expect("create comment 1");
    comment::create_branch_comment(conn, br.branch_id, "comment two", author.author_id)
        .await
        .expect("create comment 2");

    let comments = comment::list_branch_comments(conn, br.branch_id)
        .await
        .expect("list comments");

    assert_eq!(comments.len(), 2);
    assert_eq!(comments[0].content, "comment one");
    assert_eq!(comments[1].content, "comment two");
}

#[tokio::test]
async fn fk_branch_with_invalid_issue() {
    // turso (libsql/SQLite) does not enforce FK constraints by default.
    // The application-level validation happens in branch::create's caller
    // (commands/branch.rs) which checks the issue exists before creating a branch.
    // This test documents that the DB layer alone does NOT reject invalid issue_ids.
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();

    let author = author::find_or_create(conn, "Dev", Some("dev@x.com"))
        .await
        .expect("create author");

    // issue_id 99999 does not exist — insert still succeeds because FKs are not enforced
    let result = branch::create(conn, "feature", "desc", author.author_id, 99999).await;
    assert!(
        result.is_ok(),
        "turso does not enforce FK by default; the application layer validates issue existence"
    );
}

#[tokio::test]
async fn checkpoint_runs_without_error() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let db_path = tmp.path().join("test.db");
    let db_path_str = db_path.to_str().expect("valid path");

    let db = Database::open(db_path_str).await.expect("open local db");
    let conn = db.conn();

    // Write some data
    let author = author::find_or_create(conn, "Checker", Some("c@d.com"))
        .await
        .expect("create author");
    issue::create(conn, "checkpoint issue", author.author_id)
        .await
        .expect("create issue");

    db.checkpoint().await.expect("checkpoint should succeed");
}
