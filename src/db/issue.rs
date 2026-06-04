use crate::error::{Error, Result};
use turso::Value;

use super::{models::Issue, row::extract_int, row::extract_text};

const NAME_MAX_CHARS: usize = 80;

pub async fn create(conn: &turso::Connection, description: &str, author_id: i64) -> Result<Issue> {
    let name = derive_name(description);

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

/// Derive a short title from the description: the first line, truncated to
/// `NAME_MAX_CHARS` at a word boundary (never mid-word) when possible.
fn derive_name(description: &str) -> String {
    let first_line = description.lines().next().unwrap_or("").trim();
    if first_line.chars().count() <= NAME_MAX_CHARS {
        return first_line.to_string();
    }

    let truncated: String = first_line.chars().take(NAME_MAX_CHARS).collect();
    match truncated.rsplit_once(char::is_whitespace) {
        Some((head, _)) if !head.trim().is_empty() => head.trim_end().to_string(),
        _ => truncated.trim_end().to_string(),
    }
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
        name: extract_text(row, 1)?,
        description: extract_text(row, 2)?,
        author: extract_text(row, 3)?,
        created_at: extract_text(row, 4)?,
        updated_at: extract_text(row, 5)?,
    })
}
