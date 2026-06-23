use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hjkl_editor_tui::crossterm_key_event_to_input;
use hjkl_engine::{VimMode, decode_planned_input};

use crate::db::{Database, comment, issue};
use crate::tui::app::{App, EditTarget, Focus, ModalState, Screen};
use crate::tui::model::Thread;

pub enum KeyResult {
    Continue,
    Quit,
    SubmitComment,
    EditEntity,
    ToggleArchive,
    ToggleShowArchived,
    ToggleHidden,
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
        (KeyCode::Char('a'), _) => KeyResult::ToggleArchive,
        (KeyCode::Char('A'), _) => KeyResult::ToggleShowArchived,
        _ => KeyResult::Continue,
    }
}

fn handle_thread_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> KeyResult {
    if app.ctrl_w_pending {
        app.ctrl_w_pending = false;
        return match (code, modifiers) {
            (KeyCode::Char('j'), _) => {
                if !app.thread.as_ref().map(|t| t.archived).unwrap_or(false) {
                    app.focus_input_box();
                }
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
        (KeyCode::Char('H'), _) => {
            if app.thread_selected == 0 {
                return KeyResult::Continue;
            }
            KeyResult::ToggleHidden
        }
        (KeyCode::Char('q'), _) | (KeyCode::Esc, _) | (KeyCode::Char('h'), _) => {
            app.back();
            KeyResult::Continue
        }
        (KeyCode::Char('j'), _) | (KeyCode::Down, _) => {
            let max = app.thread.as_ref().map(|t| t.comments.len()).unwrap_or(0);
            app.thread_selected = (app.thread_selected + 1).min(max);
            if let Some(&start) = app.thread_item_offsets.get(app.thread_selected) {
                app.thread_scroll = start as u16;
            }
            KeyResult::Continue
        }
        (KeyCode::Char('k'), _) | (KeyCode::Up, _) => {
            app.thread_selected = app.thread_selected.saturating_sub(1);
            if let Some(&start) = app.thread_item_offsets.get(app.thread_selected) {
                app.thread_scroll = start as u16;
            }
            KeyResult::Continue
        }
        (KeyCode::Char('e'), _) => {
            if app.thread.as_ref().map(|t| t.archived).unwrap_or(false) {
                return KeyResult::Continue;
            }
            let Some(thread) = &app.thread else {
                return KeyResult::Continue;
            };
            let target = if app.thread_selected == 0 {
                EditTarget::Description
            } else {
                let idx = app.thread_selected - 1;
                if idx < thread.comments.len() {
                    if thread.comments[idx].hidden {
                        return KeyResult::Continue;
                    }
                    EditTarget::Comment(thread.comments[idx].comment_id)
                } else {
                    return KeyResult::Continue;
                }
            };
            app.pending_edit = Some(target);
            KeyResult::EditEntity
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
    let issues = if app.show_archived {
        issue::list_all(conn).await?
    } else {
        issue::list(conn).await?
    };

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

    let issue_id = app.issues[issue_idx].issue_id;

    let db = Database::open(db_path).await?;
    let conn = db.conn();
    let issue = issue::get_by_id(conn, issue_id).await?;
    let comments = comment::list_issue_comments(conn, issue_id, true).await?;

    let thread = Thread::from(&crate::db::models::IssueWithComments { issue, comments });

    let changed = match &app.thread {
        Some(t) => {
            t.updated_at != thread.updated_at
                || t.comments.len() != thread.comments.len()
                || t.comments
                    .iter()
                    .zip(thread.comments.iter())
                    .any(|(a, b)| a.hidden != b.hidden)
        }
        None => true,
    };

    let archived = thread.archived;
    app.thread = Some(thread);
    if archived && matches!(app.focus, Focus::InputBox) {
        app.focus_thread();
    }
    Ok(changed)
}

pub async fn load_branch_thread(
    db_path: &str,
    branch_name: &str,
    app: &mut App,
) -> crate::error::Result<bool> {
    let db = Database::open(db_path).await?;
    let conn = db.conn();
    let br = crate::db::branch::get_active_by_name(conn, branch_name).await?;
    let comments = crate::db::comment::list_branch_comments(conn, br.branch_id, true).await?;
    let thread = Thread::from(&crate::db::models::BranchWithComments {
        branch: br,
        comments,
    });

    let changed = match &app.thread {
        Some(t) => {
            t.updated_at != thread.updated_at
                || t.comments.len() != thread.comments.len()
                || t.comments
                    .iter()
                    .zip(thread.comments.iter())
                    .any(|(a, b)| a.hidden != b.hidden)
        }
        None => true,
    };

    app.thread = Some(thread);
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{Database, author, issue as issue_db};

    #[tokio::test]
    async fn load_issue_thread_reflects_updated_description() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let db_path = tmp
            .path()
            .join("test.db")
            .to_str()
            .expect("path")
            .to_string();

        let db = Database::open(&db_path).await.expect("open db");
        let conn = db.conn();
        let author = author::find_or_create(conn, "Alice", None)
            .await
            .expect("author");
        let created = issue_db::create(
            conn,
            Some("My Issue"),
            "original description",
            author.author_id,
            None,
        )
        .await
        .expect("create issue");
        db.checkpoint().await.expect("checkpoint");
        drop(db);

        let mut app = App::new("Alice".to_string(), None);
        app.issues = vec![created.clone()];
        load_issue_thread(&db_path, &mut app)
            .await
            .expect("load thread");
        assert_eq!(
            app.thread.as_ref().expect("thread").description,
            "original description"
        );

        let db = Database::open(&db_path).await.expect("open db");
        issue_db::update_description(db.conn(), created.issue_id, "revised description")
            .await
            .expect("update description");
        db.checkpoint().await.expect("checkpoint");
        drop(db);

        load_issue_thread(&db_path, &mut app)
            .await
            .expect("reload thread");
        assert_eq!(
            app.thread.as_ref().expect("thread").description,
            "revised description",
            "thread view must reflect the updated description without a full issues reload"
        );
    }

    #[test]
    fn key_a_on_issue_list_returns_toggle_archive() {
        let mut app = App::default();
        assert!(matches!(
            handle_key(&mut app, KeyCode::Char('a'), KeyModifiers::NONE),
            KeyResult::ToggleArchive
        ));
    }

    #[test]
    fn key_shift_a_on_issue_list_returns_toggle_show_archived() {
        let mut app = App::default();
        assert!(matches!(
            handle_key(&mut app, KeyCode::Char('A'), KeyModifiers::NONE),
            KeyResult::ToggleShowArchived
        ));
    }

    #[test]
    fn key_e_no_op_when_thread_archived() {
        let mut app = App::default();
        app.screen = Screen::Thread;
        app.thread = Some(Thread {
            issue_id: 1,
            title: "Test".to_string(),
            author: "Alice".to_string(),
            created_at: "2025-01-01".to_string(),
            updated_at: "2025-01-01".to_string(),
            description: "desc".to_string(),
            comments: vec![],
            archived: true,
        });
        assert!(matches!(
            handle_key(&mut app, KeyCode::Char('e'), KeyModifiers::NONE),
            KeyResult::Continue
        ));
        assert!(app.pending_edit.is_none());
    }

    #[test]
    fn ctrl_w_j_blocked_when_thread_archived() {
        let mut app = App::new("test".to_string(), None);
        app.screen = Screen::Thread;
        app.thread = Some(Thread {
            issue_id: 1,
            title: "Test".to_string(),
            author: "Alice".to_string(),
            created_at: "2025-01-01".to_string(),
            updated_at: "2025-01-01".to_string(),
            description: "desc".to_string(),
            comments: vec![],
            archived: true,
        });
        handle_key(&mut app, KeyCode::Char('w'), KeyModifiers::CONTROL);
        assert!(app.ctrl_w_pending);
        handle_key(&mut app, KeyCode::Char('j'), KeyModifiers::NONE);
        assert!(!app.ctrl_w_pending);
        assert!(matches!(app.focus, Focus::Thread));
        assert!(app.input_editor.is_none());
    }

    #[test]
    fn shift_h_on_description_returns_continue() {
        let mut app = App::default();
        app.screen = Screen::Thread;
        app.thread = Some(Thread {
            issue_id: 1,
            title: "T".to_string(),
            author: "Alice".to_string(),
            created_at: "2025-01-01".to_string(),
            updated_at: "2025-01-01".to_string(),
            description: "desc".to_string(),
            comments: vec![],
            archived: false,
        });
        app.thread_selected = 0; // description
        let result = handle_key(&mut app, KeyCode::Char('H'), KeyModifiers::SHIFT);
        assert!(matches!(result, KeyResult::Continue));
    }

    #[test]
    fn shift_h_on_comment_returns_toggle_hidden() {
        use crate::tui::model::ThreadComment;
        let mut app = App::default();
        app.screen = Screen::Thread;
        app.thread = Some(Thread {
            issue_id: 1,
            title: "T".to_string(),
            author: "Alice".to_string(),
            created_at: "2025-01-01".to_string(),
            updated_at: "2025-01-01".to_string(),
            description: "desc".to_string(),
            comments: vec![ThreadComment {
                comment_id: 1,
                author: "Alice".to_string(),
                created_at: "2025-01-01".to_string(),
                content: "comment".to_string(),
                hidden: false,
            }],
            archived: false,
        });
        app.thread_selected = 1; // first comment
        let result = handle_key(&mut app, KeyCode::Char('H'), KeyModifiers::SHIFT);
        assert!(matches!(result, KeyResult::ToggleHidden));
    }

    #[test]
    fn key_e_no_op_on_hidden_comment() {
        use crate::tui::model::ThreadComment;
        let mut app = App::default();
        app.screen = Screen::Thread;
        app.thread = Some(Thread {
            issue_id: 1,
            title: "T".to_string(),
            author: "Alice".to_string(),
            created_at: "2025-01-01".to_string(),
            updated_at: "2025-01-01".to_string(),
            description: "desc".to_string(),
            comments: vec![ThreadComment {
                comment_id: 1,
                author: "Alice".to_string(),
                created_at: "2025-01-01".to_string(),
                content: "content".to_string(),
                hidden: true,
            }],
            archived: false,
        });
        app.thread_selected = 1;
        let result = handle_key(&mut app, KeyCode::Char('e'), KeyModifiers::NONE);
        assert!(matches!(result, KeyResult::Continue));
        assert!(app.pending_edit.is_none());
    }
}
