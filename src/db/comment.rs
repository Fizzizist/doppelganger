use crate::error::Result;
use turso::Value;

use super::{
    author::{extract_int, extract_text},
    models::{BranchComment, IssueComment},
};

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
            "SELECT issue_comment_id, content, author_id, issue_id, created_at, updated_at \
             FROM issue_comment WHERE issue_comment_id = ?1",
            turso::params::Params::Positional(vec![Value::Integer(comment_id)]),
        )
        .await?;

    let row = rows
        .next()
        .await?
        .expect("just-inserted issue_comment must exist");
    row_to_issue_comment(&row)
}

pub async fn list_issue_comments(
    conn: &turso::Connection,
    issue_id: i64,
) -> Result<Vec<IssueComment>> {
    let mut rows = conn
        .query(
            "SELECT issue_comment_id, content, author_id, issue_id, created_at, updated_at \
             FROM issue_comment WHERE issue_id = ?1 ORDER BY created_at ASC",
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
            "SELECT branch_comment_id, content, author_id, branch_id, created_at, updated_at \
             FROM branch_comment WHERE branch_comment_id = ?1",
            turso::params::Params::Positional(vec![Value::Integer(comment_id)]),
        )
        .await?;

    let row = rows
        .next()
        .await?
        .expect("just-inserted branch_comment must exist");
    row_to_branch_comment(&row)
}

pub async fn list_branch_comments(
    conn: &turso::Connection,
    branch_id: i64,
) -> Result<Vec<BranchComment>> {
    let mut rows = conn
        .query(
            "SELECT branch_comment_id, content, author_id, branch_id, created_at, updated_at \
             FROM branch_comment WHERE branch_id = ?1 ORDER BY created_at ASC",
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
        author_id: extract_int(row, 2)?,
        issue_id: extract_int(row, 3)?,
        created_at: extract_text(row, 4)?,
        updated_at: extract_text(row, 5)?,
    })
}

fn row_to_branch_comment(row: &turso::Row) -> Result<BranchComment> {
    Ok(BranchComment {
        branch_comment_id: extract_int(row, 0)?,
        content: extract_text(row, 1)?,
        author_id: extract_int(row, 2)?,
        branch_id: extract_int(row, 3)?,
        created_at: extract_text(row, 4)?,
        updated_at: extract_text(row, 5)?,
    })
}
