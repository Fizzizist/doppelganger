use crate::error::{Error, Result};
use turso::Value;

use super::models::Author;

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

pub fn extract_int(row: &turso::Row, idx: usize) -> Result<i64> {
    match row.get_value(idx)? {
        Value::Integer(i) => Ok(i),
        other => Err(Error::Database(turso::Error::ConversionFailure(format!(
            "expected integer at column {idx}, got {other:?}"
        )))),
    }
}

pub fn extract_text(row: &turso::Row, idx: usize) -> Result<String> {
    match row.get_value(idx)? {
        Value::Text(s) => Ok(s),
        other => Err(Error::Database(turso::Error::ConversionFailure(format!(
            "expected text at column {idx}, got {other:?}"
        )))),
    }
}

pub fn extract_optional_text(row: &turso::Row, idx: usize) -> Result<Option<String>> {
    match row.get_value(idx)? {
        Value::Text(s) => Ok(Some(s)),
        Value::Null => Ok(None),
        other => Err(Error::Database(turso::Error::ConversionFailure(format!(
            "expected text or null at column {idx}, got {other:?}"
        )))),
    }
}
