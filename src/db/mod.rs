pub mod author;
pub mod branch;
pub mod comment;
pub mod issue;
pub mod models;
pub mod row;
pub mod schema;

use crate::error::Result;

pub struct Database {
    _db: turso::Database,
    conn: turso::Connection,
}

impl Database {
    pub async fn open(path: &str) -> Result<Self> {
        // Disable file locking to avoid conflicts with other processes
        // (e.g., a shell running dg commands while the TUI is open).
        // This is safe because doppelganger is a single-process,
        // single-connection application.
        // SAFETY: set_var is unsafe in edition 2024. We set this env var
        // early in the process before any concurrent access, and it only
        // affects the turso library's file locking behavior.
        unsafe {
            std::env::set_var("LIMBO_DISABLE_FILE_LOCK", "1");
        }

        let db = turso::Builder::new_local(path).build().await?;
        let conn = db.connect()?;
        let database = Self { _db: db, conn };
        database.prepare().await?;
        Ok(database)
    }

    pub async fn open_in_memory() -> Result<Self> {
        let db = turso::Builder::new_local(":memory:").build().await?;
        let conn = db.connect()?;
        let database = Self { _db: db, conn };
        database.prepare().await?;
        Ok(database)
    }

    async fn prepare(&self) -> Result<()> {
        self.conn.execute("PRAGMA foreign_keys = ON", ()).await?;
        self.migrate().await
    }

    pub async fn migrate(&self) -> Result<()> {
        self.conn.execute(schema::AUTHOR_TABLE, ()).await?;
        self.conn.execute(schema::ISSUE_TABLE, ()).await?;
        self.conn.execute(schema::BRANCH_TABLE, ()).await?;
        self.conn.execute(schema::ISSUE_COMMENT_TABLE, ()).await?;
        self.conn.execute(schema::BRANCH_COMMENT_TABLE, ()).await?;
        Ok(())
    }

    pub async fn checkpoint(&self) -> Result<()> {
        let mut rows = self
            .conn
            .query("PRAGMA wal_checkpoint(TRUNCATE)", ())
            .await?;
        while rows.next().await?.is_some() {}
        Ok(())
    }

    pub fn conn(&self) -> &turso::Connection {
        &self.conn
    }
}
