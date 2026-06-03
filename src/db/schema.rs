use turso::Connection;

use crate::error::Result;

const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS author (
    author_id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT
);

CREATE TABLE IF NOT EXISTS issue (
    issue_id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    author_id INTEGER NOT NULL REFERENCES author(author_id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS branch (
    branch_id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    author_id INTEGER NOT NULL REFERENCES author(author_id),
    issue_id INTEGER NOT NULL REFERENCES issue(issue_id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS issue_comment (
    issue_comment_id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    author_id INTEGER NOT NULL REFERENCES author(author_id),
    issue_id INTEGER NOT NULL REFERENCES issue(issue_id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS branch_comment (
    branch_comment_id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    author_id INTEGER NOT NULL REFERENCES author(author_id),
    branch_id INTEGER NOT NULL REFERENCES branch(branch_id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
"#;

pub async fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA_SQL).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_migrate_creates_tables() {
        let db = turso::Builder::new_local(":memory:")
            .build()
            .await
            .expect("build db");
        let conn = db.connect().expect("connect");
        migrate(&conn).await.expect("migrate");

        let mut rows = conn
            .query(
                "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;",
                (),
            )
            .await
            .expect("query tables");

        let mut tables = Vec::new();
        while let Some(row) = rows.next().await.expect("next row") {
            let val = row.get_value(0).expect("get value");
            tables.push(val);
        }

        assert!(
            tables
                .iter()
                .any(|t| { matches!(t, turso::Value::Text(s) if s == "author") })
        );
        assert!(
            tables
                .iter()
                .any(|t| { matches!(t, turso::Value::Text(s) if s == "issue") })
        );
        assert!(
            tables
                .iter()
                .any(|t| { matches!(t, turso::Value::Text(s) if s == "branch") })
        );
        assert!(
            tables
                .iter()
                .any(|t| { matches!(t, turso::Value::Text(s) if s == "issue_comment") })
        );
        assert!(
            tables
                .iter()
                .any(|t| { matches!(t, turso::Value::Text(s) if s == "branch_comment") })
        );
    }
}
