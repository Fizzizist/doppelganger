use crate::error::{Error, Result};
use turso::Value;

use super::{
    models::Branch,
    row::{extract_int, extract_text},
};

const SELECT_BRANCH: &str = "SELECT branch.branch_id, branch.name, branch.description, \
     author.name, branch.issue_id, branch.created_at, branch.updated_at \
     FROM branch JOIN author ON branch.author_id = author.author_id";

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

pub async fn update_description(
    conn: &turso::Connection,
    name: &str,
    description: &str,
) -> Result<Branch> {
    conn.execute(
        "UPDATE branch SET description = ?1, updated_at = datetime('now') WHERE name = ?2",
        turso::params::Params::Positional(vec![
            Value::Text(description.to_string()),
            Value::Text(name.to_string()),
        ]),
    )
    .await?;

    get_by_name(conn, name).await
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
    })
}
