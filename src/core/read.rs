use serde::Serialize;

use crate::db::{Database, branch, comment, issue};
use crate::error::{Error, Result};
use crate::git;

#[derive(Debug, Serialize)]
pub struct IssueThread {
    pub issue: issue::Issue,
    pub comments: Vec<comment::IssueComment>,
}

#[derive(Debug, Serialize)]
pub struct BranchThread {
    pub branch: branch::Branch,
    pub comments: Vec<comment::BranchComment>,
}

pub async fn read_issue(db: &Database, issue_id: i64) -> Result<issue::Issue> {
    let conn = db.connect()?;
    issue::get_by_id(&conn, issue_id)
        .await?
        .ok_or_else(|| Error::not_found(format!("issue {issue_id} not found")))
}

pub async fn read_issue_thread(db: &Database, issue_id: i64) -> Result<IssueThread> {
    let conn = db.connect()?;
    let iss = issue::get_by_id(&conn, issue_id)
        .await?
        .ok_or_else(|| Error::not_found(format!("issue {issue_id} not found")))?;
    let comments = comment::list_issue_comments(&conn, iss.issue_id).await?;
    Ok(IssueThread {
        issue: iss,
        comments,
    })
}

pub async fn read_branch(db: &Database, repo: &git2::Repository) -> Result<branch::Branch> {
    let branch_name = git::current_branch(repo)?;
    let conn = db.connect()?;
    branch::get_by_name(&conn, &branch_name)
        .await?
        .ok_or_else(|| Error::not_found(format!("branch '{branch_name}' not found in database")))
}

pub async fn read_branch_thread(db: &Database, repo: &git2::Repository) -> Result<BranchThread> {
    let branch_name = git::current_branch(repo)?;
    let conn = db.connect()?;
    let br = branch::get_by_name(&conn, &branch_name)
        .await?
        .ok_or_else(|| Error::not_found(format!("branch '{branch_name}' not found in database")))?;
    let comments = comment::list_branch_comments(&conn, br.branch_id).await?;
    Ok(BranchThread {
        branch: br,
        comments,
    })
}
