use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dg")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Issue {
        #[command(subcommand)]
        command: IssueCommands,
    },
    Branch {
        #[command(subcommand)]
        command: BranchCommands,
    },
}

#[derive(Subcommand)]
pub enum IssueCommands {
    Create {
        content: Option<String>,
        #[arg(long)]
        name: Option<String>,
    },
    Read {
        issue_number: i64,
    },
    Comment {
        issue_number: i64,
        content: Option<String>,
    },
    Tui,
}

#[derive(Subcommand)]
pub enum BranchCommands {
    Create {
        issue_number: i64,
        description: Option<String>,
        #[arg(long)]
        overwrite: bool,
    },
    Read,
    Comment {
        content: Option<String>,
    },
    Tui,
}
