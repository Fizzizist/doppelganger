use crate::{
    cli::IssueCommands,
    config::Config,
    db::{Database, author, comment, issue, models::IssueWithComments},
    error::Result,
    git,
    input::resolve_content,
    output::print_json,
    remote::{self, Provider},
};

pub async fn handle(
    cmd: IssueCommands,
    db: &Database,
    config: &Config,
    repo: &git2::Repository,
    author_name: &str,
    author_email: Option<&str>,
) -> Result<()> {
    match cmd {
        IssueCommands::Create { content, name } => {
            create(db, author_name, author_email, content, name).await
        }
        IssueCommands::Read { issue_number } => read(db, issue_number).await,
        IssueCommands::Comment {
            issue_number,
            content,
        } => comment(db, author_name, author_email, issue_number, content).await,
        IssueCommands::Sync {
            issue_number,
            overwrite,
        } => sync(db, config, repo, issue_number, overwrite).await,
        IssueCommands::Tui => Err(crate::error::Error::Validation(
            "issue tui is handled before DB open".to_string(),
        )),
    }
}

async fn create(
    db: &Database,
    author_name: &str,
    author_email: Option<&str>,
    content: Option<String>,
    name: Option<String>,
) -> Result<()> {
    let description = resolve_content(content)?;
    let conn = db.conn();
    let author = author::find_or_create(conn, author_name, author_email).await?;
    let created =
        issue::create(conn, name.as_deref(), &description, author.author_id, None).await?;
    print_json(&created)
}

async fn read(db: &Database, issue_number: i64) -> Result<()> {
    let conn = db.conn();
    let iss = issue::get_by_id(conn, issue_number).await?;
    let comments = comment::list_issue_comments(conn, iss.issue_id).await?;
    print_json(&IssueWithComments {
        issue: iss,
        comments,
    })
}

async fn comment(
    db: &Database,
    author_name: &str,
    author_email: Option<&str>,
    issue_number: i64,
    content: Option<String>,
) -> Result<()> {
    let text = resolve_content(content)?;
    let conn = db.conn();
    let author = author::find_or_create(conn, author_name, author_email).await?;
    issue::get_by_id(conn, issue_number).await?;
    let created =
        comment::create_issue_comment(conn, issue_number, &text, author.author_id).await?;
    print_json(&created)
}

async fn sync(
    db: &Database,
    config: &Config,
    repo: &git2::Repository,
    issue_number: i64,
    overwrite: Option<i64>,
) -> Result<()> {
    if issue_number <= 0 {
        return Err(crate::error::Error::Validation(
            "issue number must be a positive integer".to_string(),
        ));
    }

    let github_config = config.github.as_ref().ok_or_else(|| {
        crate::error::Error::RemoteSync(
            "GitHub token not configured; add a [github] section with token to your config"
                .to_string(),
        )
    })?;

    let url = git::remote_url(repo)?;
    let (owner, repo_name) = remote::github::parse_github_remote_url(&url)?;

    let provider = remote::github::GitHubProvider::new(&github_config.token, &owner, &repo_name)?;

    let remote_issue = provider.fetch_issue(issue_number).await?;
    let conn = db.conn();

    if let Some(local_id) = overwrite {
        let existing = issue::get_by_id(conn, local_id).await?;
        let sync_author = author::find_or_create(conn, &remote_issue.author, None).await?;
        let updated = issue::update_for_sync(
            conn,
            existing.issue_id,
            remote_issue.title.as_deref(),
            &remote_issue.body,
            sync_author.author_id,
            Some(&remote_issue.remote_id),
        )
        .await?;

        comment::delete_issue_comments(conn, existing.issue_id).await?;
        for c in &remote_issue.comments {
            let comment_author = author::find_or_create(conn, &c.author, None).await?;
            comment::create_issue_comment(
                conn,
                existing.issue_id,
                &c.body,
                comment_author.author_id,
            )
            .await?;
        }

        let comments = comment::list_issue_comments(conn, updated.issue_id).await?;
        print_json(&IssueWithComments {
            issue: updated,
            comments,
        })
    } else {
        let sync_author = author::find_or_create(conn, &remote_issue.author, None).await?;
        let created = issue::create(
            conn,
            remote_issue.title.as_deref(),
            &remote_issue.body,
            sync_author.author_id,
            Some(&remote_issue.remote_id),
        )
        .await?;

        for c in &remote_issue.comments {
            let comment_author = author::find_or_create(conn, &c.author, None).await?;
            comment::create_issue_comment(
                conn,
                created.issue_id,
                &c.body,
                comment_author.author_id,
            )
            .await?;
        }

        let comments = comment::list_issue_comments(conn, created.issue_id).await?;
        print_json(&IssueWithComments {
            issue: created,
            comments,
        })
    }
}
