use crate::error::{Error, Result};
use turso::Value;

use super::{
    author::{extract_int, extract_text},
    models::Issue,
};

pub async fn create(conn: &turso::Connection, description: &str, author_id: i64) -> Result<Issue> {
    let name: String = description.chars().take(80).collect();

    conn.execute(
        "INSERT INTO issue (name, description, author_id) VALUES (?1, ?2, ?3)",
        turso::params::Params::Positional(vec![
            Value::Text(name),
            Value::Text(description.to_string()),
            Value::Integer(author_id),
        ]),
    )
    .await?;

    let issue_id = conn.last_insert_rowid();
    get_by_id(conn, issue_id).await
}

pub async fn get_by_id(conn: &turso::Connection, issue_id: i64) -> Result<Issue> {
    let mut rows = conn
        .query(
            "SELECT issue_id, name, description, author_id, created_at, updated_at \
             FROM issue WHERE issue_id = ?1",
            turso::params::Params::Positional(vec![Value::Integer(issue_id)]),
        )
        .await?;

    match rows.next().await? {
        Some(row) => row_to_issue(&row),
        None => Err(Error::IssueNotFound(issue_id)),
    }
}

fn row_to_issue(row: &turso::Row) -> Result<Issue> {
    Ok(Issue {
        issue_id: extract_int(row, 0)?,
        name: extract_text(row, 1)?,
        description: extract_text(row, 2)?,
        author_id: extract_int(row, 3)?,
        created_at: extract_text(row, 4)?,
        updated_at: extract_text(row, 5)?,
    })
}
