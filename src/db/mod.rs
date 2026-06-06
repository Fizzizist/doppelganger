pub mod author;
pub mod branch;
pub mod comment;
pub mod fingerprint;
pub mod issue;
pub mod models;
pub mod row;
pub mod schema;

use std::time::Duration;

use crate::error::{Error, Result};

const LOCK_RETRY_ATTEMPTS: u32 = 200;
const LOCK_RETRY_DELAY: Duration = Duration::from_millis(25);

pub struct Database {
    conn: turso::Connection,
}

impl Database {
    pub async fn open(path: &str) -> Result<Self> {
        let mut attempt = 0u32;
        loop {
            match Self::try_open(path).await {
                Ok(db) => return Ok(db),
                Err(e) if is_lock_contention(&e) && attempt < LOCK_RETRY_ATTEMPTS => {
                    attempt += 1;
                    tokio::time::sleep(LOCK_RETRY_DELAY).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn try_open(path: &str) -> Result<Self> {
        let db = turso::Builder::new_local(path)
            .experimental_multiprocess_wal(true)
            .build()
            .await
            .map_err(classify_db_error)?;
        let conn = db.connect().map_err(classify_db_error)?;
        let database = Self { conn };
        database.prepare().await.map_err(classify_app_error)?;
        Ok(database)
    }

    pub async fn open_in_memory() -> Result<Self> {
        let db = turso::Builder::new_local(":memory:").build().await?;
        let conn = db.connect()?;
        let database = Self { conn };
        database.prepare().await?;
        Ok(database)
    }

    async fn prepare(&self) -> Result<()> {
        self.conn.execute("PRAGMA busy_timeout=5000", ()).await?;
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
        // A TRUNCATE checkpoint needs exclusive WAL access. Under concurrent
        // access it can be blocked; treat checkpointing as best-effort so a
        // transient lock never fails an otherwise-successful command.
        let mut attempt = 0u32;
        loop {
            match self.try_checkpoint().await {
                Ok(()) => return Ok(()),
                Err(e) if is_lock_contention(&e) && attempt < LOCK_RETRY_ATTEMPTS => {
                    attempt += 1;
                    tokio::time::sleep(LOCK_RETRY_DELAY).await;
                }
                // Give up on a persistent lock rather than failing the command.
                Err(e) if is_lock_contention(&e) => {
                    crate::log!(
                        "checkpoint: exhausted {} lock retries: {}",
                        LOCK_RETRY_ATTEMPTS,
                        e
                    );
                    return Ok(());
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn try_checkpoint(&self) -> Result<()> {
        let mut rows = self
            .conn
            .query("PRAGMA wal_checkpoint(TRUNCATE)", ())
            .await
            .map_err(classify_db_error)?;
        while rows.next().await.map_err(classify_db_error)?.is_some() {}
        Ok(())
    }

    pub fn conn(&self) -> &turso::Connection {
        &self.conn
    }
}

fn is_lock_contention(err: &Error) -> bool {
    matches!(err, Error::LockContention)
}

fn classify_db_error(err: turso::Error) -> Error {
    if let turso::Error::Error(ref msg) = err
        && (msg.contains("Locking error") || msg.contains("locked by another process"))
    {
        return Error::LockContention;
    }
    Error::Database(err)
}

fn classify_app_error(err: Error) -> Error {
    if let Error::Database(turso::Error::Error(ref msg)) = err
        && (msg.contains("Locking error") || msg.contains("locked by another process"))
    {
        return Error::LockContention;
    }
    err
}
