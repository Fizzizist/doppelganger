use crossterm::event::{Event as CrosstermEvent, EventStream, KeyCode, KeyModifiers};
use futures_util::StreamExt;

use crate::db::{Database, comment, issue};
use crate::tui::app::{App, Screen};
use crate::tui::model::Thread;

const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

pub enum AppEvent {
    Key(KeyCode, KeyModifiers),
    Tick,
}

pub async fn next_event(events: &mut EventStream) -> crate::error::Result<AppEvent> {
    loop {
        let event: Option<()> = tokio::select! {
            maybe_event = events.next() => {
                match maybe_event {
                    Some(Ok(CrosstermEvent::Key(key_event))) => {
                        return Ok(AppEvent::Key(key_event.code, key_event.modifiers));
                    }
                    Some(Ok(CrosstermEvent::Resize(_, _))) => return Ok(AppEvent::Tick),
                    Some(Ok(_)) => None::<()>,
                    Some(Err(e)) => return Err(crate::error::Error::Tui(e.to_string())),
                    None => return Err(crate::error::Error::Tui("event stream ended".to_string())),
                }
            }
            _ = tokio::time::sleep(POLL_INTERVAL) => {
                return Ok(AppEvent::Tick);
            }
        };

        if event.is_none() {
            continue;
        }
    }
}

pub fn handle_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> bool {
    match app.screen {
        Screen::IssueList => handle_issue_list_key(app, code, modifiers),
        Screen::Thread => handle_thread_key(app, code, modifiers),
    }
}

fn handle_issue_list_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> bool {
    match (code, modifiers) {
        (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => true,
        (KeyCode::Char('j'), _) | (KeyCode::Down, _) => {
            if !app.issues.is_empty() {
                app.selected_issue = (app.selected_issue + 1).min(app.issues.len() - 1);
            }
            false
        }
        (KeyCode::Char('k'), _) | (KeyCode::Up, _) => {
            if !app.issues.is_empty() {
                app.selected_issue = app.selected_issue.saturating_sub(1);
            }
            false
        }
        (KeyCode::Enter, _) | (KeyCode::Char('l'), _) => {
            app.select_issue();
            false
        }
        _ => false,
    }
}

fn handle_thread_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> bool {
    match (code, modifiers) {
        (KeyCode::Char('q'), _) | (KeyCode::Esc, _) | (KeyCode::Char('h'), _) => {
            app.back();
            false
        }
        (KeyCode::Char('j'), _) | (KeyCode::Down, _) => {
            app.thread_scroll = app.thread_scroll.saturating_add(1);
            false
        }
        (KeyCode::Char('k'), _) | (KeyCode::Up, _) => {
            app.thread_scroll = app.thread_scroll.saturating_sub(1);
            false
        }
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
            app.thread_scroll = app.thread_scroll.saturating_sub(20);
            false
        }
        (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
            app.thread_scroll = app.thread_scroll.saturating_add(20);
            false
        }
        _ => false,
    }
}

pub async fn load_issues(db_path: &str, app: &mut App) -> crate::error::Result<bool> {
    let db = Database::open(db_path).await?;
    let conn = db.conn();
    let issues = issue::list(conn).await?;

    let fingerprint: String = issues
        .iter()
        .map(|i| format!("{}:{}", i.issue_id, i.updated_at))
        .collect::<Vec<_>>()
        .join(",");

    let changed = fingerprint != app.last_fingerprint;
    if changed {
        app.issues = issues;
        app.last_fingerprint = fingerprint;
    }
    Ok(changed)
}

pub async fn load_issue_thread(db_path: &str, app: &mut App) -> crate::error::Result<bool> {
    let issue_idx = app.selected_issue;
    if issue_idx >= app.issues.len() {
        return Ok(false);
    }

    let issue = &app.issues[issue_idx];
    let issue_id = issue.issue_id;

    let db = Database::open(db_path).await?;
    let conn = db.conn();
    let comments = comment::list_issue_comments(conn, issue_id).await?;

    let thread = Thread::from(&crate::db::models::IssueWithComments {
        issue: issue.clone(),
        comments,
    });

    let changed = match &app.thread {
        Some(t) => t.updated_at != thread.updated_at || t.comments.len() != thread.comments.len(),
        None => true,
    };

    app.thread = Some(thread);
    Ok(changed)
}

pub async fn load_branch_thread(
    db_path: &str,
    branch_name: &str,
    app: &mut App,
) -> crate::error::Result<bool> {
    let db = Database::open(db_path).await?;
    let conn = db.conn();
    let br = crate::db::branch::get_by_name(conn, branch_name).await?;
    let comments = crate::db::comment::list_branch_comments(conn, br.branch_id).await?;
    let thread = Thread::from(&crate::db::models::BranchWithComments {
        branch: br,
        comments,
    });

    let changed = match &app.thread {
        Some(t) => t.updated_at != thread.updated_at || t.comments.len() != thread.comments.len(),
        None => true,
    };

    app.thread = Some(thread);
    Ok(changed)
}
