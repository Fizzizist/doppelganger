use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("not inside a git repository")]
    NoRepository,

    #[error("HEAD is detached; check out a branch before using doppelganger")]
    DetachedHead,

    #[error("config directory unavailable")]
    ConfigDirUnavailable,

    #[error("config parse error: {0}")]
    ConfigParse(#[from] toml::de::Error),

    #[error("duplicate profile identifier: '{0}' is a reserved name")]
    DuplicateProfile(String),

    #[error("unknown profile: '{0}'")]
    UnknownProfile(String),

    #[error("missing required config field: '{0}'")]
    MissingProfileField(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("database lock contention")]
    LockContention,

    #[error("database error: {0}")]
    Database(#[from] turso::Error),

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

    #[error("tui error: {0}")]
    Tui(String),

    #[error("remote error: {0}")]
    Remote(String),

    #[error("no git remote 'origin' found or not a GitHub repository")]
    NoRemote,

    #[error("remote sync error: {0}")]
    RemoteSync(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn classify_db_error(e: turso::Error) -> Error {
    match &e {
        turso::Error::Busy(_) => Error::LockContention,
        _ => {
            if e.to_string().to_lowercase().contains("lock") {
                Error::LockContention
            } else {
                Error::Database(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_db_error_maps_busy_variant() {
        let e = turso::Error::Busy("database is locked".to_string());
        match classify_db_error(e) {
            Error::LockContention => {}
            other => panic!("expected LockContention, got {other:?}"),
        }
    }

    #[test]
    fn classify_db_error_maps_lock_in_message() {
        let e = turso::Error::ConversionFailure("lock contention detected".to_string());
        match classify_db_error(e) {
            Error::LockContention => {}
            other => panic!("expected LockContention, got {other:?}"),
        }
    }

    #[test]
    fn classify_db_error_passes_through_other() {
        let e = turso::Error::ConversionFailure("something else".to_string());
        match classify_db_error(e) {
            Error::Database(_) => {}
            other => panic!("expected Database, got {other:?}"),
        }
    }
}
