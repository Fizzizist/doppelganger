use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "dg",
    version,
    about = "Local conversation layer for git repositories"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(name = "create")]
    Create {
        #[command(subcommand)]
        subcommand: CreateCommand,
    },
    #[command(name = "read")]
    Read {
        #[command(subcommand)]
        subcommand: ReadCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum CreateCommand {
    #[command(name = "issue")]
    Issue(CreateIssue),
    #[command(name = "branch")]
    Branch(CreateBranch),
    #[command(subcommand)]
    Comment(CreateComment),
}

#[derive(Parser, Debug)]
pub struct CreateIssue {
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub file: Option<PathBuf>,
}

#[derive(Parser, Debug)]
pub struct CreateBranch {
    pub issue_id: i64,
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub file: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum CreateComment {
    #[command(name = "issue")]
    Issue(CreateIssueComment),
    #[command(name = "branch")]
    Branch(CreateBranchComment),
}

#[derive(Parser, Debug)]
pub struct CreateIssueComment {
    pub issue_id: i64,
    #[arg(long)]
    pub content: Option<String>,
    #[arg(long)]
    pub file: Option<PathBuf>,
}

#[derive(Parser, Debug)]
pub struct CreateBranchComment {
    #[arg(long)]
    pub content: Option<String>,
    #[arg(long)]
    pub file: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum ReadCommand {
    #[command(name = "issue")]
    Issue {
        #[command(subcommand)]
        subcommand: ReadIssue,
    },
    #[command(name = "branch")]
    Branch {
        #[command(subcommand)]
        subcommand: ReadBranch,
    },
}

#[derive(Subcommand, Debug)]
pub enum ReadIssue {
    #[command(name = "thread")]
    Thread(ReadIssueThread),
    Single(ReadIssueSingle),
}

#[derive(Parser, Debug)]
pub struct ReadIssueSingle {
    pub issue_id: i64,
}

#[derive(Parser, Debug)]
pub struct ReadIssueThread {
    pub issue_id: i64,
}

#[derive(Subcommand, Debug)]
pub enum ReadBranch {
    #[command(name = "thread")]
    Thread(ReadBranchThread),
    Single(ReadBranchSingle),
}

#[derive(Parser, Debug)]
pub struct ReadBranchSingle;

#[derive(Parser, Debug)]
pub struct ReadBranchThread;
