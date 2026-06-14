use crate::error::{Error, Result};
use turso::Value;

use super::{
    models::{BranchComment, IssueComment},
    row::{extract_int, extract_text},
};

const SELECT_ISSUE_COMMENT: &str = r#"
SELECT issue_comment.issue_comment_id, issue_comment.content, author.name,
       issue_comment.issue_id, issue_comment.created_at, issue_comment.updated_at
FROM issue_comment JOIN author ON issue_comment.author_id = author.author_id
"#;

const SELECT_BRANCH_COMMENT: &str = r#"
SELECT branch_comment.branch_comment_id, branch_comment.content, author.name,
       branch_comment.branch_id, branch_comment.created_at, branch_comment.updated_at
FROM branch_comment JOIN author ON branch_comment.author_id = author.author_id
"#;

pub async fn delete_issue_comments(conn: &turso::Connection, issue_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM issue_comment WHERE issue_id = ?1",
        turso::params::Params::Positional(vec![Value::Integer(issue_id)]),
    )
    .await?;
    Ok(())
}

pub async fn create_issue_comment(
    conn: &turso::Connection,
    issue_id: i64,
    content: &str,
    author_id: i64,
) -> Result<IssueComment> {
    conn.execute(
        "INSERT INTO issue_comment (issue_id, content, author_id) VALUES (?1, ?2, ?3)",
        turso::params::Params::Positional(vec![
            Value::Integer(issue_id),
            Value::Text(content.to_string()),
            Value::Integer(author_id),
        ]),
    )
    .await?;

    let comment_id = conn.last_insert_rowid();
    let mut rows = conn
        .query(
            format!("{SELECT_ISSUE_COMMENT} WHERE issue_comment.issue_comment_id = ?1"),
            turso::params::Params::Positional(vec![Value::Integer(comment_id)]),
        )
        .await?;

    match rows.next().await? {
        Some(row) => row_to_issue_comment(&row),
        None => Err(Error::Database(turso::Error::ConversionFailure(
            "just-inserted issue_comment could not be read back".to_string(),
        ))),
    }
}

pub async fn list_issue_comments(
    conn: &turso::Connection,
    issue_id: i64,
) -> Result<Vec<IssueComment>> {
    let mut rows = conn
        .query(
            format!(
                "{SELECT_ISSUE_COMMENT} WHERE issue_comment.issue_id = ?1 \
                 ORDER BY issue_comment.created_at ASC"
            ),
            turso::params::Params::Positional(vec![Value::Integer(issue_id)]),
        )
        .await?;

    let mut comments = Vec::new();
    while let Some(row) = rows.next().await? {
        comments.push(row_to_issue_comment(&row)?);
    }
    Ok(comments)
}

pub async fn create_branch_comment(
    conn: &turso::Connection,
    branch_id: i64,
    content: &str,
    author_id: i64,
) -> Result<BranchComment> {
    conn.execute(
        "INSERT INTO branch_comment (branch_id, content, author_id) VALUES (?1, ?2, ?3)",
        turso::params::Params::Positional(vec![
            Value::Integer(branch_id),
            Value::Text(content.to_string()),
            Value::Integer(author_id),
        ]),
    )
    .await?;

    let comment_id = conn.last_insert_rowid();
    let mut rows = conn
        .query(
            format!("{SELECT_BRANCH_COMMENT} WHERE branch_comment.branch_comment_id = ?1"),
            turso::params::Params::Positional(vec![Value::Integer(comment_id)]),
        )
        .await?;

    match rows.next().await? {
        Some(row) => row_to_branch_comment(&row),
        None => Err(Error::Database(turso::Error::ConversionFailure(
            "just-inserted branch_comment could not be read back".to_string(),
        ))),
    }
}

pub async fn list_branch_comments(
    conn: &turso::Connection,
    branch_id: i64,
) -> Result<Vec<BranchComment>> {
    let mut rows = conn
        .query(
            format!(
                "{SELECT_BRANCH_COMMENT} WHERE branch_comment.branch_id = ?1 \
                 ORDER BY branch_comment.created_at ASC"
            ),
            turso::params::Params::Positional(vec![Value::Integer(branch_id)]),
        )
        .await?;

    let mut comments = Vec::new();
    while let Some(row) = rows.next().await? {
        comments.push(row_to_branch_comment(&row)?);
    }
    Ok(comments)
}

fn row_to_issue_comment(row: &turso::Row) -> Result<IssueComment> {
    Ok(IssueComment {
        issue_comment_id: extract_int(row, 0)?,
        content: extract_text(row, 1)?,
        author: extract_text(row, 2)?,
        issue_id: extract_int(row, 3)?,
        created_at: extract_text(row, 4)?,
        updated_at: extract_text(row, 5)?,
    })
}

fn row_to_branch_comment(row: &turso::Row) -> Result<BranchComment> {
    Ok(BranchComment {
        branch_comment_id: extract_int(row, 0)?,
        content: extract_text(row, 1)?,
        author: extract_text(row, 2)?,
        branch_id: extract_int(row, 3)?,
        created_at: extract_text(row, 4)?,
        updated_at: extract_text(row, 5)?,
    })
}
