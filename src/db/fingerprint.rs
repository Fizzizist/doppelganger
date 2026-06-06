use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::error::Result;

pub async fn issue_list_fingerprint(conn: &turso::Connection) -> Result<u64> {
    let mut rows = conn
        .query("SELECT COUNT(issue_id), MAX(updated_at) FROM issue", ())
        .await?;

    match rows.next().await? {
        Some(row) => {
            let count: i64 = row.get(0).unwrap_or(0);
            let max_updated: Option<String> = row.get(1).ok();
            let mut hasher = DefaultHasher::new();
            (count, max_updated).hash(&mut hasher);
            Ok(hasher.finish())
        }
        None => Ok(0),
    }
}

pub async fn thread_fingerprint(
    conn: &turso::Connection,
    issue_id: Option<i64>,
    branch_id: Option<i64>,
) -> Result<u64> {
    let (comment_count, max_updated_at) = if let Some(id) = issue_id {
        let mut rows = conn
            .query(
                "SELECT COUNT(issue_comment_id), MAX(updated_at) FROM issue_comment WHERE issue_id = ?1",
                turso::params::Params::Positional(vec![turso::Value::Integer(id)]),
            )
            .await?;
        match rows.next().await? {
            Some(row) => {
                let count: i64 = row.get(0).unwrap_or(0);
                let max_updated: Option<String> = row.get(1).ok();
                (count, max_updated)
            }
            None => return Ok(0),
        }
    } else if let Some(id) = branch_id {
        let mut rows = conn
            .query(
                "SELECT COUNT(branch_comment_id), MAX(updated_at) FROM branch_comment WHERE branch_id = ?1",
                turso::params::Params::Positional(vec![turso::Value::Integer(id)]),
            )
            .await?;
        match rows.next().await? {
            Some(row) => {
                let count: i64 = row.get(0).unwrap_or(0);
                let max_updated: Option<String> = row.get(1).ok();
                (count, max_updated)
            }
            None => return Ok(0),
        }
    } else {
        return Ok(0);
    };

    let parent_updated = if let Some(id) = issue_id {
        let mut rows = conn
            .query(
                "SELECT updated_at FROM issue WHERE issue_id = ?1",
                turso::params::Params::Positional(vec![turso::Value::Integer(id)]),
            )
            .await?;
        let row = rows.next().await?;
        row.and_then(|r| r.get::<String>(0).ok())
    } else if let Some(id) = branch_id {
        let mut rows = conn
            .query(
                "SELECT updated_at FROM branch WHERE branch_id = ?1",
                turso::params::Params::Positional(vec![turso::Value::Integer(id)]),
            )
            .await?;
        let row = rows.next().await?;
        row.and_then(|r| r.get::<String>(0).ok())
    } else {
        None
    };

    let mut hasher = DefaultHasher::new();
    (comment_count, max_updated_at, parent_updated).hash(&mut hasher);
    Ok(hasher.finish())
}
