use crate::{
    cli::BranchCommands,
    db::{Database, author, branch, comment, issue, models::BranchWithComments},
    error::{Error, Result},
    git::current_branch,
    input::resolve_content,
    output::print_json,
    tui::{Thread, ThreadComment, app::App},
};

pub async fn handle(
    cmd: BranchCommands,
    db: &Database,
    repo: &git2::Repository,
    author_name: &str,
    author_email: Option<&str>,
) -> Result<()> {
    match cmd {
        BranchCommands::Create {
            issue_number,
            description,
            overwrite,
        } => {
            create(
                db,
                repo,
                author_name,
                author_email,
                issue_number,
                description,
                overwrite,
            )
            .await
        }
        BranchCommands::Read => read(db, repo).await,
        BranchCommands::Comment { content } => {
            comment_cmd(db, repo, author_name, author_email, content).await
        }
        BranchCommands::Tui => tui(db, repo).await,
    }
}

async fn create(
    db: &Database,
    repo: &git2::Repository,
    author_name: &str,
    author_email: Option<&str>,
    issue_number: i64,
    description: Option<String>,
    overwrite: bool,
) -> Result<()> {
    let branch_name = current_branch(repo)?;
    let description_text = resolve_content(description)?;
    let conn = db.conn();

    issue::get_by_id(conn, issue_number).await?;
    let author = author::find_or_create(conn, author_name, author_email).await?;

    let created = if overwrite {
        match branch::get_by_name(conn, &branch_name).await {
            Ok(_existing) => {
                branch::update_description(conn, &branch_name, &description_text).await?
            }
            Err(Error::BranchNotFound(_)) => {
                branch::create(
                    conn,
                    &branch_name,
                    &description_text,
                    author.author_id,
                    issue_number,
                )
                .await?
            }
            Err(e) => return Err(e),
        }
    } else {
        branch::create(
            conn,
            &branch_name,
            &description_text,
            author.author_id,
            issue_number,
        )
        .await?
    };
    print_json(&created)
}

async fn read(db: &Database, repo: &git2::Repository) -> Result<()> {
    let branch_name = current_branch(repo)?;
    let conn = db.conn();
    let br = branch::get_by_name(conn, &branch_name).await?;
    let comments = comment::list_branch_comments(conn, br.branch_id).await?;
    print_json(&BranchWithComments {
        branch: br,
        comments,
    })
}

async fn comment_cmd(
    db: &Database,
    repo: &git2::Repository,
    author_name: &str,
    author_email: Option<&str>,
    content: Option<String>,
) -> Result<()> {
    let branch_name = current_branch(repo)?;
    let text = resolve_content(content)?;
    let conn = db.conn();
    let br = branch::get_by_name(conn, &branch_name).await?;
    let author = author::find_or_create(conn, author_name, author_email).await?;
    let created =
        comment::create_branch_comment(conn, br.branch_id, &text, author.author_id).await?;
    print_json(&created)
}

async fn tui(db: &Database, repo: &git2::Repository) -> Result<()> {
    let branch_name = current_branch(repo)?;
    let conn = db.conn();
    let br = branch::get_by_name(conn, &branch_name).await?;
    let comments = comment::list_branch_comments(conn, br.branch_id).await?;
    let thread = Thread::from(br);
    let thread_comments: Vec<ThreadComment> =
        comments.into_iter().map(ThreadComment::from).collect();
    let mut app = App::new_thread_view(thread, thread_comments, db);
    app.run().await
}
