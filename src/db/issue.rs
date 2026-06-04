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
) -> Result<Issue> {
    let name_value = match name {
        Some(n) => Value::Text(n.to_string()),
        None => Value::Null,
    };

    conn.execute(
        "INSERT INTO issue (name, description, author_id) VALUES (?1, ?2, ?3)",
        turso::params::Params::Positional(vec![
            name_value,
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
            "SELECT issue.issue_id, issue.name, issue.description, author.name, \
             issue.created_at, issue.updated_at \
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

fn row_to_issue(row: &turso::Row) -> Result<Issue> {
    Ok(Issue {
        issue_id: extract_int(row, 0)?,
        name: extract_optional_text(row, 1)?,
        description: extract_text(row, 2)?,
        author: extract_text(row, 3)?,
        created_at: extract_text(row, 4)?,
        updated_at: extract_text(row, 5)?,
    })
}
