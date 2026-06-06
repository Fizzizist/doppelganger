use clap::Parser;
use doppelganger::{
    cli::{Cli, Commands},
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
        .ok_or_else(|| Error::Validation("db path is not valid UTF-8".to_string()))?
        .to_string();

    let db = Database::open(&db_path_str).await?;

    let dispatch = match cli.command {
        Commands::Issue { command } => {
            commands::issue::handle(
                command,
                &db,
                &db_path_str,
                &author_name,
                author_email.as_deref(),
            )
            .await
        }
        Commands::Branch { command } => {
            commands::branch::handle(
                command,
                &db,
                &db_path_str,
                &repo,
                &author_name,
                author_email.as_deref(),
            )
            .await
        }
    };

    let checkpoint = db.checkpoint().await;
    dispatch?;
    checkpoint?;
    Ok(())
}
