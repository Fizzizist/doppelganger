use doppelganger::db::{Database, author, branch, comment, issue};

#[tokio::test]
async fn test_schema_creates_all_tables() {
    let db = Database::open_in_memory().await.expect("open db");
    let conn = db.connect().expect("connect");

    let mut rows = conn
        .query(
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name",
            (),
        )
        .await
        .expect("query tables");

    let mut tables = Vec::new();
    while let Some(row) = rows.next().await.expect("next row") {
        let val = row.get_value(0).expect("get value");
        tables.push(val);
    }

    assert!(
        tables
            .iter()
            .any(|t| matches!(t, turso::Value::Text(s) if s == "author"))
    );
    assert!(
        tables
            .iter()
            .any(|t| matches!(t, turso::Value::Text(s) if s == "issue"))
    );
    assert!(
        tables
            .iter()
            .any(|t| matches!(t, turso::Value::Text(s) if s == "branch"))
    );
    assert!(
        tables
            .iter()
            .any(|t| matches!(t, turso::Value::Text(s) if s == "issue_comment"))
    );
    assert!(
        tables
            .iter()
            .any(|t| matches!(t, turso::Value::Text(s) if s == "branch_comment"))
    );
}

#[tokio::test]
async fn test_author_crud() {
    let db = Database::open_in_memory().await.expect("open db");
    let conn = db.connect().expect("connect");

    let a1 = author::create(&conn, "Alice", Some("alice@example.com"))
        .await
        .expect("create author");
    assert_eq!(a1.name, "Alice");
    assert_eq!(a1.email, Some("alice@example.com".to_string()));

    let a2 = author::find_or_create(&conn, "Alice", Some("alice@example.com"))
        .await
        .expect("find_or_create");
    assert_eq!(a1.author_id, a2.author_id);

    let a3 = author::find_or_create(&conn, "Alice", None)
        .await
        .expect("find_or_create different email");
    assert_ne!(a1.author_id, a3.author_id);
}

#[tokio::test]
async fn test_issue_crud() {
    let db = Database::open_in_memory().await.expect("open db");
    let conn = db.connect().expect("connect");

    let a = author::create(&conn, "Author", None)
        .await
        .expect("create author");
    let issue = issue::create(&conn, "Bug report", "Something is broken", a.author_id)
        .await
        .expect("create issue");
    assert_eq!(issue.name, "Bug report");

    let fetched = issue::get_by_id(&conn, issue.issue_id)
        .await
        .expect("get_by_id")
        .expect("found");
    assert_eq!(fetched.name, "Bug report");
    assert_eq!(fetched.description, "Something is broken");

    let missing = issue::get_by_id(&conn, 99999).await.expect("get_by_id");
    assert!(missing.is_none());
}

#[tokio::test]
async fn test_branch_crud() {
    let db = Database::open_in_memory().await.expect("open db");
    let conn = db.connect().expect("connect");

    let a = author::create(&conn, "Author", None)
        .await
        .expect("create author");
    let iss = issue::create(&conn, "Issue", "desc", a.author_id)
        .await
        .expect("create issue");

    let br = branch::create(&conn, "feature-x", "New feature", a.author_id, iss.issue_id)
        .await
        .expect("create branch");
    assert_eq!(br.name, "feature-x");

    let fetched = branch::get_by_name(&conn, "feature-x")
        .await
        .expect("get_by_name")
        .expect("found");
    assert_eq!(fetched.branch_id, br.branch_id);
}

#[tokio::test]
async fn test_comment_crud() {
    let db = Database::open_in_memory().await.expect("open db");
    let conn = db.connect().expect("connect");

    let a = author::create(&conn, "Author", None)
        .await
        .expect("create author");
    let iss = issue::create(&conn, "Issue", "desc", a.author_id)
        .await
        .expect("create issue");
    let br = branch::create(&conn, "my-branch", "desc", a.author_id, iss.issue_id)
        .await
        .expect("create branch");

    let ic = comment::create_issue_comment(&conn, "issue comment text", a.author_id, iss.issue_id)
        .await
        .expect("create issue comment");
    assert_eq!(ic.content, "issue comment text");
    assert_eq!(ic.issue_id, iss.issue_id);

    let bc =
        comment::create_branch_comment(&conn, "branch comment text", a.author_id, br.branch_id)
            .await
            .expect("create branch comment");
    assert_eq!(bc.content, "branch comment text");
    assert_eq!(bc.branch_id, br.branch_id);

    let issue_comments = comment::list_issue_comments(&conn, iss.issue_id)
        .await
        .expect("list issue comments");
    assert_eq!(issue_comments.len(), 1);

    let branch_comments = comment::list_branch_comments(&conn, br.branch_id)
        .await
        .expect("list branch comments");
    assert_eq!(branch_comments.len(), 1);
}
