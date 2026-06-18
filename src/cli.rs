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

    pub fn tui_author_selection(&self) -> AuthorSelection {
        if let Some(ref id) = self.author {
            AuthorSelection::Named(id.clone())
        } else {
            AuthorSelection::Human
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
        #[arg(long)]
        hidden: bool,
    },
    Comment {
        issue_number: i64,
        content: Option<String>,
    },
    Sync {
        issue_number: i64,
        #[arg(long)]
        overwrite: Option<i64>,
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
    Read {
        #[arg(long)]
        hidden: bool,
    },
    Comment {
        content: Option<String>,
    },
    Tui,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse_cli(args: &[&str]) -> Cli {
        Cli::try_parse_from(args).expect("parse cli args")
    }

    #[test]
    fn tui_author_selection_defaults_to_human() {
        let cli = parse_cli(&["dg", "issue", "tui"]);
        match cli.tui_author_selection() {
            AuthorSelection::Human => {}
            other => panic!("expected Human, got {other:?}"),
        }
    }

    #[test]
    fn tui_author_selection_with_author_flag() {
        let cli = parse_cli(&["dg", "--author", "ci", "issue", "tui"]);
        match cli.tui_author_selection() {
            AuthorSelection::Named(id) => assert_eq!(id, "ci"),
            other => panic!("expected Named, got {other:?}"),
        }
    }

    #[test]
    fn cli_author_selection_defaults_to_robot() {
        let cli = parse_cli(&["dg", "issue", "create", "test"]);
        match cli.author_selection() {
            AuthorSelection::Robot => {}
            other => panic!("expected Robot, got {other:?}"),
        }
    }

    #[test]
    fn cli_author_selection_human_flag() {
        let cli = parse_cli(&["dg", "--human", "issue", "create", "test"]);
        match cli.author_selection() {
            AuthorSelection::Human => {}
            other => panic!("expected Human, got {other:?}"),
        }
    }
}
