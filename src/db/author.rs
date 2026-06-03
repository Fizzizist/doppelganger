use turso::Connection;

use crate::error::{Error, Result};

pub struct Author {
    pub author_id: i64,
    pub name: String,
    pub email: Option<String>,
}

pub async fn find_or_create(conn: &Connection, name: &str, email: Option<&str>) -> Result<Author> {
    let existing = if let Some(e) = email {
        find_by_name_and_email(conn, name, e).await?
    } else {
        find_by_name_no_email(conn, name).await?
    };

    if let Some(author) = existing {
        return Ok(author);
    }

    create(conn, name, email).await
}

async fn find_by_name_and_email(
    conn: &Connection,
    name: &str,
    email: &str,
) -> Result<Option<Author>> {
    let mut stmt = conn
        .prepare("SELECT author_id, name, email FROM author WHERE name = ?1 AND email = ?2")
        .await?;
    let mut rows = stmt
        .query(turso::params::Params::Positional(vec![
            turso::Value::Text(name.to_string()),
            turso::Value::Text(email.to_string()),
        ]))
        .await?;

    if let Some(row) = rows.next().await? {
        let author = row_to_author(&row)?;
        Ok(Some(author))
    } else {
        Ok(None)
    }
}

async fn find_by_name_no_email(conn: &Connection, name: &str) -> Result<Option<Author>> {
    let mut stmt = conn
        .prepare("SELECT author_id, name, email FROM author WHERE name = ?1 AND email IS NULL")
        .await?;
    let mut rows = stmt
        .query(turso::params::Params::Positional(vec![turso::Value::Text(
            name.to_string(),
        )]))
        .await?;

    if let Some(row) = rows.next().await? {
        let author = row_to_author(&row)?;
        Ok(Some(author))
    } else {
        Ok(None)
    }
}

pub async fn create(conn: &Connection, name: &str, email: Option<&str>) -> Result<Author> {
    match email {
        Some(e) => {
            conn.execute(
                "INSERT INTO author (name, email) VALUES (?1, ?2)",
                turso::params::Params::Positional(vec![
                    turso::Value::Text(name.to_string()),
                    turso::Value::Text(e.to_string()),
                ]),
            )
            .await?;
        }
        None => {
            conn.execute(
                "INSERT INTO author (name) VALUES (?1)",
                turso::params::Params::Positional(vec![turso::Value::Text(name.to_string())]),
            )
            .await?;
        }
    }

    let author_id = conn.last_insert_rowid();
    Ok(Author {
        author_id,
        name: name.to_string(),
        email: email.map(|s| s.to_string()),
    })
}

fn row_to_author(row: &turso::Row) -> Result<Author> {
    let id_val = row.get_value(0)?;
    let name_val = row.get_value(1)?;
    let email_val = row.get_value(2)?;

    let author_id = match id_val {
        turso::Value::Integer(i) => i,
        _ => return Err(Error::validation("author_id must be integer")),
    };

    let name = match name_val {
        turso::Value::Text(s) => s,
        _ => return Err(Error::validation("name must be text")),
    };

    let email = match email_val {
        turso::Value::Null => None,
        turso::Value::Text(s) => Some(s),
        _ => return Err(Error::validation("email must be text or null")),
    };

    Ok(Author {
        author_id,
        name,
        email,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
    async fn test_create_author_with_email() {
        let db = setup_db().await;
        let conn = db.connect().expect("connect");
        let author = create(&conn, "Alice", Some("alice@example.com"))
            .await
            .expect("create");
        assert_eq!(author.name, "Alice");
        assert_eq!(author.email, Some("alice@example.com".to_string()));
        assert!(author.author_id > 0);
    }

    #[tokio::test]
    async fn test_create_author_without_email() {
        let db = setup_db().await;
        let conn = db.connect().expect("connect");
        let author = create(&conn, "Bob", None).await.expect("create");
        assert_eq!(author.name, "Bob");
        assert_eq!(author.email, None);
    }

    #[tokio::test]
    async fn test_find_or_create_creates_new() {
        let db = setup_db().await;
        let conn = db.connect().expect("connect");
        let author = find_or_create(&conn, "Charlie", Some("charlie@example.com"))
            .await
            .expect("find_or_create");
        assert_eq!(author.name, "Charlie");
    }

    #[tokio::test]
    async fn test_find_or_create_finds_existing() {
        let db = setup_db().await;
        let conn = db.connect().expect("connect");
        let a1 = find_or_create(&conn, "Dave", Some("dave@example.com"))
            .await
            .expect("find_or_create 1");
        let a2 = find_or_create(&conn, "Dave", Some("dave@example.com"))
            .await
            .expect("find_or_create 2");
        assert_eq!(a1.author_id, a2.author_id);
    }
}
