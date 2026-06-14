pub mod author;
pub mod branch;
pub mod comment;
pub mod issue;
pub mod models;
pub mod row;
pub mod schema;

use crate::error::{Error, Result, classify_db_error};

pub struct Database {
    conn: turso::Connection,
}

impl Database {
    pub async fn open(path: &str) -> Result<Self> {
        let max_retries = 200;
        let delay = std::time::Duration::from_millis(25);

        for attempt in 0..max_retries {
            let result: Result<Self> = async {
                let db = turso::Builder::new_local(path)
                    .experimental_multiprocess_wal(true)
                    .build()
                    .await?;
                let conn = db.connect()?;
                let database = Self { conn };
                database.prepare().await?;
                Ok(database)
            }
            .await
            .map_err(|e| match e {
                Error::Database(e) => classify_db_error(e),
                other => other,
            });

            match result {
                Ok(db) => return Ok(db),
                Err(Error::LockContention) if attempt < max_retries - 1 => {
                    tokio::time::sleep(delay).await;
                }
                Err(Error::LockContention) => return Err(Error::LockContention),
                Err(e) => return Err(e),
            }
        }

        Err(Error::LockContention)
    }

    pub async fn open_in_memory() -> Result<Self> {
        let db = turso::Builder::new_local(":memory:").build().await?;
        let conn = db.connect()?;
        let database = Self { conn };
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
        self.migrate_remote_id().await?;
        Ok(())
    }

    async fn migrate_remote_id(&self) -> Result<()> {
        match self
            .conn
            .execute(schema::ALTER_TABLE_ISSUE_REMOTE_ID, ())
            .await
        {
            Ok(_) => Ok(()),
            Err(e) if e.to_string().contains("duplicate column name") => Ok(()),
            Err(e) => Err(classify_db_error(e)),
        }
    }

    pub async fn checkpoint(&self) -> Result<()> {
        let max_retries = 200;
        let delay = std::time::Duration::from_millis(25);

        for attempt in 0..max_retries {
            let result: Result<()> = async {
                let mut rows = self
                    .conn
                    .query("PRAGMA wal_checkpoint(TRUNCATE)", ())
                    .await?;
                while rows.next().await?.is_some() {}
                Ok(())
            }
            .await
            .map_err(|e| match e {
                Error::Database(e) => classify_db_error(e),
                other => other,
            });

            match result {
                Ok(()) => return Ok(()),
                Err(Error::LockContention) if attempt < max_retries - 1 => {
                    tokio::time::sleep(delay).await;
                }
                Err(Error::LockContention) => {
                    tracing::warn!(
                        "checkpoint failed after {} retries due to lock contention",
                        max_retries
                    );
                    return Ok(());
                }
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    pub fn conn(&self) -> &turso::Connection {
        &self.conn
    }
}
