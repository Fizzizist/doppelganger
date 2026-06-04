use crate::error::{Error, Result};
use turso::Value;

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
