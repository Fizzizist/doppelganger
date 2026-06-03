use serde::Serialize;
use turso::Connection;

use crate::error::{Error, Result};

#[derive(Debug, Serialize)]
pub struct Issue {
    pub issue_id: i64,
    pub name: Option<String>,
    pub description: String,
    pub author_id: i64,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn create(
    conn: &Connection,
    name: Option<&str>,
    description: &str,
    author_id: i64,
) -> Result<Issue> {
    match name {
        Some(n) => {
            conn.execute(
                "INSERT INTO issue (name, description, author_id) VALUES (?1, ?2, ?3)",
                turso::params::Params::Positional(vec![
                    turso::Value::Text(n.to_string()),
                    turso::Value::Text(description.to_string()),
                    turso::Value::Integer(author_id),
                ]),
            )
            .await?;
        }
        None => {
            conn.execute(
                "INSERT INTO issue (description, author_id) VALUES (?1, ?2)",
                turso::params::Params::Positional(vec![
                    turso::Value::Text(description.to_string()),
                    turso::Value::Integer(author_id),
                ]),
            )
            .await?;
        }
    }

    let issue_id = conn.last_insert_rowid();

    get_by_id(conn, issue_id)
        .await?
        .ok_or_else(|| Error::not_found(format!("issue {issue_id} not found after insert")))
}

pub async fn get_by_id(conn: &Connection, issue_id: i64) -> Result<Option<Issue>> {
    let mut stmt = conn
        .prepare("SELECT issue_id, name, description, author_id, created_at, updated_at FROM issue WHERE issue_id = ?1")
        .await?;
    let mut rows = stmt
        .query(turso::params::Params::Positional(vec![
            turso::Value::Integer(issue_id),
        ]))
        .await?;

    if let Some(row) = rows.next().await? {
        Ok(Some(row_to_issue(&row)?))
    } else {
        Ok(None)
    }
}

fn row_to_issue(row: &turso::Row) -> Result<Issue> {
    let issue_id = match row.get_value(0)? {
        turso::Value::Integer(i) => i,
        _ => return Err(Error::validation("issue_id must be integer")),
    };
    let name = match row.get_value(1)? {
        turso::Value::Null => None,
        turso::Value::Text(s) => Some(s),
        _ => return Err(Error::validation("name must be text or null")),
    };
    let description = match row.get_value(2)? {
        turso::Value::Text(s) => s,
        _ => return Err(Error::validation("description must be text")),
    };
    let author_id = match row.get_value(3)? {
        turso::Value::Integer(i) => i,
        _ => return Err(Error::validation("author_id must be integer")),
    };
    let created_at = match row.get_value(4)? {
        turso::Value::Text(s) => s,
        _ => return Err(Error::validation("created_at must be text")),
    };
    let updated_at = match row.get_value(5)? {
        turso::Value::Text(s) => s,
        _ => return Err(Error::validation("updated_at must be text")),
    };

    Ok(Issue {
        issue_id,
        name,
        description,
        author_id,
        created_at,
        updated_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::author;

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
    async fn test_create_and_get_issue_with_name() {
        let db = setup_db().await;
        let conn = db.connect().expect("connect");
        let a = author::create(&conn, "TestAuthor", Some("test@example.com"))
            .await
            .expect("create author");

        let issue = create(
            &conn,
            Some("Bug in login"),
            "Login button does nothing",
            a.author_id,
        )
        .await
        .expect("create issue");

        assert_eq!(issue.name, Some("Bug in login".to_string()));
        assert_eq!(issue.description, "Login button does nothing");
        assert_eq!(issue.author_id, a.author_id);
        assert!(issue.issue_id > 0);

        let fetched = get_by_id(&conn, issue.issue_id)
            .await
            .expect("get_by_id")
            .expect("found");
        assert_eq!(fetched.name, Some("Bug in login".to_string()));
    }

    #[tokio::test]
    async fn test_create_issue_without_name() {
        let db = setup_db().await;
        let conn = db.connect().expect("connect");
        let a = author::create(&conn, "TestAuthor", None)
            .await
            .expect("create author");

        let issue = create(&conn, None, "Just a description", a.author_id)
            .await
            .expect("create issue");

        assert!(issue.name.is_none());
        assert_eq!(issue.description, "Just a description");
    }

    #[tokio::test]
    async fn test_get_nonexistent_issue() {
        let db = setup_db().await;
        let conn = db.connect().expect("connect");
        let result = get_by_id(&conn, 9999).await.expect("get_by_id");
        assert!(result.is_none());
    }
}
