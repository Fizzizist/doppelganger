use crate::{
    cli::IssueCommands,
    db::{Database, author, comment, issue, models::IssueWithComments},
    error::Result,
    input::resolve_content,
    output::print_json,
};

pub async fn handle(
    cmd: IssueCommands,
    db: &Database,
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
        IssueCommands::Tui => {
            unreachable!("TUI is handled directly in main.rs")
        }
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
    let created = issue::create(conn, name.as_deref(), &description, author.author_id).await?;
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
