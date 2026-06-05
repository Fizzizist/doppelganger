pub mod app;
pub mod event;
pub mod markdown;
pub mod model;
pub mod terminal;
pub mod view;

use crate::{
    db::{Database, branch, comment, issue, models::BranchWithComments},
    error::Result,
};
use model::Thread;

pub async fn run_issue_tui(db_path: &str) -> Result<()> {
    let db = Database::open(db_path).await?;
    let issues = issue::list_issues(db.conn()).await?;
    let mut guard = terminal::TerminalGuard::new()?;
    let mut app = app::App::new_issue_list(issues);
    event::run(&mut guard.terminal, &mut app, &db).await
}

pub async fn run_branch_tui(db_path: &str, branch_name: &str) -> Result<()> {
    let db = Database::open(db_path).await?;
    let br = branch::get_by_name(db.conn(), branch_name).await?;
    let comments = comment::list_branch_comments(db.conn(), br.branch_id).await?;
    let thread = Thread::from(BranchWithComments {
        branch: br,
        comments,
    });
    let mut guard = terminal::TerminalGuard::new()?;
    let mut app = app::App::new_thread(thread);
    event::run(&mut guard.terminal, &mut app, &db).await
}
