use serde::Serialize;
use turso::Connection;

use crate::error::{Error, Result};

#[derive(Debug, Serialize)]
pub struct IssueComment {
    pub issue_comment_id: i64,
    pub content: String,
    pub author_id: i64,
    pub issue_id: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct BranchComment {
    pub branch_comment_id: i64,
    pub content: String,
    pub author_id: i64,
    pub branch_id: i64,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn create_issue_comment(
    conn: &Connection,
    content: &str,
    author_id: i64,
    issue_id: i64,
) -> Result<IssueComment> {
    conn.execute(
        "INSERT INTO issue_comment (content, author_id, issue_id) VALUES (?1, ?2, ?3)",
        turso::params::Params::Positional(vec![
            turso::Value::Text(content.to_string()),
            turso::Value::Integer(author_id),
            turso::Value::Integer(issue_id),
        ]),
    )
    .await?;

    let comment_id = conn.last_insert_rowid();

    get_issue_comment_by_id(conn, comment_id)
        .await?
        .ok_or_else(|| {
            Error::not_found(format!("issue comment {comment_id} not found after insert"))
        })
}

pub async fn get_issue_comment_by_id(
    conn: &Connection,
    comment_id: i64,
) -> Result<Option<IssueComment>> {
    let mut stmt = conn
        .prepare("SELECT issue_comment_id, content, author_id, issue_id, created_at, updated_at FROM issue_comment WHERE issue_comment_id = ?1")
        .await?;
    let mut rows = stmt
        .query(turso::params::Params::Positional(vec![
            turso::Value::Integer(comment_id),
        ]))
        .await?;

    if let Some(row) = rows.next().await? {
        Ok(Some(row_to_issue_comment(&row)?))
    } else {
        Ok(None)
    }
}

pub async fn list_issue_comments(conn: &Connection, issue_id: i64) -> Result<Vec<IssueComment>> {
    let mut stmt = conn
        .prepare("SELECT issue_comment_id, content, author_id, issue_id, created_at, updated_at FROM issue_comment WHERE issue_id = ?1 ORDER BY created_at")
        .await?;
    let mut rows = stmt
        .query(turso::params::Params::Positional(vec![
            turso::Value::Integer(issue_id),
        ]))
        .await?;

    let mut comments = Vec::new();
    while let Some(row) = rows.next().await? {
        comments.push(row_to_issue_comment(&row)?);
    }
    Ok(comments)
}

pub async fn create_branch_comment(
    conn: &Connection,
    content: &str,
    author_id: i64,
    branch_id: i64,
) -> Result<BranchComment> {
    conn.execute(
        "INSERT INTO branch_comment (content, author_id, branch_id) VALUES (?1, ?2, ?3)",
        turso::params::Params::Positional(vec![
            turso::Value::Text(content.to_string()),
            turso::Value::Integer(author_id),
            turso::Value::Integer(branch_id),
        ]),
    )
    .await?;

    let comment_id = conn.last_insert_rowid();

    get_branch_comment_by_id(conn, comment_id)
        .await?
        .ok_or_else(|| {
            Error::not_found(format!(
                "branch comment {comment_id} not found after insert"
            ))
        })
}

pub async fn get_branch_comment_by_id(
    conn: &Connection,
    comment_id: i64,
) -> Result<Option<BranchComment>> {
    let mut stmt = conn
        .prepare("SELECT branch_comment_id, content, author_id, branch_id, created_at, updated_at FROM branch_comment WHERE branch_comment_id = ?1")
        .await?;
    let mut rows = stmt
        .query(turso::params::Params::Positional(vec![
            turso::Value::Integer(comment_id),
        ]))
        .await?;

    if let Some(row) = rows.next().await? {
        Ok(Some(row_to_branch_comment(&row)?))
    } else {
        Ok(None)
    }
}

pub async fn list_branch_comments(conn: &Connection, branch_id: i64) -> Result<Vec<BranchComment>> {
    let mut stmt = conn
        .prepare("SELECT branch_comment_id, content, author_id, branch_id, created_at, updated_at FROM branch_comment WHERE branch_id = ?1 ORDER BY created_at")
        .await?;
    let mut rows = stmt
        .query(turso::params::Params::Positional(vec![
            turso::Value::Integer(branch_id),
        ]))
        .await?;

    let mut comments = Vec::new();
    while let Some(row) = rows.next().await? {
        comments.push(row_to_branch_comment(&row)?);
    }
    Ok(comments)
}

fn row_to_issue_comment(row: &turso::Row) -> Result<IssueComment> {
    let issue_comment_id = match row.get_value(0)? {
        turso::Value::Integer(i) => i,
        _ => return Err(Error::validation("issue_comment_id must be integer")),
    };
    let content = match row.get_value(1)? {
        turso::Value::Text(s) => s,
        _ => return Err(Error::validation("content must be text")),
    };
    let author_id = match row.get_value(2)? {
        turso::Value::Integer(i) => i,
        _ => return Err(Error::validation("author_id must be integer")),
    };
    let issue_id = match row.get_value(3)? {
        turso::Value::Integer(i) => i,
        _ => return Err(Error::validation("issue_id must be integer")),
    };
    let created_at = match row.get_value(4)? {
        turso::Value::Text(s) => s,
        _ => return Err(Error::validation("created_at must be text")),
    };
    let updated_at = match row.get_value(5)? {
        turso::Value::Text(s) => s,
        _ => return Err(Error::validation("updated_at must be text")),
    };

    Ok(IssueComment {
        issue_comment_id,
        content,
        author_id,
        issue_id,
        created_at,
        updated_at,
    })
}

fn row_to_branch_comment(row: &turso::Row) -> Result<BranchComment> {
    let branch_comment_id = match row.get_value(0)? {
        turso::Value::Integer(i) => i,
        _ => return Err(Error::validation("branch_comment_id must be integer")),
    };
    let content = match row.get_value(1)? {
        turso::Value::Text(s) => s,
        _ => return Err(Error::validation("content must be text")),
    };
    let author_id = match row.get_value(2)? {
        turso::Value::Integer(i) => i,
        _ => return Err(Error::validation("author_id must be integer")),
    };
    let branch_id = match row.get_value(3)? {
        turso::Value::Integer(i) => i,
        _ => return Err(Error::validation("branch_id must be integer")),
    };
    let created_at = match row.get_value(4)? {
        turso::Value::Text(s) => s,
        _ => return Err(Error::validation("created_at must be text")),
    };
    let updated_at = match row.get_value(5)? {
        turso::Value::Text(s) => s,
        _ => return Err(Error::validation("updated_at must be text")),
    };

    Ok(BranchComment {
        branch_comment_id,
        content,
        author_id,
        branch_id,
        created_at,
        updated_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{author, branch, issue};

    async fn setup_db() -> turso::Database {
        let db = turso::Builder::new_local(":memory:")
            .build()
            .await
            .expect("build db");
        let conn = db.connect().expect("connect");
        crate::db::schema::migrate(&conn).await.expect("migrate");
        db
    }

    #[tokio::test]
    async fn test_create_and_list_issue_comments() {
        let db = setup_db().await;
        let conn = db.connect().expect("connect");
        let a = author::create(&conn, "Author", None)
            .await
            .expect("create author");
        let iss = issue::create(&conn, "Issue", "desc", a.author_id)
            .await
            .expect("create issue");

        let c1 = create_issue_comment(&conn, "first comment", a.author_id, iss.issue_id)
            .await
            .expect("create comment 1");
        let c2 = create_issue_comment(&conn, "second comment", a.author_id, iss.issue_id)
            .await
            .expect("create comment 2");

        let comments = list_issue_comments(&conn, iss.issue_id)
            .await
            .expect("list comments");
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0].issue_comment_id, c1.issue_comment_id);
        assert_eq!(comments[1].issue_comment_id, c2.issue_comment_id);
    }

    #[tokio::test]
    async fn test_create_and_list_branch_comments() {
        let db = setup_db().await;
        let conn = db.connect().expect("connect");
        let a = author::create(&conn, "Author", None)
            .await
            .expect("create author");
        let iss = issue::create(&conn, "Issue", "desc", a.author_id)
            .await
            .expect("create issue");
        let br = branch::create(&conn, "my-branch", "desc", a.author_id, iss.issue_id)
            .await
            .expect("create branch");

        let c = create_branch_comment(&conn, "branch comment", a.author_id, br.branch_id)
            .await
            .expect("create comment");

        assert_eq!(c.content, "branch comment");

        let comments = list_branch_comments(&conn, br.branch_id)
            .await
            .expect("list comments");
        assert_eq!(comments.len(), 1);
    }
}
