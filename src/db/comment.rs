use crate::error::{Error, Result};
use turso::Value;

use super::{
    models::{BranchComment, IssueComment},
    row::{extract_int, extract_optional_text, extract_text},
};

const SELECT_ISSUE_COMMENT: &str = r#"
SELECT issue_comment.issue_comment_id, issue_comment.content, author.name,
       issue_comment.issue_id, issue_comment.created_at, issue_comment.updated_at,
       issue_comment.hidden_at
FROM issue_comment JOIN author ON issue_comment.author_id = author.author_id
"#;

const SELECT_BRANCH_COMMENT: &str = r#"
SELECT branch_comment.branch_comment_id, branch_comment.content, author.name,
       branch_comment.branch_id, branch_comment.created_at, branch_comment.updated_at,
       branch_comment.hidden_at
FROM branch_comment JOIN author ON branch_comment.author_id = author.author_id
"#;

pub async fn delete_issue_comments(conn: &turso::Connection, issue_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM issue_comment WHERE issue_id = ?1",
        turso::params::Params::Positional(vec![Value::Integer(issue_id)]),
    )
    .await?;
    Ok(())
}

pub async fn create_issue_comment(
    conn: &turso::Connection,
    issue_id: i64,
    content: &str,
    author_id: i64,
) -> Result<IssueComment> {
    conn.execute(
        "INSERT INTO issue_comment (issue_id, content, author_id) VALUES (?1, ?2, ?3)",
        turso::params::Params::Positional(vec![
            Value::Integer(issue_id),
            Value::Text(content.to_string()),
            Value::Integer(author_id),
        ]),
    )
    .await?;

    let comment_id = conn.last_insert_rowid();
    let mut rows = conn
        .query(
            format!("{SELECT_ISSUE_COMMENT} WHERE issue_comment.issue_comment_id = ?1"),
            turso::params::Params::Positional(vec![Value::Integer(comment_id)]),
        )
        .await?;

    match rows.next().await? {
        Some(row) => row_to_issue_comment(&row),
        None => Err(Error::Database(turso::Error::ConversionFailure(
            "just-inserted issue_comment could not be read back".to_string(),
        ))),
    }
}

pub async fn list_issue_comments(
    conn: &turso::Connection,
    issue_id: i64,
    show_hidden: bool,
) -> Result<Vec<IssueComment>> {
    let hidden_filter = if show_hidden {
        ""
    } else {
        " AND issue_comment.hidden_at IS NULL"
    };
    let mut rows = conn
        .query(
            format!(
                "{SELECT_ISSUE_COMMENT} WHERE issue_comment.issue_id = ?1{hidden_filter} \
                 ORDER BY issue_comment.created_at ASC"
            ),
            turso::params::Params::Positional(vec![Value::Integer(issue_id)]),
        )
        .await?;

    let mut comments = Vec::new();
    while let Some(row) = rows.next().await? {
        comments.push(row_to_issue_comment(&row)?);
    }
    Ok(comments)
}

pub async fn set_issue_comment_hidden(
    conn: &turso::Connection,
    issue_comment_id: i64,
    hidden: bool,
) -> Result<()> {
    let sql = if hidden {
        "UPDATE issue_comment SET hidden_at = datetime('now') WHERE issue_comment_id = ?1"
    } else {
        "UPDATE issue_comment SET hidden_at = NULL WHERE issue_comment_id = ?1"
    };
    conn.execute(
        sql,
        turso::params::Params::Positional(vec![Value::Integer(issue_comment_id)]),
    )
    .await?;
    Ok(())
}

pub async fn create_branch_comment(
    conn: &turso::Connection,
    branch_id: i64,
    content: &str,
    author_id: i64,
) -> Result<BranchComment> {
    conn.execute(
        "INSERT INTO branch_comment (branch_id, content, author_id) VALUES (?1, ?2, ?3)",
        turso::params::Params::Positional(vec![
            Value::Integer(branch_id),
            Value::Text(content.to_string()),
            Value::Integer(author_id),
        ]),
    )
    .await?;

    let comment_id = conn.last_insert_rowid();
    let mut rows = conn
        .query(
            format!("{SELECT_BRANCH_COMMENT} WHERE branch_comment.branch_comment_id = ?1"),
            turso::params::Params::Positional(vec![Value::Integer(comment_id)]),
        )
        .await?;

    match rows.next().await? {
        Some(row) => row_to_branch_comment(&row),
        None => Err(Error::Database(turso::Error::ConversionFailure(
            "just-inserted branch_comment could not be read back".to_string(),
        ))),
    }
}

pub async fn list_branch_comments(
    conn: &turso::Connection,
    branch_id: i64,
    show_hidden: bool,
) -> Result<Vec<BranchComment>> {
    let hidden_filter = if show_hidden {
        ""
    } else {
        " AND branch_comment.hidden_at IS NULL"
    };
    let mut rows = conn
        .query(
            format!(
                "{SELECT_BRANCH_COMMENT} WHERE branch_comment.branch_id = ?1{hidden_filter} \
                 ORDER BY branch_comment.created_at ASC"
            ),
            turso::params::Params::Positional(vec![Value::Integer(branch_id)]),
        )
        .await?;

    let mut comments = Vec::new();
    while let Some(row) = rows.next().await? {
        comments.push(row_to_branch_comment(&row)?);
    }
    Ok(comments)
}

pub async fn set_branch_comment_hidden(
    conn: &turso::Connection,
    branch_comment_id: i64,
    hidden: bool,
) -> Result<()> {
    let sql = if hidden {
        "UPDATE branch_comment SET hidden_at = datetime('now') WHERE branch_comment_id = ?1"
    } else {
        "UPDATE branch_comment SET hidden_at = NULL WHERE branch_comment_id = ?1"
    };
    conn.execute(
        sql,
        turso::params::Params::Positional(vec![Value::Integer(branch_comment_id)]),
    )
    .await?;
    Ok(())
}

pub async fn update_issue_comment(
    conn: &turso::Connection,
    issue_comment_id: i64,
    content: &str,
) -> Result<IssueComment> {
    conn.execute(
        "UPDATE issue_comment SET content = ?1, updated_at = datetime('now') \
         WHERE issue_comment_id = ?2",
        turso::params::Params::Positional(vec![
            Value::Text(content.to_string()),
            Value::Integer(issue_comment_id),
        ]),
    )
    .await?;

    let mut rows = conn
        .query(
            format!("{SELECT_ISSUE_COMMENT} WHERE issue_comment.issue_comment_id = ?1"),
            turso::params::Params::Positional(vec![Value::Integer(issue_comment_id)]),
        )
        .await?;

    match rows.next().await? {
        Some(row) => row_to_issue_comment(&row),
        None => Err(Error::Database(turso::Error::ConversionFailure(format!(
            "issue_comment {issue_comment_id} not found after update"
        )))),
    }
}

pub async fn update_branch_comment(
    conn: &turso::Connection,
    branch_comment_id: i64,
    content: &str,
) -> Result<BranchComment> {
    conn.execute(
        "UPDATE branch_comment SET content = ?1, updated_at = datetime('now') \
         WHERE branch_comment_id = ?2",
        turso::params::Params::Positional(vec![
            Value::Text(content.to_string()),
            Value::Integer(branch_comment_id),
        ]),
    )
    .await?;

    let mut rows = conn
        .query(
            format!("{SELECT_BRANCH_COMMENT} WHERE branch_comment.branch_comment_id = ?1"),
            turso::params::Params::Positional(vec![Value::Integer(branch_comment_id)]),
        )
        .await?;

    match rows.next().await? {
        Some(row) => row_to_branch_comment(&row),
        None => Err(Error::Database(turso::Error::ConversionFailure(format!(
            "branch_comment {branch_comment_id} not found after update"
        )))),
    }
}

fn row_to_issue_comment(row: &turso::Row) -> Result<IssueComment> {
    Ok(IssueComment {
        issue_comment_id: extract_int(row, 0)?,
        content: extract_text(row, 1)?,
        author: extract_text(row, 2)?,
        issue_id: extract_int(row, 3)?,
        created_at: extract_text(row, 4)?,
        updated_at: extract_text(row, 5)?,
        hidden_at: extract_optional_text(row, 6)?,
    })
}

fn row_to_branch_comment(row: &turso::Row) -> Result<BranchComment> {
    Ok(BranchComment {
        branch_comment_id: extract_int(row, 0)?,
        content: extract_text(row, 1)?,
        author: extract_text(row, 2)?,
        branch_id: extract_int(row, 3)?,
        created_at: extract_text(row, 4)?,
        updated_at: extract_text(row, 5)?,
        hidden_at: extract_optional_text(row, 6)?,
    })
}

#[cfg(test)]
mod tests {
    use crate::db::{Database, author, branch, issue};

    #[tokio::test]
    async fn update_issue_comment_changes_content_and_preserves_author() {
        let db = Database::open_in_memory().await.expect("open in-memory db");
        let conn = db.conn();

        let author = author::find_or_create(conn, "Alice", Some("a@b.com"))
            .await
            .expect("create author");
        let iss = issue::create(conn, None, "desc", author.author_id, None)
            .await
            .expect("create issue");

        let c = super::create_issue_comment(conn, iss.issue_id, "original", author.author_id)
            .await
            .expect("create comment");
        let original_updated_at = c.updated_at.clone();

        let updated = super::update_issue_comment(conn, c.issue_comment_id, "edited")
            .await
            .expect("update comment");

        assert_eq!(updated.content, "edited");
        assert_eq!(updated.author, "Alice");
        assert_eq!(updated.issue_id, iss.issue_id);
        assert!(
            updated.updated_at >= original_updated_at,
            "updated_at must not go backwards"
        );
    }

    #[tokio::test]
    async fn update_branch_comment_changes_content_and_preserves_author() {
        let db = Database::open_in_memory().await.expect("open in-memory db");
        let conn = db.conn();

        let author = author::find_or_create(conn, "Bob", Some("b@b.com"))
            .await
            .expect("create author");
        let iss = issue::create(conn, None, "desc", author.author_id, None)
            .await
            .expect("create issue");
        let br = branch::create(conn, "feature", "desc", author.author_id, iss.issue_id)
            .await
            .expect("create branch");

        let c = super::create_branch_comment(conn, br.branch_id, "original", author.author_id)
            .await
            .expect("create comment");
        let original_updated_at = c.updated_at.clone();

        let updated = super::update_branch_comment(conn, c.branch_comment_id, "edited")
            .await
            .expect("update comment");

        assert_eq!(updated.content, "edited");
        assert_eq!(updated.author, "Bob");
        assert_eq!(updated.branch_id, br.branch_id);
        assert!(
            updated.updated_at >= original_updated_at,
            "updated_at must not go backwards"
        );
    }

    #[tokio::test]
    async fn set_hidden_and_list_issue_comments_filter() {
        let db = Database::open_in_memory().await.expect("open in-memory db");
        let conn = db.conn();

        let author = author::find_or_create(conn, "Alice", Some("a@b.com"))
            .await
            .expect("create author");
        let iss = issue::create(conn, None, "desc", author.author_id, None)
            .await
            .expect("create issue");

        let c1 = super::create_issue_comment(conn, iss.issue_id, "visible", author.author_id)
            .await
            .expect("create c1");
        let c2 = super::create_issue_comment(conn, iss.issue_id, "to hide", author.author_id)
            .await
            .expect("create c2");

        super::set_issue_comment_hidden(conn, c2.issue_comment_id, true)
            .await
            .expect("hide c2");

        let visible = super::list_issue_comments(conn, iss.issue_id, false)
            .await
            .expect("list visible");
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].content, "visible");

        let all = super::list_issue_comments(conn, iss.issue_id, true)
            .await
            .expect("list all");
        assert_eq!(all.len(), 2);
        let hidden_comment = all
            .iter()
            .find(|c| c.issue_comment_id == c2.issue_comment_id)
            .expect("find c2");
        assert!(hidden_comment.hidden_at.is_some());

        super::set_issue_comment_hidden(conn, c2.issue_comment_id, false)
            .await
            .expect("unhide c2");

        let after_unhide = super::list_issue_comments(conn, iss.issue_id, false)
            .await
            .expect("list after unhide");
        assert_eq!(after_unhide.len(), 2, "both visible after unhide");
        let c1_idx = after_unhide
            .iter()
            .position(|c| c.issue_comment_id == c1.issue_comment_id)
            .expect("find c1");
        assert!(after_unhide[c1_idx].hidden_at.is_none());
    }

    #[tokio::test]
    async fn set_hidden_and_list_branch_comments_filter() {
        let db = Database::open_in_memory().await.expect("open in-memory db");
        let conn = db.conn();

        let author = author::find_or_create(conn, "Bob", Some("b@b.com"))
            .await
            .expect("create author");
        let iss = issue::create(conn, None, "desc", author.author_id, None)
            .await
            .expect("create issue");
        let br = branch::create(conn, "feat", "desc", author.author_id, iss.issue_id)
            .await
            .expect("create branch");

        let c1 = super::create_branch_comment(conn, br.branch_id, "visible", author.author_id)
            .await
            .expect("create c1");
        let c2 = super::create_branch_comment(conn, br.branch_id, "to hide", author.author_id)
            .await
            .expect("create c2");

        super::set_branch_comment_hidden(conn, c2.branch_comment_id, true)
            .await
            .expect("hide c2");

        let visible = super::list_branch_comments(conn, br.branch_id, false)
            .await
            .expect("list visible");
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].content, "visible");

        let all = super::list_branch_comments(conn, br.branch_id, true)
            .await
            .expect("list all");
        assert_eq!(all.len(), 2);
        let hidden_comment = all
            .iter()
            .find(|c| c.branch_comment_id == c2.branch_comment_id)
            .expect("find c2");
        assert!(hidden_comment.hidden_at.is_some());

        super::set_branch_comment_hidden(conn, c2.branch_comment_id, false)
            .await
            .expect("unhide c2");

        let after_unhide = super::list_branch_comments(conn, br.branch_id, false)
            .await
            .expect("list after unhide");
        assert_eq!(after_unhide.len(), 2, "both visible after unhide");
        let c1_idx = after_unhide
            .iter()
            .position(|c| c.branch_comment_id == c1.branch_comment_id)
            .expect("find c1");
        assert!(after_unhide[c1_idx].hidden_at.is_none());
    }
}
