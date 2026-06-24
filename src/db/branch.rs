use crate::error::{Error, Result};
use turso::Value;

use super::{
    models::Branch,
    row::{extract_int, extract_optional_text, extract_text},
};

const SELECT_BRANCH: &str = r#"
SELECT branch.branch_id, branch.name, branch.description, author.name,
       branch.issue_id, branch.created_at, branch.updated_at, branch.archived_at
FROM branch JOIN author ON branch.author_id = author.author_id
"#;

pub async fn create(
    conn: &turso::Connection,
    name: &str,
    description: &str,
    author_id: i64,
    issue_id: i64,
) -> Result<Branch> {
    let result = conn
        .execute(
            "INSERT INTO branch (name, description, author_id, issue_id) \
             VALUES (?1, ?2, ?3, ?4)",
            turso::params::Params::Positional(vec![
                Value::Text(name.to_string()),
                Value::Text(description.to_string()),
                Value::Integer(author_id),
                Value::Integer(issue_id),
            ]),
        )
        .await;

    match result {
        Ok(_) => {}
        Err(turso::Error::Constraint(_)) => {
            return Err(Error::BranchAlreadyExists(name.to_string()));
        }
        Err(e) => return Err(Error::Database(e)),
    }

    let branch_id = conn.last_insert_rowid();
    get_by_id(conn, branch_id).await
}

pub async fn get_by_name(conn: &turso::Connection, name: &str) -> Result<Branch> {
    let mut rows = conn
        .query(
            format!("{SELECT_BRANCH} WHERE branch.name = ?1"),
            turso::params::Params::Positional(vec![Value::Text(name.to_string())]),
        )
        .await?;

    match rows.next().await? {
        Some(row) => row_to_branch(&row),
        None => Err(Error::BranchNotFound(name.to_string())),
    }
}

pub async fn get_active_by_name(conn: &turso::Connection, name: &str) -> Result<Branch> {
    let mut rows = conn
        .query(
            format!("{SELECT_BRANCH} WHERE branch.name = ?1 AND branch.archived_at IS NULL"),
            turso::params::Params::Positional(vec![Value::Text(name.to_string())]),
        )
        .await?;

    match rows.next().await? {
        Some(row) => row_to_branch(&row),
        None => Err(Error::BranchNotFound(name.to_string())),
    }
}

pub async fn get_by_id(conn: &turso::Connection, branch_id: i64) -> Result<Branch> {
    let mut rows = conn
        .query(
            format!("{SELECT_BRANCH} WHERE branch.branch_id = ?1"),
            turso::params::Params::Positional(vec![Value::Integer(branch_id)]),
        )
        .await?;

    match rows.next().await? {
        Some(row) => row_to_branch(&row),
        None => Err(Error::BranchNotFound(branch_id.to_string())),
    }
}

pub async fn set_archived(conn: &turso::Connection, branch_id: i64, archived: bool) -> Result<()> {
    let sql = if archived {
        "UPDATE branch SET archived_at = datetime('now'), updated_at = datetime('now') WHERE branch_id = ?1"
    } else {
        "UPDATE branch SET archived_at = NULL, updated_at = datetime('now') WHERE branch_id = ?1"
    };
    conn.execute(
        sql,
        turso::params::Params::Positional(vec![Value::Integer(branch_id)]),
    )
    .await?;
    Ok(())
}

pub async fn update_description(
    conn: &turso::Connection,
    name: &str,
    description: &str,
) -> Result<Branch> {
    let rows_affected = conn
        .execute(
            "UPDATE branch SET description = ?1, updated_at = datetime('now') \
             WHERE name = ?2 AND archived_at IS NULL",
            turso::params::Params::Positional(vec![
                Value::Text(description.to_string()),
                Value::Text(name.to_string()),
            ]),
        )
        .await?;

    if rows_affected == 0 {
        match get_by_name(conn, name).await {
            Ok(_) => {
                return Err(Error::Validation(format!(
                    "branch '{name}' is archived and cannot be updated"
                )));
            }
            Err(Error::BranchNotFound(_)) => {
                return Err(Error::BranchNotFound(name.to_string()));
            }
            Err(e) => return Err(e),
        }
    }

    get_active_by_name(conn, name).await
}

fn row_to_branch(row: &turso::Row) -> Result<Branch> {
    Ok(Branch {
        branch_id: extract_int(row, 0)?,
        name: extract_text(row, 1)?,
        description: extract_text(row, 2)?,
        author: extract_text(row, 3)?,
        issue_id: extract_int(row, 4)?,
        created_at: extract_text(row, 5)?,
        updated_at: extract_text(row, 6)?,
        archived_at: extract_optional_text(row, 7)?,
    })
}

#[cfg(test)]
mod tests {
    use crate::db::{Database, author, issue};

    #[tokio::test]
    async fn set_archived_sets_archived_at() {
        let db = Database::open_in_memory().await.expect("open in-memory db");
        let conn = db.conn();
        let author = author::find_or_create(conn, "Alice", None)
            .await
            .expect("author");
        let iss = issue::create(conn, Some("test"), "desc", author.author_id, None)
            .await
            .expect("issue");
        let br = super::create(conn, "feature-1", "desc", author.author_id, iss.issue_id)
            .await
            .expect("branch");

        assert!(br.archived_at.is_none());

        super::set_archived(conn, br.branch_id, true)
            .await
            .expect("archive");
        let fetched = super::get_by_id(conn, br.branch_id).await.expect("get");
        assert!(fetched.archived_at.is_some());
        assert!(
            fetched.updated_at >= br.updated_at,
            "updated_at must not go backwards after archiving"
        );
    }

    #[tokio::test]
    async fn set_archived_unarchives() {
        let db = Database::open_in_memory().await.expect("open in-memory db");
        let conn = db.conn();
        let author = author::find_or_create(conn, "Alice", None)
            .await
            .expect("author");
        let iss = issue::create(conn, Some("test"), "desc", author.author_id, None)
            .await
            .expect("issue");
        let br = super::create(conn, "feature-1", "desc", author.author_id, iss.issue_id)
            .await
            .expect("branch");

        super::set_archived(conn, br.branch_id, true)
            .await
            .expect("archive");
        super::set_archived(conn, br.branch_id, false)
            .await
            .expect("unarchive");
        let fetched = super::get_by_id(conn, br.branch_id).await.expect("get");
        assert!(fetched.archived_at.is_none());
    }

    #[tokio::test]
    async fn get_active_by_name_excludes_archived() {
        let db = Database::open_in_memory().await.expect("open in-memory db");
        let conn = db.conn();
        let author = author::find_or_create(conn, "Alice", None)
            .await
            .expect("author");
        let iss = issue::create(conn, Some("test"), "desc", author.author_id, None)
            .await
            .expect("issue");
        let br = super::create(conn, "feature-1", "desc", author.author_id, iss.issue_id)
            .await
            .expect("branch");

        super::set_archived(conn, br.branch_id, true)
            .await
            .expect("archive");

        let result = super::get_active_by_name(conn, "feature-1").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::Error::BranchNotFound(name) => assert_eq!(name, "feature-1"),
            other => panic!("expected BranchNotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn get_active_by_name_returns_active() {
        let db = Database::open_in_memory().await.expect("open in-memory db");
        let conn = db.conn();
        let author = author::find_or_create(conn, "Alice", None)
            .await
            .expect("author");
        let iss = issue::create(conn, Some("test"), "desc", author.author_id, None)
            .await
            .expect("issue");
        let br = super::create(conn, "feature-1", "desc", author.author_id, iss.issue_id)
            .await
            .expect("branch");

        let fetched = super::get_active_by_name(conn, "feature-1")
            .await
            .expect("get active");
        assert_eq!(fetched.branch_id, br.branch_id);
        assert!(fetched.archived_at.is_none());
    }

    #[tokio::test]
    async fn row_to_branch_includes_archived_at() {
        let db = Database::open_in_memory().await.expect("open in-memory db");
        let conn = db.conn();
        let author = author::find_or_create(conn, "Alice", None)
            .await
            .expect("author");
        let iss = issue::create(conn, Some("test"), "desc", author.author_id, None)
            .await
            .expect("issue");
        let br = super::create(conn, "feature-1", "desc", author.author_id, iss.issue_id)
            .await
            .expect("branch");

        super::set_archived(conn, br.branch_id, true)
            .await
            .expect("archive");
        let fetched = super::get_by_id(conn, br.branch_id).await.expect("get");
        assert!(fetched.archived_at.is_some());
        assert!(fetched.archived_at.as_ref().is_some_and(|s| !s.is_empty()));
    }

    #[tokio::test]
    async fn partial_unique_index_prevents_duplicate_active() {
        let db = Database::open_in_memory().await.expect("open in-memory db");
        let conn = db.conn();
        let author = author::find_or_create(conn, "Alice", None)
            .await
            .expect("author");
        let iss = issue::create(conn, Some("test"), "desc", author.author_id, None)
            .await
            .expect("issue");
        let br1 = super::create(conn, "same-name", "desc1", author.author_id, iss.issue_id)
            .await
            .expect("first branch");

        super::set_archived(conn, br1.branch_id, true)
            .await
            .expect("archive first");

        let br2 = super::create(conn, "same-name", "desc2", author.author_id, iss.issue_id)
            .await
            .expect("second branch with same name after first archived");

        assert_eq!(br1.name, br2.name);
        assert_ne!(br1.branch_id, br2.branch_id);

        let result =
            super::create(conn, "same-name", "desc3", author.author_id, iss.issue_id).await;
        assert!(
            result.is_err(),
            "should not be able to create a third active branch with the same name"
        );
    }

    #[tokio::test]
    async fn update_description_fails_on_archived() {
        let db = Database::open_in_memory().await.expect("open in-memory db");
        let conn = db.conn();
        let author = author::find_or_create(conn, "Alice", None)
            .await
            .expect("author");
        let iss = issue::create(conn, Some("test"), "desc", author.author_id, None)
            .await
            .expect("issue");
        let br = super::create(
            conn,
            "feature-1",
            "original",
            author.author_id,
            iss.issue_id,
        )
        .await
        .expect("branch");

        super::set_archived(conn, br.branch_id, true)
            .await
            .expect("archive");

        let result = super::update_description(conn, "feature-1", "updated").await;
        assert!(result.is_err(), "should not update archived branch");
        match result.unwrap_err() {
            crate::error::Error::Validation(msg) => {
                assert!(
                    msg.contains("archived"),
                    "error should mention archived, got: {msg}"
                );
            }
            other => panic!("expected Validation error for archived branch, got {other:?}"),
        }
    }
}
