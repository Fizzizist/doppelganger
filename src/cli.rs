use crate::config::AuthorSelection;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dg")]
pub struct Cli {
    /// Use the human author profile instead of the default robot profile
    #[arg(long, global = true, conflicts_with = "author")]
    pub human: bool,

    /// Use a named author profile
    #[arg(long, global = true, conflicts_with = "human")]
    pub author: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn author_selection(&self) -> AuthorSelection {
        if self.human {
            AuthorSelection::Human
        } else if let Some(ref id) = self.author {
            AuthorSelection::Named(id.clone())
        } else {
            AuthorSelection::Robot
        }
    }
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
