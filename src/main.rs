use clap::Parser;
use doppelganger::{
    cli::{BranchCommands, Cli, Commands, IssueCommands},
    commands,
    db::Database,
    error::Error,
    git::{author_from_config, discover_repo, repo_root},
};

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

async fn run() -> doppelganger::error::Result<()> {
    let cli = Cli::parse();

    let repo = discover_repo()?;
    let root = repo_root(&repo)?;
    let (author_name, author_email) = author_from_config(&repo)?;

    let db_path = root.join(".doppelganger.db");
    let db_path_str = db_path
        .to_str()
        .ok_or_else(|| Error::Validation("db path is not valid UTF-8".to_string()))?;

    match cli.command {
        Commands::Issue {
            command: IssueCommands::Tui,
        } => doppelganger::tui::run_issue_tui(db_path_str).await,
        Commands::Branch {
            command: BranchCommands::Tui,
        } => doppelganger::tui::run_branch_tui(db_path_str, &repo).await,
        Commands::Issue { command } => {
            let db = Database::open(db_path_str).await?;
            let dispatch =
                commands::issue::handle(command, &db, &author_name, author_email.as_deref()).await;
            let checkpoint = db.checkpoint().await;
            dispatch?;
            checkpoint?;
            Ok(())
        }
        Commands::Branch { command } => {
            let db = Database::open(db_path_str).await?;
            let dispatch = commands::branch::handle(
                command,
                &db,
                &repo,
                &author_name,
                author_email.as_deref(),
            )
            .await;
            let checkpoint = db.checkpoint().await;
            dispatch?;
            checkpoint?;
            Ok(())
        }
    }
}
