use clap::Parser;

use doppelganger::cli::{
    Cli, Command, CreateCommand, CreateComment, ReadBranch, ReadCommand, ReadIssue,
};
use doppelganger::core;
use doppelganger::db::Database;
use doppelganger::error::Result;
use doppelganger::git;
use doppelganger::output;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let repo = git::discover_repo(std::path::Path::new("."))?;
    let repo_root = git::repo_root(&repo)?;

    let db = Database::open(repo_root).await?;

    match cli.command {
        Command::Create { subcommand } => handle_create(subcommand, &db, repo).await?,
        Command::Read { subcommand } => handle_read(subcommand, &db, repo).await?,
    }

    Ok(())
}

async fn handle_create(
    cmd: CreateCommand,
    db: &Database,
    mut repo: git2::Repository,
) -> Result<()> {
    match cmd {
        CreateCommand::Issue(args) => {
            let description =
                core::create::read_text(args.description.as_deref(), args.file.as_deref())?;
            let issue =
                core::create::create_issue(db, &repo, args.name.as_deref(), &description).await?;
            output::output(&issue)?;
        }
        CreateCommand::Branch(args) => {
            let description =
                core::create::read_text(args.description.as_deref(), args.file.as_deref())?;
            let br =
                core::create::create_branch(db, &mut repo, &args.name, &description, args.issue_id)
                    .await?;
            output::output(&br)?;
        }
        CreateCommand::Comment(comment_cmd) => match comment_cmd {
            CreateComment::Issue(args) => {
                let content =
                    core::create::read_text(args.content.as_deref(), args.file.as_deref())?;
                let comment =
                    core::create::create_issue_comment(db, &repo, args.issue_id, &content).await?;
                output::output(&comment)?;
            }
            CreateComment::Branch(args) => {
                let content =
                    core::create::read_text(args.content.as_deref(), args.file.as_deref())?;
                let comment = core::create::create_branch_comment(db, &repo, &content).await?;
                output::output(&comment)?;
            }
        },
    }
    Ok(())
}

async fn handle_read(cmd: ReadCommand, db: &Database, repo: git2::Repository) -> Result<()> {
    match cmd {
        ReadCommand::Issue { subcommand } => match subcommand {
            ReadIssue::Single(args) => {
                let issue = core::read::read_issue(db, args.issue_id).await?;
                output::output(&issue)?;
            }
            ReadIssue::Thread(args) => {
                let thread = core::read::read_issue_thread(db, args.issue_id).await?;
                output::output(&thread)?;
            }
        },
        ReadCommand::Branch { subcommand } => match subcommand {
            ReadBranch::Single(_) => {
                let br = core::read::read_branch(db, &repo).await?;
                output::output(&br)?;
            }
            ReadBranch::Thread(_) => {
                let thread = core::read::read_branch_thread(db, &repo).await?;
                output::output(&thread)?;
            }
        },
    }
    Ok(())
}
