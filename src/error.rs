use std::fmt;
use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database error: {0}")]
    Database(#[from] turso::Error),
    #[error("git error: {0}")]
    Git(#[from] git2::Error),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

impl Error {
    pub fn validation(msg: impl fmt::Display) -> Self {
        Error::Validation(msg.to_string())
    }

    pub fn not_found(msg: impl fmt::Display) -> Self {
        Error::NotFound(msg.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
