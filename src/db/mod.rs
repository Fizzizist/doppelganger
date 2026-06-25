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
    path: Option<String>,
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
                let database = Self {
                    conn,
                    path: Some(path.to_string()),
                };
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
        let database = Self { conn, path: None };
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
        self.migrate_archived_at().await?;
        self.migrate_hidden_at().await?;
        self.migrate_branch_drop_unique().await?;
        self.migrate_branch_archived_at().await?;
        self.migrate_branch_active_index().await?;
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

    async fn migrate_archived_at(&self) -> Result<()> {
        match self
            .conn
            .execute(schema::ALTER_TABLE_ISSUE_ARCHIVED_AT, ())
            .await
        {
            Ok(_) => Ok(()),
            Err(e) if e.to_string().contains("duplicate column name") => Ok(()),
            Err(e) => Err(classify_db_error(e)),
        }
    }

    async fn migrate_hidden_at(&self) -> Result<()> {
        let stmts = [
            schema::ALTER_TABLE_ISSUE_COMMENT_HIDDEN_AT,
            schema::ALTER_TABLE_BRANCH_COMMENT_HIDDEN_AT,
        ];
        for stmt in stmts {
            match self.conn.execute(stmt, ()).await {
                Ok(_) => {}
                Err(e) if e.to_string().contains("duplicate column name") => {}
                Err(e) => return Err(classify_db_error(e)),
            }
        }
        Ok(())
    }

    async fn migrate_branch_archived_at(&self) -> Result<()> {
        match self
            .conn
            .execute(schema::ALTER_TABLE_BRANCH_ARCHIVED_AT, ())
            .await
        {
            Ok(_) => Ok(()),
            Err(e) if e.to_string().contains("duplicate column name") => Ok(()),
            Err(e) => Err(classify_db_error(e)),
        }
    }

    async fn migrate_branch_drop_unique(&self) -> Result<()> {
        let mut rows = self
            .conn
            .query(
                "SELECT sql FROM sqlite_master WHERE type='table' AND name='branch'",
                (),
            )
            .await?;

        let table_sql = match rows.next().await? {
            Some(row) => {
                let val = row.get_value(0)?;
                match val {
                    turso::Value::Text(s) => s,
                    _ => return Ok(()),
                }
            }
            None => return Ok(()),
        };

        if !table_sql
            .to_lowercase()
            .contains("name text not null unique")
        {
            return Ok(());
        }

        tracing::info!("recreating branch table to drop UNIQUE constraint on name");

        let Some(db_path) = &self.path else {
            return Ok(());
        };
        self.checkpoint().await?;

        let non_wal_db = turso::Builder::new_local(db_path).build().await?;
        let non_wal_conn = non_wal_db.connect()?;

        non_wal_conn
            .execute("PRAGMA foreign_keys = OFF", ())
            .await?;
        non_wal_conn.execute(schema::BRANCH_TABLE_NEW, ()).await?;
        non_wal_conn
            .execute(
                "INSERT INTO branch_new (branch_id, name, description, author_id, issue_id, created_at, updated_at) \
                 SELECT branch_id, name, description, author_id, issue_id, created_at, updated_at FROM branch",
                (),
            )
            .await?;
        non_wal_conn.execute(schema::DROP_BRANCH_OLD, ()).await?;
        non_wal_conn.execute(schema::RENAME_BRANCH_NEW, ()).await?;
        non_wal_conn.execute("PRAGMA foreign_keys = ON", ()).await?;

        Ok(())
    }

    async fn migrate_branch_active_index(&self) -> Result<()> {
        match self
            .conn
            .execute(schema::CREATE_INDEX_BRANCH_ACTIVE_NAME, ())
            .await
        {
            Ok(_) => Ok(()),
            Err(e) if e.to_string().contains("already exists") => Ok(()),
            Err(turso::Error::Constraint(_)) => Err(Error::Validation(
                "cannot create unique index idx_branch_active_name: duplicate active branch names exist; \
                 archive duplicates with `dg branch archive` first"
                    .to_string(),
            )),
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
