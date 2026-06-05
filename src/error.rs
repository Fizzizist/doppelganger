use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("not inside a git repository")]
    NoRepository,

    #[error("HEAD is detached; check out a branch before using doppelganger")]
    DetachedHead,

    #[error("git config user.name is required but not set")]
    MissingAuthorName,

    #[error("validation error: {0}")]
    Validation(String),

    #[error("database error: {0}")]
    Database(#[from] turso::Error),

    #[error("database lock contention")]
    LockContention,

    #[error("git error: {0}")]
    Git(#[from] git2::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("branch not found: no branch record for '{0}'; run `dg branch create` first")]
    BranchNotFound(String),

    #[error("branch already exists: '{0}'; use --overwrite to update description")]
    BranchAlreadyExists(String),

    #[error("issue not found: #{0}")]
    IssueNotFound(i64),
}

pub type Result<T> = std::result::Result<T, Error>;
