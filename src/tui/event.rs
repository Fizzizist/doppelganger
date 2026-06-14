use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hjkl_editor_tui::crossterm_key_event_to_input;
use hjkl_engine::{VimMode, decode_planned_input};

use crate::db::{Database, comment, issue};
use crate::tui::app::{App, Focus, ModalState, Screen};
use crate::tui::model::Thread;

pub enum KeyResult {
    Continue,
    Quit,
    SubmitComment,
}

pub fn handle_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> KeyResult {
    match app.screen {
        Screen::IssueList => handle_issue_list_key(app, code, modifiers),
        Screen::Thread => handle_thread_key(app, code, modifiers),
    }
}

pub fn handle_modal_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> bool {
    match &app.modal {
        Some(ModalState::NameInput) => match (code, modifiers) {
            (KeyCode::Esc, _) => {
                app.cancel_modal();
                false
            }
            (KeyCode::Backspace, _) => {
                app.input_buffer.pop();
                false
            }
            (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                app.input_buffer.push(c);
                false
            }
            _ => false,
        },
        Some(ModalState::Error(_)) => match code {
            KeyCode::Esc | KeyCode::Enter => {
                app.cancel_modal();
                false
            }
            _ => false,
        },
        None => false,
    }
}

fn handle_issue_list_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> KeyResult {
    match (code, modifiers) {
        (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => KeyResult::Quit,
        (KeyCode::Char('j'), _) | (KeyCode::Down, _) => {
            if !app.issues.is_empty() {
                app.selected_issue = (app.selected_issue + 1).min(app.issues.len() - 1);
            }
            KeyResult::Continue
        }
        (KeyCode::Char('k'), _) | (KeyCode::Up, _) => {
            if !app.issues.is_empty() {
                app.selected_issue = app.selected_issue.saturating_sub(1);
            }
            KeyResult::Continue
        }
        (KeyCode::Enter, _) | (KeyCode::Char('l'), _) => {
            app.select_issue();
            KeyResult::Continue
        }
        (KeyCode::Char('n'), _) => {
            app.start_name_input();
            KeyResult::Continue
        }
        _ => KeyResult::Continue,
    }
}

fn handle_thread_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> KeyResult {
    if app.ctrl_w_pending {
        app.ctrl_w_pending = false;
        return match (code, modifiers) {
            (KeyCode::Char('j'), _) => {
                app.focus_input_box();
                KeyResult::Continue
            }
            (KeyCode::Char('k'), _) => {
                app.focus_thread();
                KeyResult::Continue
            }
            _ => KeyResult::Continue,
        };
    }

    if code == KeyCode::Char('w') && modifiers.contains(KeyModifiers::CONTROL) {
        app.ctrl_w_pending = true;
        return KeyResult::Continue;
    }

    match app.focus {
        Focus::Thread => handle_thread_focus_key(app, code, modifiers),
        Focus::InputBox => handle_input_box_key(app, code, modifiers),
    }
}

fn handle_thread_focus_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> KeyResult {
    match (code, modifiers) {
        (KeyCode::Char('q'), _) | (KeyCode::Esc, _) | (KeyCode::Char('h'), _) => {
            app.back();
            KeyResult::Continue
        }
        _ => {
            handle_thread_scroll(app, code, modifiers);
            KeyResult::Continue
        }
    }
}

fn handle_input_box_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> KeyResult {
    let Some(editor) = app.input_editor.as_mut() else {
        return KeyResult::Continue;
    };

    if code == KeyCode::Esc {
        if matches!(
            editor.vim_mode(),
            VimMode::Insert | VimMode::Visual | VimMode::VisualLine | VimMode::VisualBlock
        ) {
            let planned = crossterm_key_event_to_input(KeyEvent::new(code, modifiers));
            if let Some(input) = decode_planned_input(planned) {
                editor.handle_input(input);
            }
            return KeyResult::Continue;
        }
        app.focus_thread();
        return KeyResult::Continue;
    }

    if code == KeyCode::Enter && matches!(editor.vim_mode(), VimMode::Normal) {
        if !editor.text().trim().is_empty() {
            return KeyResult::SubmitComment;
        }
        return KeyResult::Continue;
    }

    let planned = crossterm_key_event_to_input(KeyEvent::new(code, modifiers));
    if let Some(input) = decode_planned_input(planned) {
        editor.handle_input(input);
    }
    KeyResult::Continue
}

pub fn handle_thread_scroll(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    match (code, modifiers) {
        (KeyCode::Char('j'), _) | (KeyCode::Down, _) => {
            app.thread_scroll = app.thread_scroll.saturating_add(1);
        }
        (KeyCode::Char('k'), _) | (KeyCode::Up, _) => {
            app.thread_scroll = app.thread_scroll.saturating_sub(1);
        }
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
            app.thread_scroll = app.thread_scroll.saturating_sub(20);
        }
        (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
            app.thread_scroll = app.thread_scroll.saturating_add(20);
        }
        _ => {}
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
