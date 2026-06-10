use clap::Parser;
use doppelganger::{
    cli::{BranchCommands, Cli, Commands, IssueCommands},
    commands,
    config::{self, LoadOutcome},
    db::Database,
    error::Error,
    git::{discover_repo, repo_root},
    logging,
};

enum RunOutcome {
    Completed,
    FirstRun(std::path::PathBuf),
}

#[tokio::main]
async fn main() {
    logging::init();

    match run().await {
        Ok(RunOutcome::FirstRun(path)) => {
            tracing::info!("sample config written to {}", path.display());
            eprintln!(
                "No config found. A sample config has been written to: {}\nEdit it and re-run.",
                path.display()
            );
            std::process::exit(0);
        }
        Ok(RunOutcome::Completed) => {}
        Err(e) => {
            tracing::error!("{e}");
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}

async fn run() -> doppelganger::error::Result<RunOutcome> {
    let cli = Cli::parse();

    match config::load_or_init()? {
        LoadOutcome::Created(path) => Ok(RunOutcome::FirstRun(path)),
        LoadOutcome::Loaded(config) => {
            let selection = cli.author_selection();
            let (author_name, author_email) = config.resolve(selection)?;

            let tui_selection = cli.tui_author_selection();
            let (tui_author_name, tui_author_email) = config.resolve(tui_selection)?;

            let repo = discover_repo()?;
            let root = repo_root(&repo)?;

            let db_path = root.join(".doppelganger.db");
            let db_path_str = db_path
                .to_str()
                .ok_or_else(|| Error::Validation("db path is not valid UTF-8".to_string()))?;

            match cli.command {
                Commands::Issue { command } => match command {
                    IssueCommands::Tui => {
                        doppelganger::tui::run_issue_tui(
                            db_path_str,
                            &tui_author_name,
                            tui_author_email.as_deref(),
                        )
                        .await?;
                        Ok(RunOutcome::Completed)
                    }
                    other_command => {
                        let db = Database::open(db_path_str).await?;
                        let dispatch = commands::issue::handle(
                            other_command,
                            &db,
                            &author_name,
                            author_email.as_deref(),
                        )
                        .await;
                        let checkpoint = db.checkpoint().await;
                        dispatch?;
                        checkpoint?;
                        Ok(RunOutcome::Completed)
                    }
                },
                Commands::Branch { command } => match command {
                    BranchCommands::Tui => {
                        doppelganger::tui::run_branch_tui(
                            db_path_str,
                            &repo,
                            &tui_author_name,
                            tui_author_email.as_deref(),
                        )
                        .await?;
                        Ok(RunOutcome::Completed)
                    }
                    other_command => {
                        let db = Database::open(db_path_str).await?;
                        let dispatch = commands::branch::handle(
                            other_command,
                            &db,
                            &repo,
                            &author_name,
                            author_email.as_deref(),
                        )
                        .await;
                        let checkpoint = db.checkpoint().await;
                        dispatch?;
                        checkpoint?;
                        Ok(RunOutcome::Completed)
                    }
                },
            }
        }
    }
}
