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
    // Open a short-lived connection only to load the initial list; the event
    // loop opens fresh connections per poll so no lock is held between ticks.
    let issues = {
        let db = Database::open(db_path).await?;
        issue::list_issues(db.conn()).await?
    };
    let mut guard = terminal::TerminalGuard::new()?;
    let mut app = app::App::new_issue_list(issues);
    event::run(&mut guard.terminal, &mut app, db_path).await
}

pub async fn run_branch_tui(db_path: &str, branch_name: &str) -> Result<()> {
    let thread = {
        let db = Database::open(db_path).await?;
        let br = branch::get_by_name(db.conn(), branch_name).await?;
        let comments = comment::list_branch_comments(db.conn(), br.branch_id).await?;
        Thread::from(BranchWithComments {
            branch: br,
            comments,
        })
    };
    let mut guard = terminal::TerminalGuard::new()?;
    let mut app = app::App::new_thread(thread);
    event::run(&mut guard.terminal, &mut app, db_path).await
}
