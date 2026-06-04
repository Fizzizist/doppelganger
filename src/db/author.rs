use crate::error::Result;
use turso::Value;

use super::models::Author;
use super::row::{extract_int, extract_optional_text, extract_text};

pub async fn find_or_create(
    conn: &turso::Connection,
    name: &str,
    email: Option<&str>,
) -> Result<Author> {
    let mut rows = conn
        .query(
            "SELECT author_id, name, email FROM author WHERE name = ?1",
            turso::params::Params::Positional(vec![Value::Text(name.to_string())]),
        )
        .await?;

    if let Some(row) = rows.next().await? {
        return row_to_author(&row);
    }

    let email_value = match email {
        Some(e) => Value::Text(e.to_string()),
        None => Value::Null,
    };

    conn.execute(
        "INSERT INTO author (name, email) VALUES (?1, ?2)",
        turso::params::Params::Positional(vec![Value::Text(name.to_string()), email_value]),
    )
    .await?;

    let author_id = conn.last_insert_rowid();

    Ok(Author {
        author_id,
        name: name.to_string(),
        email: email.map(|s| s.to_string()),
    })
}

fn row_to_author(row: &turso::Row) -> Result<Author> {
    let author_id = extract_int(row, 0)?;
    let name = extract_text(row, 1)?;
    let email = extract_optional_text(row, 2)?;
    Ok(Author {
        author_id,
        name,
        email,
    })
}
