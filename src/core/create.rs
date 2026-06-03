use std::io::{self, IsTerminal, Read as IoRead};
use std::path::Path;

use crate::db::{Database, author, branch, comment, issue};
use crate::error::{Error, Result};
use crate::git;

pub async fn create_issue(
    db: &Database,
    repo: &git2::Repository,
    name: Option<&str>,
    description: &str,
) -> Result<issue::Issue> {
    let git_author = git::get_author(repo)?;
    let conn = db.connect()?;
    let author =
        author::find_or_create(&conn, &git_author.name, git_author.email.as_deref()).await?;
    let result = issue::create(&conn, name, description, author.author_id).await?;
    Ok(result)
}

pub async fn create_branch(
    db: &Database,
    repo: &mut git2::Repository,
    name: &str,
    description: &str,
    issue_id: i64,
) -> Result<branch::Branch> {
    let git_author = git::get_author(repo)?;
    let conn = db.connect()?;

    let existing_issue = issue::get_by_id(&conn, issue_id)
        .await?
        .ok_or_else(|| Error::not_found(format!("issue {issue_id} not found")))?;

    let author =
        author::find_or_create(&conn, &git_author.name, git_author.email.as_deref()).await?;

    git::create_and_checkout_branch(repo, name)?;

    let result = branch::create(
        &conn,
        name,
        description,
        author.author_id,
        existing_issue.issue_id,
    )
    .await?;
    Ok(result)
}

pub async fn create_issue_comment(
    db: &Database,
    repo: &git2::Repository,
    issue_id: i64,
    content: &str,
) -> Result<comment::IssueComment> {
    let git_author = git::get_author(repo)?;
    let conn = db.connect()?;

    let existing_issue = issue::get_by_id(&conn, issue_id)
        .await?
        .ok_or_else(|| Error::not_found(format!("issue {issue_id} not found")))?;

    let author =
        author::find_or_create(&conn, &git_author.name, git_author.email.as_deref()).await?;

    let result =
        comment::create_issue_comment(&conn, content, author.author_id, existing_issue.issue_id)
            .await?;
    Ok(result)
}

pub async fn create_branch_comment(
    db: &Database,
    repo: &git2::Repository,
    content: &str,
) -> Result<comment::BranchComment> {
    let git_author = git::get_author(repo)?;
    let branch_name = git::current_branch(repo)?;
    let conn = db.connect()?;

    let existing_branch = branch::get_by_name(&conn, &branch_name)
        .await?
        .ok_or_else(|| Error::not_found(format!("branch '{branch_name}' not found in database")))?;

    let author =
        author::find_or_create(&conn, &git_author.name, git_author.email.as_deref()).await?;

    let result =
        comment::create_branch_comment(&conn, content, author.author_id, existing_branch.branch_id)
            .await?;
    Ok(result)
}

pub fn read_text(input: Option<&str>, file: Option<&Path>) -> Result<String> {
    match (input, file) {
        (Some(text), _) => Ok(text.to_string()),
        (None, Some(path)) => {
            let mut content = String::new();
            let mut f = std::fs::File::open(path).map_err(|e| {
                Error::validation(format!("cannot read file {}: {}", path.display(), e))
            })?;
            f.read_to_string(&mut content).map_err(|e| {
                Error::validation(format!("cannot read file {}: {}", path.display(), e))
            })?;
            Ok(content)
        }
        (None, None) => {
            if io::stdin().is_terminal() {
                Err(Error::validation(
                    "no text provided; use --description/--content, --file, or pipe stdin",
                ))
            } else {
                let mut content = String::new();
                io::stdin()
                    .read_to_string(&mut content)
                    .map_err(|e| Error::validation(format!("cannot read stdin: {e}")))?;
                Ok(content)
            }
        }
    }
}
