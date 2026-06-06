use crate::error::{Error, Result};
use turso::Value;

use super::{
    models::Issue,
    row::{extract_int, extract_optional_text, extract_text},
};

pub async fn list(conn: &turso::Connection) -> Result<Vec<Issue>> {
    let mut rows = conn
        .query(
            "SELECT issue.issue_id, issue.name, issue.description, author.name, \
             issue.created_at, issue.updated_at \
             FROM issue JOIN author ON issue.author_id = author.author_id \
             ORDER BY issue.updated_at DESC, issue.issue_id DESC",
            (),
        )
        .await?;

    let mut issues = Vec::new();
    while let Some(row) = rows.next().await? {
        issues.push(row_to_issue(&row)?);
    }
    Ok(issues)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{Database, author};

    #[tokio::test]
    async fn list_returns_issues_ordered_by_updated_at_desc() {
        let db = Database::open_in_memory().await.expect("open in-memory db");
        let conn = db.conn();

        let a = author::find_or_create(conn, "Author", Some("a@b.com"))
            .await
            .expect("create author");

        let first = create(conn, None, "oldest", a.author_id)
            .await
            .expect("create first issue");
        let second = create(conn, None, "newest", a.author_id)
            .await
            .expect("create second issue");

        let issues = list(conn).await.expect("list issues");
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].issue_id, second.issue_id);
        assert_eq!(issues[1].issue_id, first.issue_id);
    }

    #[tokio::test]
    async fn list_returns_empty_when_no_issues() {
        let db = Database::open_in_memory().await.expect("open in-memory db");
        let conn = db.conn();

        let issues = list(conn).await.expect("list issues");
        assert!(issues.is_empty());
    }
}

pub async fn create(
    conn: &turso::Connection,
    name: Option<&str>,
    description: &str,
    author_id: i64,
) -> Result<Issue> {
    let name_value = match name {
        Some(n) => Value::Text(n.to_string()),
        None => Value::Null,
    };

    conn.execute(
        "INSERT INTO issue (name, description, author_id) VALUES (?1, ?2, ?3)",
        turso::params::Params::Positional(vec![
            name_value,
            Value::Text(description.to_string()),
            Value::Integer(author_id),
        ]),
    )
    .await?;

    let issue_id = conn.last_insert_rowid();
    get_by_id(conn, issue_id).await
}

pub async fn get_by_id(conn: &turso::Connection, issue_id: i64) -> Result<Issue> {
    let mut rows = conn
        .query(
            "SELECT issue.issue_id, issue.name, issue.description, author.name, \
             issue.created_at, issue.updated_at \
             FROM issue JOIN author ON issue.author_id = author.author_id \
             WHERE issue.issue_id = ?1",
            turso::params::Params::Positional(vec![Value::Integer(issue_id)]),
        )
        .await?;

    match rows.next().await? {
        Some(row) => row_to_issue(&row),
        None => Err(Error::IssueNotFound(issue_id)),
    }
}

fn row_to_issue(row: &turso::Row) -> Result<Issue> {
    Ok(Issue {
        issue_id: extract_int(row, 0)?,
        name: extract_optional_text(row, 1)?,
        description: extract_text(row, 2)?,
        author: extract_text(row, 3)?,
        created_at: extract_text(row, 4)?,
        updated_at: extract_text(row, 5)?,
    })
}
