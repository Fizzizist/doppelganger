use serde::Serialize;
use turso::Connection;

use crate::error::{Error, Result};

#[derive(Debug, Serialize)]
pub struct Branch {
    pub branch_id: i64,
    pub name: String,
    pub description: String,
    pub author_id: i64,
    pub issue_id: i64,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn create(
    conn: &Connection,
    name: &str,
    description: &str,
    author_id: i64,
    issue_id: i64,
) -> Result<Branch> {
    conn.execute(
        "INSERT INTO branch (name, description, author_id, issue_id) VALUES (?1, ?2, ?3, ?4)",
        turso::params::Params::Positional(vec![
            turso::Value::Text(name.to_string()),
            turso::Value::Text(description.to_string()),
            turso::Value::Integer(author_id),
            turso::Value::Integer(issue_id),
        ]),
    )
    .await?;

    let branch_id = conn.last_insert_rowid();

    get_by_id(conn, branch_id)
        .await?
        .ok_or_else(|| Error::not_found(format!("branch {branch_id} not found after insert")))
}

pub async fn get_by_id(conn: &Connection, branch_id: i64) -> Result<Option<Branch>> {
    let mut stmt = conn
        .prepare("SELECT branch_id, name, description, author_id, issue_id, created_at, updated_at FROM branch WHERE branch_id = ?1")
        .await?;
    let mut rows = stmt
        .query(turso::params::Params::Positional(vec![
            turso::Value::Integer(branch_id),
        ]))
        .await?;

    if let Some(row) = rows.next().await? {
        Ok(Some(row_to_branch(&row)?))
    } else {
        Ok(None)
    }
}

pub async fn get_by_name(conn: &Connection, name: &str) -> Result<Option<Branch>> {
    let mut stmt = conn
        .prepare("SELECT branch_id, name, description, author_id, issue_id, created_at, updated_at FROM branch WHERE name = ?1")
        .await?;
    let mut rows = stmt
        .query(turso::params::Params::Positional(vec![turso::Value::Text(
            name.to_string(),
        )]))
        .await?;

    if let Some(row) = rows.next().await? {
        Ok(Some(row_to_branch(&row)?))
    } else {
        Ok(None)
    }
}

fn row_to_branch(row: &turso::Row) -> Result<Branch> {
    let branch_id = match row.get_value(0)? {
        turso::Value::Integer(i) => i,
        _ => return Err(Error::validation("branch_id must be integer")),
    };
    let name = match row.get_value(1)? {
        turso::Value::Text(s) => s,
        _ => return Err(Error::validation("name must be text")),
    };
    let description = match row.get_value(2)? {
        turso::Value::Text(s) => s,
        _ => return Err(Error::validation("description must be text")),
    };
    let author_id = match row.get_value(3)? {
        turso::Value::Integer(i) => i,
        _ => return Err(Error::validation("author_id must be integer")),
    };
    let issue_id = match row.get_value(4)? {
        turso::Value::Integer(i) => i,
        _ => return Err(Error::validation("issue_id must be integer")),
    };
    let created_at = match row.get_value(5)? {
        turso::Value::Text(s) => s,
        _ => return Err(Error::validation("created_at must be text")),
    };
    let updated_at = match row.get_value(6)? {
        turso::Value::Text(s) => s,
        _ => return Err(Error::validation("updated_at must be text")),
    };

    Ok(Branch {
        branch_id,
        name,
        description,
        author_id,
        issue_id,
        created_at,
        updated_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{author, issue};

    async fn setup_db() -> turso::Database {
        let db = turso::Builder::new_local(":memory:")
            .build()
            .await
            .expect("build db");
        let conn = db.connect().expect("connect");
        crate::db::schema::migrate(&conn).await.expect("migrate");
        db
    }

    #[tokio::test]
    async fn test_create_and_get_branch() {
        let db = setup_db().await;
        let conn = db.connect().expect("connect");
        let a = author::create(&conn, "Author1", None)
            .await
            .expect("create author");
        let iss = issue::create(&conn, Some("Test issue"), "desc", a.author_id)
            .await
            .expect("create issue");

        let branch = create(
            &conn,
            "feature-branch",
            "implementation of feature",
            a.author_id,
            iss.issue_id,
        )
        .await
        .expect("create branch");

        assert_eq!(branch.name, "feature-branch");
        assert_eq!(branch.issue_id, iss.issue_id);

        let fetched = get_by_name(&conn, "feature-branch")
            .await
            .expect("get_by_name")
            .expect("found");
        assert_eq!(fetched.branch_id, branch.branch_id);
    }

    #[tokio::test]
    async fn test_get_nonexistent_branch() {
        let db = setup_db().await;
        let conn = db.connect().expect("connect");
        let result = get_by_name(&conn, "nonexistent")
            .await
            .expect("get_by_name");
        assert!(result.is_none());
    }
}
