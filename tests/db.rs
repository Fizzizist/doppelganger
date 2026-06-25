mod common;

use doppelganger::db::{Database, author, branch, comment, issue};
use turso::Value;

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

    let desc = "A description without a name";
    let created = issue::create(conn, None, desc, author.author_id, None)
        .await
        .expect("create issue without name");

    assert_eq!(created.name, None, "name should be None when not provided");
    assert_eq!(created.description, desc);
    assert_eq!(created.author, author.name);

    let created_named = issue::create(conn, Some("My Issue"), desc, author.author_id, None)
        .await
        .expect("create issue with name");

    assert_eq!(created_named.name.as_deref(), Some("My Issue"));
    assert_eq!(created_named.description, desc);

    let fetched = issue::get_by_id(conn, created.issue_id)
        .await
        .expect("get issue by id");
    assert_eq!(fetched.issue_id, created.issue_id);
    assert_eq!(fetched.name, None);
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
    let iss = issue::create(conn, None, "Fix bug", author.author_id, None)
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
    let iss = issue::create(conn, None, "Fix bug", author.author_id, None)
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
    let iss = issue::create(conn, None, "Fix bug", author.author_id, None)
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
    let iss = issue::create(conn, None, "Issue", author.author_id, None)
        .await
        .expect("create issue");

    comment::create_issue_comment(conn, iss.issue_id, "first comment", author.author_id)
        .await
        .expect("create first comment");
    comment::create_issue_comment(conn, iss.issue_id, "second comment", author.author_id)
        .await
        .expect("create second comment");

    let comments = comment::list_issue_comments(conn, iss.issue_id, true)
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
    let iss = issue::create(conn, None, "Issue", author.author_id, None)
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

    let comments = comment::list_branch_comments(conn, br.branch_id, true)
        .await
        .expect("list comments");

    assert_eq!(comments.len(), 2);
    assert_eq!(comments[0].content, "comment one");
    assert_eq!(comments[1].content, "comment two");
}

#[tokio::test]
async fn fk_branch_with_invalid_issue_is_rejected() {
    // Database::open enables `PRAGMA foreign_keys = ON`, so inserting a branch
    // that references a nonexistent issue must be rejected by the DB layer.
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();

    let author = author::find_or_create(conn, "Dev", Some("dev@x.com"))
        .await
        .expect("create author");

    // issue_id 99999 does not exist — FK enforcement should reject the insert.
    let result = branch::create(conn, "feature", "desc", author.author_id, 99999).await;
    assert!(
        result.is_err(),
        "foreign key enforcement must reject a branch referencing a missing issue"
    );
}

#[tokio::test]
async fn checkpoint_truncates_wal_and_persists_data() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let db_path = tmp.path().join("test.db");
    let wal_path = tmp.path().join("test.db-wal");
    let db_path_str = db_path.to_str().expect("valid path");

    let issue_id = {
        let db = Database::open(db_path_str).await.expect("open local db");
        let conn = db.conn();

        let author = author::find_or_create(conn, "Checker", Some("c@d.com"))
            .await
            .expect("create author");
        let created = issue::create(conn, None, "checkpoint issue", author.author_id, None)
            .await
            .expect("create issue");

        // The WAL should hold the just-written, uncheckpointed data.
        let wal_before = std::fs::metadata(&wal_path).map(|m| m.len()).unwrap_or(0);
        assert!(wal_before > 0, "WAL should be non-empty before checkpoint");

        db.checkpoint().await.expect("checkpoint should succeed");

        let wal_after = std::fs::metadata(&wal_path).map(|m| m.len()).unwrap_or(0);
        assert!(
            wal_after < wal_before,
            "TRUNCATE checkpoint should shrink the WAL (before: {wal_before}, after: {wal_after})"
        );

        created.issue_id
    };

    // Reopen the database and confirm the data persisted past checkpoint + close.
    let db = Database::open(db_path_str).await.expect("reopen local db");
    let fetched = issue::get_by_id(db.conn(), issue_id)
        .await
        .expect("issue should persist after checkpoint and reopen");
    assert_eq!(fetched.description, "checkpoint issue");
}

#[tokio::test]
async fn issue_list_returns_empty() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();
    let issues = issue::list(conn).await.expect("list issues");
    assert!(issues.is_empty(), "should return empty vec for no issues");
}

#[tokio::test]
async fn issue_list_returns_ordered_by_updated_at_desc() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();
    let author = author::find_or_create(conn, "Lister", Some("l@b.com"))
        .await
        .expect("create author");

    let first = issue::create(conn, None, "first issue", author.author_id, None)
        .await
        .expect("create first");
    let second = issue::create(conn, None, "second issue", author.author_id, None)
        .await
        .expect("create second");
    let third = issue::create(conn, None, "third issue", author.author_id, None)
        .await
        .expect("create third");

    let issues = issue::list(conn).await.expect("list issues");
    assert_eq!(issues.len(), 3);
    assert_eq!(issues[0].issue_id, third.issue_id);
    assert_eq!(issues[1].issue_id, second.issue_id);
    assert_eq!(issues[2].issue_id, first.issue_id);
}

#[tokio::test]
async fn fingerprint_differs_after_insert() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();
    let author = author::find_or_create(conn, "Checker", Some("c@b.com"))
        .await
        .expect("create author");

    let initial_issues = issue::list(conn).await.expect("list initial");
    let fp1: String = initial_issues
        .iter()
        .map(|i| format!("{}:{}", i.issue_id, i.updated_at))
        .collect::<Vec<_>>()
        .join(",");

    issue::create(conn, None, "new issue", author.author_id, None)
        .await
        .expect("create issue");

    let after_issues = issue::list(conn).await.expect("list after insert");
    let fp2: String = after_issues
        .iter()
        .map(|i| format!("{}:{}", i.issue_id, i.updated_at))
        .collect::<Vec<_>>()
        .join(",");

    assert_ne!(fp1, fp2, "fingerprint should differ after insert");
}

#[tokio::test]
async fn issue_create_with_remote_id() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();
    let author = author::find_or_create(conn, "Author", Some("a@b.com"))
        .await
        .expect("create author");
    let created = issue::create(
        conn,
        Some("Remote Issue"),
        "desc",
        author.author_id,
        Some("gh:owner/repo#42"),
    )
    .await
    .expect("create issue with remote_id");
    assert_eq!(created.remote_id, Some("gh:owner/repo#42".to_string()));
}

#[tokio::test]
async fn issue_create_without_remote_id() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();
    let author = author::find_or_create(conn, "Author", Some("a@b.com"))
        .await
        .expect("create author");
    let created = issue::create(conn, None, "local issue", author.author_id, None)
        .await
        .expect("create issue without remote_id");
    assert_eq!(created.remote_id, None);
}

#[tokio::test]
async fn issue_get_by_remote_id() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();
    let author = author::find_or_create(conn, "Author", Some("a@b.com"))
        .await
        .expect("create author");
    issue::create(
        conn,
        Some("Remote"),
        "desc",
        author.author_id,
        Some("gh:owner/repo#7"),
    )
    .await
    .expect("create issue");
    let found = issue::get_by_remote_id(conn, "gh:owner/repo#7")
        .await
        .expect("get by remote_id");
    assert_eq!(found.name.as_deref(), Some("Remote"));
    assert_eq!(found.remote_id, Some("gh:owner/repo#7".to_string()));
}

#[tokio::test]
async fn issue_get_by_remote_id_not_found() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();
    let result = issue::get_by_remote_id(conn, "gh:nonexistent/repo#999").await;
    assert!(result.is_err(), "should error for nonexistent remote_id");
}

#[tokio::test]
async fn issue_update_for_sync() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();
    let author = author::find_or_create(conn, "Original", Some("a@b.com"))
        .await
        .expect("create author");
    let created = issue::create(
        conn,
        Some("Original Title"),
        "original body",
        author.author_id,
        None,
    )
    .await
    .expect("create issue");

    let new_author = author::find_or_create(conn, "SyncAuthor", None)
        .await
        .expect("create sync author");
    let updated = issue::update_for_sync(
        conn,
        created.issue_id,
        Some("Synced Title"),
        "synced body",
        new_author.author_id,
        Some("gh:owner/repo#1"),
    )
    .await
    .expect("update for sync");

    assert_eq!(updated.name.as_deref(), Some("Synced Title"));
    assert_eq!(updated.description, "synced body");
    assert_eq!(updated.author, "SyncAuthor");
    assert_eq!(updated.remote_id, Some("gh:owner/repo#1".to_string()));
}

#[tokio::test]
async fn migration_adds_remote_id_to_existing_db() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let db_path = tmp.path().join("legacy.db");
    let db_path_str = db_path.to_str().expect("valid path");

    {
        let db = Database::open(db_path_str).await.expect("open db");
        let conn = db.conn();
        let author = author::find_or_create(conn, "LegacyTest", None)
            .await
            .expect("create author");
        let created = issue::create(conn, Some("Legacy Issue"), "body", author.author_id, None)
            .await
            .expect("create issue");
        assert_eq!(created.remote_id, None);
        db.checkpoint().await.expect("checkpoint");
    }

    {
        let db = Database::open(db_path_str).await.expect("reopen db");
        let conn = db.conn();
        let author = author::find_or_create(conn, "LegacyTest", None)
            .await
            .expect("find author");
        let created = issue::create(
            conn,
            Some("With Remote"),
            "body",
            author.author_id,
            Some("gh:owner/repo#1"),
        )
        .await
        .expect("create issue with remote_id");
        assert_eq!(created.remote_id, Some("gh:owner/repo#1".to_string()));
    }
}

#[tokio::test]
async fn migration_remote_id_is_idempotent() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    db.migrate().await.expect("second migrate should succeed");
}

#[tokio::test]
async fn migrate_adds_branch_archived_at() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();

    let mut rows = conn
        .query("PRAGMA table_info(branch)", ())
        .await
        .expect("pragma query");
    let mut found = false;
    while let Some(row) = rows.next().await.expect("next row") {
        let name = row.get_value(1).expect("column name");
        if let Value::Text(s) = name {
            if s == "archived_at" {
                found = true;
            }
        }
    }
    assert!(found, "branch table should have archived_at column");
}

#[tokio::test]
async fn migrate_creates_branch_active_index() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();

    let mut rows = conn
        .query(
            "SELECT name FROM sqlite_master WHERE type='index' AND name='idx_branch_active_name'",
            (),
        )
        .await
        .expect("query indexes");
    let mut found = false;
    while let Some(row) = rows.next().await.expect("next row") {
        let name = row.get_value(0).expect("index name");
        if let Value::Text(s) = name {
            if s == "idx_branch_active_name" {
                found = true;
            }
        }
    }
    assert!(found, "should have idx_branch_active_name index");
}

#[tokio::test]
async fn migrate_drops_unique_constraint_from_legacy_branch_table() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let db_path = tmp.path().join("legacy_unique.db");
    let db_path_str = db_path.to_str().expect("valid path");

    {
        let db = Database::open(db_path_str).await.expect("open db");
        let conn = db.conn();
        let author = author::find_or_create(conn, "Legacy", None)
            .await
            .expect("author");
        let iss = issue::create(conn, None, "desc", author.author_id, None)
            .await
            .expect("issue");
        branch::create(conn, "old-name", "desc", author.author_id, iss.issue_id)
            .await
            .expect("branch");
        db.checkpoint().await.expect("checkpoint");
    }

    // Simulate a legacy DB by recreating the branch table with the old UNIQUE constraint
    {
        let db = Database::open(db_path_str).await.expect("reopen db");
        let conn = db.conn();
        conn.execute("ALTER TABLE branch RENAME TO branch_old", ())
            .await
            .expect("rename old branch");
        conn.execute(
            "CREATE TABLE branch (\
             branch_id INTEGER PRIMARY KEY AUTOINCREMENT,\
             name TEXT NOT NULL UNIQUE,\
             description TEXT NOT NULL,\
             author_id INTEGER NOT NULL REFERENCES author(author_id),\
             issue_id INTEGER NOT NULL REFERENCES issue(issue_id),\
             created_at TEXT NOT NULL DEFAULT (datetime('now')),\
             updated_at TEXT NOT NULL DEFAULT (datetime('now'))\
             )",
            (),
        )
        .await
        .expect("create legacy branch table");
        conn.execute(
            "INSERT INTO branch (branch_id, name, description, author_id, issue_id, created_at, updated_at) \
             SELECT branch_id, name, description, author_id, issue_id, created_at, updated_at FROM branch_old",
            (),
        )
        .await
        .expect("copy data");
        conn.execute("DROP TABLE branch_old", ())
            .await
            .expect("drop old");
        db.checkpoint().await.expect("checkpoint");
    }

    // Now re-open — migration should detect the UNIQUE constraint and recreate the table
    let db = Database::open(db_path_str)
        .await
        .expect("reopen after migration");
    let conn = db.conn();

    // Verify the table no longer has UNIQUE on name
    let mut rows = conn
        .query(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='branch'",
            (),
        )
        .await
        .expect("query table def");
    let table_sql = match rows.next().await.expect("next row") {
        Some(row) => {
            let val = row.get_value(0).expect("sql value");
            match val {
                Value::Text(s) => s.to_lowercase(),
                _ => panic!("expected text for table sql"),
            }
        }
        None => panic!("branch table should exist"),
    };
    assert!(
        !table_sql.contains("name text not null unique"),
        "branch table should no longer have UNIQUE constraint on name, got: {table_sql}"
    );

    // Verify archived_at column exists (migration added it after table recreation)
    let br = branch::get_by_name(conn, "old-name")
        .await
        .expect("get branch");
    assert!(
        br.archived_at.is_none(),
        "existing branch should have null archived_at"
    );
}
