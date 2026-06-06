use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Stdout;

use crossterm::event::{Event, EventStream, KeyEventKind};
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;

use crate::db::models::{BranchWithComments, IssueWithComments};
use crate::db::{Database, comment, issue};
use crate::error::Result;
use crate::log;
use crate::tui::app::{Action, App, Screen};
use crate::tui::model::Thread;
use crate::tui::view;

pub async fn run(
    terminal: &mut ratatui::Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    db_path: &str,
) -> Result<()> {
    render(terminal, app)?;

    let mut prev_fingerprint = compute_fingerprint(db_path, app).await.ok();

    let mut event_stream = EventStream::new();
    let mut poll_interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

    loop {
        if app.should_quit {
            return Ok(());
        }

        tokio::select! {
            maybe_event = event_stream.next() => {
                if let Some(Ok(Event::Key(key))) = maybe_event
                    && key.kind == KeyEventKind::Press
                {
                    app.handle_key(key);
                    if let Some(action) = app.take_action() {
                        handle_action(db_path, app, action).await;
                    }
                    render(terminal, app)?;
                }
            }
            _ = poll_interval.tick() => {
                let curr_fingerprint = compute_fingerprint(db_path, app).await.ok();
                if curr_fingerprint != prev_fingerprint {
                    if let Err(e) = reload_data(db_path, app).await {
                        log!("TUI reload error: {}", e);
                    } else {
                        render(terminal, app)?;
                    }
                    prev_fingerprint = curr_fingerprint;
                }
            }
        }
    }
}

fn render(terminal: &mut ratatui::Terminal<CrosstermBackend<Stdout>>, app: &App) -> Result<()> {
    terminal.draw(|frame| view::render(frame, app))?;
    Ok(())
}

async fn handle_action(db_path: &str, app: &mut App, action: Action) {
    match action {
        Action::OpenIssueThread(issue_id) => match load_issue_thread(db_path, issue_id).await {
            Ok(thread) => {
                app.has_issue_list = true;
                app.transition_to_thread(thread);
            }
            Err(e) => {
                log!("Failed to load issue thread {}: {}", issue_id, e);
            }
        },
        Action::BackToList => {
            app.transition_to_list();
        }
        Action::Quit => {
            app.should_quit = true;
        }
    }
}

pub async fn compute_fingerprint(db_path: &str, app: &App) -> Result<u64> {
    // Open a short-lived connection per poll so the TUI never holds the WAL
    // lock between ticks, allowing other processes to write concurrently.
    let db = Database::open(db_path).await?;
    let conn = db.conn();
    match &app.screen {
        Screen::IssueList => issue_list_fingerprint(conn).await,
        Screen::Thread(thread) => {
            if let Some(issue_id) = thread.issue_id {
                thread_fingerprint(conn, Some(issue_id), None).await
            } else if let Some(branch_id) = thread.branch_id {
                thread_fingerprint(conn, None, Some(branch_id)).await
            } else {
                Ok(0)
            }
        }
    }
}

pub async fn issue_list_fingerprint(conn: &turso::Connection) -> Result<u64> {
    let mut rows = conn
        .query("SELECT COUNT(issue_id), MAX(updated_at) FROM issue", ())
        .await?;

    match rows.next().await? {
        Some(row) => {
            let count: i64 = row.get(0).unwrap_or(0);
            let max_updated: Option<String> = row.get(1).ok();
            let mut hasher = DefaultHasher::new();
            (count, max_updated).hash(&mut hasher);
            Ok(hasher.finish())
        }
        None => Ok(0),
    }
}

pub async fn thread_fingerprint(
    conn: &turso::Connection,
    issue_id: Option<i64>,
    branch_id: Option<i64>,
) -> Result<u64> {
    let (comment_count, max_updated_at) = if let Some(id) = issue_id {
        let mut rows = conn
            .query(
                "SELECT COUNT(issue_comment_id), MAX(updated_at) FROM issue_comment WHERE issue_id = ?1",
                turso::params::Params::Positional(vec![turso::Value::Integer(id)]),
            )
            .await?;
        match rows.next().await? {
            Some(row) => {
                let count: i64 = row.get(0).unwrap_or(0);
                let max_updated: Option<String> = row.get(1).ok();
                (count, max_updated)
            }
            None => return Ok(0),
        }
    } else if let Some(id) = branch_id {
        let mut rows = conn
            .query(
                "SELECT COUNT(branch_comment_id), MAX(updated_at) FROM branch_comment WHERE branch_id = ?1",
                turso::params::Params::Positional(vec![turso::Value::Integer(id)]),
            )
            .await?;
        match rows.next().await? {
            Some(row) => {
                let count: i64 = row.get(0).unwrap_or(0);
                let max_updated: Option<String> = row.get(1).ok();
                (count, max_updated)
            }
            None => return Ok(0),
        }
    } else {
        return Ok(0);
    };

    let parent_updated = if let Some(id) = issue_id {
        let mut rows = conn
            .query(
                "SELECT updated_at FROM issue WHERE issue_id = ?1",
                turso::params::Params::Positional(vec![turso::Value::Integer(id)]),
            )
            .await?;
        let row = rows.next().await?;
        row.and_then(|r| r.get::<String>(0).ok())
    } else if let Some(id) = branch_id {
        let mut rows = conn
            .query(
                "SELECT updated_at FROM branch WHERE branch_id = ?1",
                turso::params::Params::Positional(vec![turso::Value::Integer(id)]),
            )
            .await?;
        let row = rows.next().await?;
        row.and_then(|r| r.get::<String>(0).ok())
    } else {
        None
    };

    let mut hasher = DefaultHasher::new();
    (comment_count, max_updated_at, parent_updated).hash(&mut hasher);
    Ok(hasher.finish())
}

async fn load_issue_thread(db_path: &str, issue_id: i64) -> Result<Thread> {
    let db = Database::open(db_path).await?;
    let conn = db.conn();
    let issue = issue::get_by_id(conn, issue_id).await?;
    let comments = comment::list_issue_comments(conn, issue_id).await?;
    Ok(Thread::from(IssueWithComments { issue, comments }))
}

async fn load_branch_thread(db_path: &str, branch_id: i64) -> Result<Thread> {
    let db = Database::open(db_path).await?;
    let conn = db.conn();
    let branch = crate::db::branch::get_by_id(conn, branch_id).await?;
    let comments = comment::list_branch_comments(conn, branch_id).await?;
    Ok(Thread::from(BranchWithComments { branch, comments }))
}

async fn reload_data(db_path: &str, app: &mut App) -> Result<()> {
    match &app.screen {
        Screen::IssueList => {
            let db = Database::open(db_path).await?;
            let issues = issue::list_issues(db.conn()).await?;
            app.set_issues(issues);
        }
        Screen::Thread(thread) => {
            let issue_id = thread.issue_id;
            let branch_id = thread.branch_id;
            if let Some(issue_id) = issue_id {
                let new_thread = load_issue_thread(db_path, issue_id).await?;
                app.replace_thread(new_thread);
            } else if let Some(branch_id) = branch_id {
                let new_thread = load_branch_thread(db_path, branch_id).await?;
                app.replace_thread(new_thread);
            }
        }
    }
    Ok(())
}
