pub mod author;
pub mod branch;
pub mod comment;
pub mod issue;
pub mod models;
pub mod schema;

use crate::error::Result;

pub struct Database {
    conn: turso::Connection,
}

impl Database {
    pub async fn open(path: &str) -> Result<Self> {
        let db = turso::Builder::new_local(path).build().await?;
        let conn = db.connect()?;
        let database = Self { conn };
        database.migrate().await?;
        Ok(database)
    }

    pub async fn open_in_memory() -> Result<Self> {
        let db = turso::Builder::new_local(":memory:").build().await?;
        let conn = db.connect()?;
        let database = Self { conn };
        database.migrate().await?;
        Ok(database)
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
