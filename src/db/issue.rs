use crate::error::{Error, Result};
use turso::Value;

use super::{
    models::Issue,
    row::{extract_int, extract_optional_text, extract_text},
};

pub async fn create(
    conn: &turso::Connection,
    name: Option<&str>,
    description: &str,
    author_id: i64,
    remote_id: Option<&str>,
) -> Result<Issue> {
    let name_value = match name {
        Some(n) => Value::Text(n.to_string()),
        None => Value::Null,
    };
    let remote_id_value = match remote_id {
        Some(r) => Value::Text(r.to_string()),
        None => Value::Null,
    };

    conn.execute(
        "INSERT INTO issue (name, description, author_id, remote_id) VALUES (?1, ?2, ?3, ?4)",
        turso::params::Params::Positional(vec![
            name_value,
            Value::Text(description.to_string()),
            Value::Integer(author_id),
            remote_id_value,
        ]),
    )
    .await?;

    let issue_id = conn.last_insert_rowid();
    get_by_id(conn, issue_id).await
}

pub async fn get_by_id(conn: &turso::Connection, issue_id: i64) -> Result<Issue> {
    let mut rows = conn
        .query(
            "SELECT issue.issue_id, issue.name, issue.description, author.name, \
             issue.created_at, issue.updated_at, issue.remote_id, issue.archived_at \
             FROM issue JOIN author ON issue.author_id = author.author_id \
             WHERE issue.issue_id = ?1",
            turso::params::Params::Positional(vec![Value::Integer(issue_id)]),
        )
        .await?;

    match rows.next().await? {
        Some(row) => row_to_issue(&row),
        None => Err(Error::IssueNotFound(issue_id)),
    }
}

pub async fn list(conn: &turso::Connection) -> Result<Vec<Issue>> {
    let mut rows = conn
        .query(
            "SELECT issue.issue_id, issue.name, issue.description, author.name, \
             issue.created_at, issue.updated_at, issue.remote_id, issue.archived_at \
             FROM issue JOIN author ON issue.author_id = author.author_id \
             WHERE issue.archived_at IS NULL \
             ORDER BY issue.updated_at DESC, issue.issue_id DESC",
            (),
        )
        .await?;

    let mut issues = Vec::new();
    while let Some(row) = rows.next().await? {
        issues.push(row_to_issue(&row)?);
    }
    Ok(issues)
}

pub async fn list_all(conn: &turso::Connection) -> Result<Vec<Issue>> {
    let mut rows = conn
        .query(
            "SELECT issue.issue_id, issue.name, issue.description, author.name, \
             issue.created_at, issue.updated_at, issue.remote_id, issue.archived_at \
             FROM issue JOIN author ON issue.author_id = author.author_id \
             ORDER BY issue.updated_at DESC, issue.issue_id DESC",
            (),
        )
        .await?;

    let mut issues = Vec::new();
    while let Some(row) = rows.next().await? {
        issues.push(row_to_issue(&row)?);
    }
    Ok(issues)
}

pub async fn set_archived(conn: &turso::Connection, issue_id: i64, archived: bool) -> Result<()> {
    let sql = if archived {
        "UPDATE issue SET archived_at = datetime('now'), updated_at = datetime('now') WHERE issue_id = ?1"
    } else {
        "UPDATE issue SET archived_at = NULL, updated_at = datetime('now') WHERE issue_id = ?1"
    };
    conn.execute(
        sql,
        turso::params::Params::Positional(vec![Value::Integer(issue_id)]),
    )
    .await?;
    Ok(())
}

pub async fn get_by_remote_id(conn: &turso::Connection, remote_id: &str) -> Result<Issue> {
    let mut rows = conn
        .query(
            "SELECT issue.issue_id, issue.name, issue.description, author.name, \
             issue.created_at, issue.updated_at, issue.remote_id, issue.archived_at \
             FROM issue JOIN author ON issue.author_id = author.author_id \
             WHERE issue.remote_id = ?1",
            turso::params::Params::Positional(vec![Value::Text(remote_id.to_string())]),
        )
        .await?;

    match rows.next().await? {
        Some(row) => row_to_issue(&row),
        None => Err(Error::RemoteSync(format!(
            "no issue with remote_id '{remote_id}'"
        ))),
    }
}

pub async fn update_for_sync(
    conn: &turso::Connection,
    issue_id: i64,
    name: Option<&str>,
    description: &str,
    author_id: i64,
    remote_id: Option<&str>,
) -> Result<Issue> {
    let name_value = match name {
        Some(n) => Value::Text(n.to_string()),
        None => Value::Null,
    };
    let remote_id_value = match remote_id {
        Some(r) => Value::Text(r.to_string()),
        None => Value::Null,
    };

    conn.execute(
        "UPDATE issue SET name = ?1, description = ?2, author_id = ?3, remote_id = ?4, \
         archived_at = NULL, updated_at = datetime('now') WHERE issue_id = ?5",
        turso::params::Params::Positional(vec![
            name_value,
            Value::Text(description.to_string()),
            Value::Integer(author_id),
            remote_id_value,
            Value::Integer(issue_id),
        ]),
    )
    .await?;

    get_by_id(conn, issue_id).await
}

pub async fn update_description(
    conn: &turso::Connection,
    issue_id: i64,
    description: &str,
) -> Result<Issue> {
    conn.execute(
        "UPDATE issue SET description = ?1, updated_at = datetime('now') \
         WHERE issue_id = ?2",
        turso::params::Params::Positional(vec![
            Value::Text(description.to_string()),
            Value::Integer(issue_id),
        ]),
    )
    .await?;

    get_by_id(conn, issue_id).await
}

fn row_to_issue(row: &turso::Row) -> Result<Issue> {
    Ok(Issue {
        issue_id: extract_int(row, 0)?,
        name: extract_optional_text(row, 1)?,
        description: extract_text(row, 2)?,
        author: extract_text(row, 3)?,
        created_at: extract_text(row, 4)?,
        updated_at: extract_text(row, 5)?,
        remote_id: extract_optional_text(row, 6)?,
        archived_at: extract_optional_text(row, 7)?,
    })
}

#[cfg(test)]
mod tests {
    use crate::db::{Database, author};

    #[tokio::test]
    async fn update_description_changes_only_description() {
        let db = Database::open_in_memory().await.expect("open in-memory db");
        let conn = db.conn();

        let author = author::find_or_create(conn, "Alice", Some("a@b.com"))
            .await
            .expect("create author");
        let original = super::create(
            conn,
            Some("test issue"),
            "original desc",
            author.author_id,
            Some("remote-123"),
        )
        .await
        .expect("create issue");

        let updated = super::update_description(conn, original.issue_id, "new desc")
            .await
            .expect("update description");

        assert_eq!(updated.description, "new desc");
        assert_eq!(updated.name, original.name);
        assert_eq!(updated.author, original.author);
        assert_eq!(updated.remote_id, original.remote_id);
        assert!(
            updated.updated_at >= original.updated_at,
            "updated_at must not go backwards"
        );
    }

    #[tokio::test]
    async fn set_archived_archives_issue() {
        let db = Database::open_in_memory().await.expect("open in-memory db");
        let conn = db.conn();
        let author = author::find_or_create(conn, "Alice", None)
            .await
            .expect("author");
        let issue = super::create(conn, Some("test"), "desc", author.author_id, None)
            .await
            .expect("create");
        assert!(issue.archived_at.is_none());

        super::set_archived(conn, issue.issue_id, true)
            .await
            .expect("archive");
        let fetched = super::get_by_id(conn, issue.issue_id).await.expect("get");
        assert!(fetched.archived_at.is_some());
    }

    #[tokio::test]
    async fn set_archived_unarchives_issue() {
        let db = Database::open_in_memory().await.expect("open in-memory db");
        let conn = db.conn();
        let author = author::find_or_create(conn, "Alice", None)
            .await
            .expect("author");
        let issue = super::create(conn, Some("test"), "desc", author.author_id, None)
            .await
            .expect("create");

        super::set_archived(conn, issue.issue_id, true)
            .await
            .expect("archive");
        super::set_archived(conn, issue.issue_id, false)
            .await
            .expect("unarchive");
        let fetched = super::get_by_id(conn, issue.issue_id).await.expect("get");
        assert!(fetched.archived_at.is_none());
    }

    #[tokio::test]
    async fn list_excludes_archived() {
        let db = Database::open_in_memory().await.expect("open in-memory db");
        let conn = db.conn();
        let author = author::find_or_create(conn, "Alice", None)
            .await
            .expect("author");
        let i1 = super::create(conn, Some("active"), "desc", author.author_id, None)
            .await
            .expect("create i1");
        let i2 = super::create(conn, Some("to archive"), "desc", author.author_id, None)
            .await
            .expect("create i2");

        super::set_archived(conn, i2.issue_id, true)
            .await
            .expect("archive i2");

        let listed = super::list(conn).await.expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].issue_id, i1.issue_id);
    }

    #[tokio::test]
    async fn list_all_includes_archived() {
        let db = Database::open_in_memory().await.expect("open in-memory db");
        let conn = db.conn();
        let author = author::find_or_create(conn, "Alice", None)
            .await
            .expect("author");
        let i1 = super::create(conn, Some("active"), "desc", author.author_id, None)
            .await
            .expect("create i1");
        let i2 = super::create(conn, Some("archived"), "desc", author.author_id, None)
            .await
            .expect("create i2");

        super::set_archived(conn, i2.issue_id, true)
            .await
            .expect("archive i2");

        let all = super::list_all(conn).await.expect("list_all");
        let ids: Vec<i64> = all.iter().map(|i| i.issue_id).collect();
        assert!(ids.contains(&i1.issue_id));
        assert!(ids.contains(&i2.issue_id));
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn update_for_sync_clears_archived_at() {
        let db = Database::open_in_memory().await.expect("open in-memory db");
        let conn = db.conn();
        let author = author::find_or_create(conn, "Alice", None)
            .await
            .expect("author");
        let issue = super::create(conn, Some("test"), "desc", author.author_id, Some("r1"))
            .await
            .expect("create");

        super::set_archived(conn, issue.issue_id, true)
            .await
            .expect("archive");
        let archived = super::get_by_id(conn, issue.issue_id)
            .await
            .expect("get archived");
        assert!(archived.archived_at.is_some());

        super::update_for_sync(
            conn,
            issue.issue_id,
            Some("test"),
            "new desc",
            author.author_id,
            Some("r1"),
        )
        .await
        .expect("sync");
        let synced = super::get_by_id(conn, issue.issue_id)
            .await
            .expect("get synced");
        assert!(
            synced.archived_at.is_none(),
            "update_for_sync must clear archived_at"
        );
    }

    #[tokio::test]
    async fn migration_idempotent() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let db_path = tmp
            .path()
            .join("idem.db")
            .to_str()
            .expect("path")
            .to_string();
        let db = Database::open(&db_path).await.expect("first open");
        drop(db);
        // Second open runs migrate again — must not error on duplicate column
        let db2 = Database::open(&db_path).await.expect("second open");
        drop(db2);
    }
}
