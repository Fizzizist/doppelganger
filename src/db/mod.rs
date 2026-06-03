pub mod author;
pub mod branch;
pub mod comment;
pub mod issue;
pub mod schema;

use std::path::PathBuf;

use crate::error::{Error, Result};

const DB_FILENAME: &str = ".doppelganger.db";

pub struct Database {
    db: turso::Database,
}

impl Database {
    pub async fn open(repo_root: &std::path::Path) -> Result<Self> {
        let db_path = repo_root.join(DB_FILENAME);
        let db_path_str = db_path
            .to_str()
            .ok_or_else(|| Error::validation("invalid database path"))?;
        let db = turso::Builder::new_local(db_path_str).build().await?;
        let database = Database { db };
        let conn = database.connect()?;
        schema::migrate(&conn).await?;
        Ok(database)
    }

    pub async fn open_in_memory() -> Result<Self> {
        let db = turso::Builder::new_local(":memory:").build().await?;
        let database = Database { db };
        let conn = database.connect()?;
        schema::migrate(&conn).await?;
        Ok(database)
    }

    pub fn connect(&self) -> Result<turso::Connection> {
        self.db.connect().map_err(Error::from)
    }

    pub fn db_path(repo_root: &std::path::Path) -> PathBuf {
        repo_root.join(DB_FILENAME)
    }
}
