pub mod app;
pub mod editor;
pub mod event;
pub mod highlight;
pub mod input_box;
pub mod model;
pub mod terminal;
pub mod view;

pub use app::{App, EditTarget, Focus, ModalState, TuiMode};
pub use model::{Thread, ThreadComment};

use crossterm::event::{Event as CrosstermEvent, EventStream, KeyCode, KeyModifiers};
use futures_util::StreamExt;
use tokio::time::{MissedTickBehavior, interval_at};

use crate::db;
use crate::error::{Error, Result};
use crate::tui::app::Screen;
use crate::tui::event::KeyResult;

const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

pub async fn run_issue_tui(
    db_path: &str,
    author_name: &str,
    author_email: Option<&str>,
) -> Result<()> {
    crate::logging::init();
    let mut app = App::new(author_name.to_string(), author_email.map(|s| s.to_string()));
    if let Err(e) = event::load_issues(db_path, &mut app).await {
        tracing::warn!("initial issue load failed: {e}");
    }
    let mut guard = terminal::TuiGuard::init()?;
    run_loop(db_path, &mut guard, &mut app).await
}

pub async fn run_branch_tui(
    db_path: &str,
    repo: &git2::Repository,
    author_name: &str,
    author_email: Option<&str>,
) -> Result<()> {
    crate::logging::init();
    let branch_name = crate::git::current_branch(repo)?;
    let mut app = App::new(author_name.to_string(), author_email.map(|s| s.to_string()));
    event::load_branch_thread(db_path, &branch_name, &mut app).await?;
    app.screen = Screen::Thread;
    app.tui_mode = TuiMode::Branch {
        branch_name: branch_name.clone(),
    };
    let mut guard = terminal::TuiGuard::init()?;
    run_loop(db_path, &mut guard, &mut app).await
}

async fn run_loop(db_path: &str, guard: &mut terminal::TuiGuard, app: &mut App) -> Result<()> {
    let mut events = EventStream::new();
    let mut tick = interval_at(tokio::time::Instant::now() + POLL_INTERVAL, POLL_INTERVAL);
    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);

    draw(guard, app)?;

    loop {
        tokio::select! {
            _ = tick.tick() => {
                if poll_db(db_path, app).await {
                    draw(guard, app)?;
                }
            }
            maybe_event = events.next() => {
                match maybe_event {
                    Some(Ok(CrosstermEvent::Key(key))) => {
                        if key.kind == crossterm::event::KeyEventKind::Press {
                            match handle_key_event(db_path, guard, app, key.code, key.modifiers).await? {
                                KeyResult::Quit => break,
                                KeyResult::SubmitComment => {
                                    if let Err(e) = submit_comment(db_path, app).await {
                                        app.show_error(e.to_string());
                                    }
                                }
                                KeyResult::EditEntity => {
                                    if let Err(e) = handle_edit(db_path, guard, app).await {
                                        app.show_error(e.to_string());
                                    }
                                }
                                KeyResult::Continue => {}
                            }
                        }
                        if matches!(app.screen, Screen::Thread) && app.thread.is_none()
                            && let Err(e) = event::load_issue_thread(db_path, app).await
                        {
                            tracing::warn!("failed to load thread: {e}");
                        }
                        draw(guard, app)?;
                    }
                    Some(Ok(CrosstermEvent::Resize(_, _))) => draw(guard, app)?,
                    Some(Ok(_)) => {}
                    Some(Err(e)) => return Err(Error::Tui(e.to_string())),
                    None => return Err(Error::Tui("event stream ended".to_string())),
                }
            }
        }
    }

    Ok(())
}

async fn handle_key_event(
    db_path: &str,
    guard: &mut terminal::TuiGuard,
    app: &mut App,
    code: KeyCode,
    modifiers: KeyModifiers,
) -> Result<KeyResult> {
    if app.modal.is_some() {
        if matches!(&app.modal, Some(ModalState::NameInput)) && code == KeyCode::Enter {
            let name = app.confirm_name_input();
            let name_opt = if name.is_empty() {
                None
            } else {
                Some(name.as_str())
            };

            guard.suspend()?;

            let editor = match crate::config::load_or_init()? {
                crate::config::LoadOutcome::Loaded(c) => c.editor,
                crate::config::LoadOutcome::Created(_, c) => c.editor,
            };

            let content_result = editor::spawn_editor(&editor, "");

            guard.resume()?;

            match content_result {
                Ok(Some(description)) => {
                    let db = crate::db::Database::open(db_path).await?;
                    let conn = db.conn();
                    let author = crate::db::author::find_or_create(
                        conn,
                        &app.author_name,
                        app.author_email.as_deref(),
                    )
                    .await?;
                    let created = crate::db::issue::create(
                        conn,
                        name_opt,
                        &description,
                        author.author_id,
                        None,
                    )
                    .await?;
                    db.checkpoint().await?;
                    drop(db);

                    event::load_issues(db_path, app).await?;
                    let idx = app
                        .issues
                        .iter()
                        .position(|i| i.issue_id == created.issue_id)
                        .expect("newly created issue must be in refreshed list");
                    app.selected_issue = idx;
                    app.select_issue();
                    event::load_issue_thread(db_path, app).await?;
                }
                Ok(None) => {
                    app.show_error("content is empty; issue not created".to_string());
                }
                Err(e) => {
                    app.show_error(e.to_string());
                }
            }

            return Ok(KeyResult::Continue);
        }

        return Ok(KeyResult::from(event::handle_modal_key(
            app, code, modifiers,
        )));
    }

    let was_thread = matches!(app.screen, Screen::Thread);
    let result = event::handle_key(app, code, modifiers);

    if matches!(app.tui_mode, TuiMode::Branch { .. })
        && was_thread
        && matches!(app.screen, Screen::IssueList)
    {
        return Ok(KeyResult::Quit);
    }

    Ok(result)
}

async fn submit_comment(db_path: &str, app: &mut App) -> Result<()> {
    let content = match app.input_editor.as_mut() {
        Some(editor) => {
            let text = editor.text();
            if text.trim().is_empty() {
                return Ok(());
            }
            text
        }
        None => return Ok(()),
    };

    let db = crate::db::Database::open(db_path).await?;
    let conn = db.conn();
    let author =
        crate::db::author::find_or_create(conn, &app.author_name, app.author_email.as_deref())
            .await?;

    let is_branch = matches!(app.tui_mode, TuiMode::Branch { .. });
    let branch_name = match &app.tui_mode {
        TuiMode::Branch { branch_name } => Some(branch_name.clone()),
        TuiMode::Issue => None,
    };

    match &app.tui_mode {
        TuiMode::Issue => {
            let issue_idx = app.selected_issue;
            if issue_idx >= app.issues.len() {
                return Ok(());
            }
            let issue_id = app.issues[issue_idx].issue_id;
            db::comment::create_issue_comment(conn, issue_id, &content, author.author_id).await?;
        }
        TuiMode::Branch { branch_name } => {
            let branch = db::branch::get_by_name(conn, branch_name).await?;
            db::comment::create_branch_comment(conn, branch.branch_id, &content, author.author_id)
                .await?;
        }
    }

    db.checkpoint().await?;

    if let Some(editor) = app.input_editor.as_mut() {
        editor.set_text("");
        editor.enter_normal();
    }
    app.focus_thread();

    if is_branch {
        if let Some(branch_name) = branch_name {
            event::load_branch_thread(db_path, &branch_name, app).await?;
        }
    } else {
        event::load_issue_thread(db_path, app).await?;
    }

    Ok(())
}

async fn handle_edit(db_path: &str, guard: &mut terminal::TuiGuard, app: &mut App) -> Result<()> {
    let initial = match &app.pending_edit {
        Some(EditTarget::Description) => app
            .thread
            .as_ref()
            .map(|t| t.description.clone())
            .unwrap_or_default(),
        Some(EditTarget::Comment(id)) => app
            .thread
            .as_ref()
            .and_then(|t| t.comments.iter().find(|c| c.comment_id == *id))
            .map(|c| c.content.clone())
            .unwrap_or_default(),
        None => return Ok(()),
    };

    let editor = match crate::config::load_or_init()? {
        crate::config::LoadOutcome::Loaded(c) => c.editor,
        crate::config::LoadOutcome::Created(_, c) => c.editor,
    };

    guard.suspend()?;
    let result = editor::spawn_editor(&editor, &initial);
    guard.resume()?;

    apply_edit(db_path, app, result?).await
}

async fn apply_edit(db_path: &str, app: &mut App, content: Option<String>) -> Result<()> {
    let target = match app.pending_edit.take() {
        Some(t) => t,
        None => return Ok(()),
    };

    let content = match content {
        Some(c) => c,
        None => {
            app.show_error("edit cancelled: content was empty".to_string());
            return Ok(());
        }
    };

    let db = crate::db::Database::open(db_path).await?;
    let conn = db.conn();

    match &target {
        crate::tui::app::EditTarget::Description => match &app.tui_mode {
            TuiMode::Issue => {
                let issue_id = app
                    .thread
                    .as_ref()
                    .map(|t| t.issue_id)
                    .ok_or_else(|| Error::Tui("no thread loaded".to_string()))?;
                crate::db::issue::update_description(conn, issue_id, &content).await?;
            }
            TuiMode::Branch { branch_name } => {
                crate::db::branch::update_description(conn, branch_name, &content).await?;
            }
        },
        crate::tui::app::EditTarget::Comment(id) => match &app.tui_mode {
            TuiMode::Issue => {
                crate::db::comment::update_issue_comment(conn, *id, &content).await?;
            }
            TuiMode::Branch { .. } => {
                crate::db::comment::update_branch_comment(conn, *id, &content).await?;
            }
        },
    }

    db.checkpoint().await?;

    let branch_name = match &app.tui_mode {
        TuiMode::Branch { branch_name } => Some(branch_name.clone()),
        TuiMode::Issue => None,
    };

    if let Some(name) = branch_name {
        event::load_branch_thread(db_path, &name, app).await?;
    } else {
        event::load_issue_thread(db_path, app).await?;
    }

    let max = app.thread.as_ref().map(|t| t.comments.len()).unwrap_or(0);
    app.thread_selected = app.thread_selected.min(max);

    Ok(())
}

impl From<bool> for KeyResult {
    fn from(quit: bool) -> Self {
        if quit {
            KeyResult::Quit
        } else {
            KeyResult::Continue
        }
    }
}

fn draw(guard: &mut terminal::TuiGuard, app: &mut App) -> Result<()> {
    guard
        .terminal()
        .draw(|f| {
            match app.screen {
                Screen::IssueList => view::issue_list::render(f, app),
                Screen::Thread => view::thread::render(f, app),
            }
            if app.modal.is_some() {
                view::modal::render(f, app);
            }
        })
        .map_err(|e| Error::Tui(e.to_string()))?;
    Ok(())
}

async fn poll_db(db_path: &str, app: &mut App) -> bool {
    let is_branch = matches!(app.tui_mode, TuiMode::Branch { .. });
    let branch_name = match &app.tui_mode {
        TuiMode::Branch { branch_name } => Some(branch_name.clone()),
        TuiMode::Issue => None,
    };
    let result = if is_branch {
        if let Some(branch_name) = branch_name {
            event::load_branch_thread(db_path, &branch_name, app).await
        } else {
            Ok(false)
        }
    } else {
        match app.screen {
            Screen::IssueList => event::load_issues(db_path, app).await,
            Screen::Thread => event::load_issue_thread(db_path, app).await,
        }
    };
    match result {
        Ok(changed) => changed,
        Err(e) => {
            tracing::warn!("db poll failed: {e}");
            false
        }
    }
}

#[cfg(test)]
mod edit_tests {
    use super::*;
    use crate::db::{Database, author, branch, comment, issue};

    #[tokio::test]
    async fn apply_edit_issue_description_updates_db() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let db_path = tmp.path().join("edit_issue_desc.db");
        let db_path = db_path.to_str().expect("path").to_string();

        let db = Database::open(&db_path).await.expect("open db");
        let conn = db.conn();
        let auth = author::find_or_create(conn, "Alice", Some("a@b.com"))
            .await
            .expect("author");
        let iss = issue::create(conn, Some("test"), "original desc", auth.author_id, None)
            .await
            .expect("issue");
        drop(db);

        let mut app = App::new("Alice".to_string(), Some("a@b.com".to_string()));
        app.screen = Screen::Thread;
        app.tui_mode = TuiMode::Issue;
        app.pending_edit = Some(EditTarget::Description);

        // load the issue thread
        let db = Database::open(&db_path).await.expect("open");
        let conn = db.conn();
        let loaded = issue::get_by_id(conn, iss.issue_id)
            .await
            .expect("get issue");
        drop(db);

        app.issues = vec![loaded.clone()];
        app.selected_issue = 0;
        event::load_issue_thread(&db_path, &mut app)
            .await
            .expect("load thread");

        // Verify initial state
        assert_eq!(
            app.thread.as_ref().expect("thread").description,
            "original desc"
        );

        apply_edit(&db_path, &mut app, Some("updated description".to_string()))
            .await
            .expect("apply edit");

        // Verify DB was updated (thread.reload uses app.issues which may be stale)
        let db = Database::open(&db_path).await.expect("open verify");
        let conn = db.conn();
        let updated = issue::get_by_id(conn, iss.issue_id)
            .await
            .expect("get updated");
        assert_eq!(updated.description, "updated description");
    }

    #[tokio::test]
    async fn apply_edit_issue_comment_updates_db() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let db_path = tmp.path().join("edit_issue_comment.db");
        let db_path = db_path.to_str().expect("path").to_string();

        let db = Database::open(&db_path).await.expect("open db");
        let conn = db.conn();
        let auth = author::find_or_create(conn, "Alice", Some("a@b.com"))
            .await
            .expect("author");
        let iss = issue::create(conn, None, "desc", auth.author_id, None)
            .await
            .expect("issue");
        let c =
            comment::create_issue_comment(conn, iss.issue_id, "original comment", auth.author_id)
                .await
                .expect("comment");
        drop(db);

        let mut app = App::new("Alice".to_string(), Some("a@b.com".to_string()));
        app.screen = Screen::Thread;
        app.tui_mode = TuiMode::Issue;
        app.pending_edit = Some(EditTarget::Comment(c.issue_comment_id));

        app.issues = vec![iss.clone()];
        app.selected_issue = 0;
        event::load_issue_thread(&db_path, &mut app)
            .await
            .expect("load thread");

        let thread = app.thread.as_ref().expect("thread");
        let original_content = thread
            .comments
            .iter()
            .find(|tc| tc.comment_id == c.issue_comment_id)
            .expect("comment")
            .content
            .clone();
        assert_eq!(original_content, "original comment");

        apply_edit(&db_path, &mut app, Some("edited comment".to_string()))
            .await
            .expect("apply edit");

        let thread = app.thread.as_ref().expect("thread after edit");
        let updated_content = thread
            .comments
            .iter()
            .find(|tc| tc.comment_id == c.issue_comment_id)
            .expect("comment")
            .content
            .clone();
        assert_eq!(updated_content, "edited comment");
    }

    #[tokio::test]
    async fn apply_edit_branch_description_updates_db() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let db_path = tmp.path().join("edit_branch_desc.db");
        let db_path = db_path.to_str().expect("path").to_string();

        let db = Database::open(&db_path).await.expect("open db");
        let conn = db.conn();
        let auth = author::find_or_create(conn, "Bob", Some("b@b.com"))
            .await
            .expect("author");
        let iss = issue::create(conn, None, "desc", auth.author_id, None)
            .await
            .expect("issue");
        let br = branch::create(
            conn,
            "feature-x",
            "original branch desc",
            auth.author_id,
            iss.issue_id,
        )
        .await
        .expect("branch");
        drop(db);

        let mut app = App::new("Bob".to_string(), Some("b@b.com".to_string()));
        app.screen = Screen::Thread;
        app.tui_mode = TuiMode::Branch {
            branch_name: br.name.clone(),
        };
        app.pending_edit = Some(EditTarget::Description);

        event::load_branch_thread(&db_path, &br.name, &mut app)
            .await
            .expect("load branch thread");

        let thread = app.thread.as_ref().expect("thread");
        assert_eq!(thread.description, "original branch desc");

        apply_edit(&db_path, &mut app, Some("updated branch desc".to_string()))
            .await
            .expect("apply edit");

        let thread = app.thread.as_ref().expect("thread after edit");
        assert_eq!(thread.description, "updated branch desc");

        // Verify DB directly
        let db = Database::open(&db_path).await.expect("open verify");
        let conn = db.conn();
        let updated = branch::get_by_name(conn, &br.name)
            .await
            .expect("get updated branch");
        assert_eq!(updated.description, "updated branch desc");
    }

    #[tokio::test]
    async fn apply_edit_branch_comment_updates_db() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let db_path = tmp.path().join("edit_branch_comment.db");
        let db_path = db_path.to_str().expect("path").to_string();

        let db = Database::open(&db_path).await.expect("open db");
        let conn = db.conn();
        let auth = author::find_or_create(conn, "Bob", Some("b@b.com"))
            .await
            .expect("author");
        let iss = issue::create(conn, None, "desc", auth.author_id, None)
            .await
            .expect("issue");
        let br = branch::create(conn, "feature-y", "desc", auth.author_id, iss.issue_id)
            .await
            .expect("branch");
        let c = comment::create_branch_comment(conn, br.branch_id, "original", auth.author_id)
            .await
            .expect("comment");
        drop(db);

        let mut app = App::new("Bob".to_string(), Some("b@b.com".to_string()));
        app.screen = Screen::Thread;
        app.tui_mode = TuiMode::Branch {
            branch_name: br.name.clone(),
        };
        app.pending_edit = Some(EditTarget::Comment(c.branch_comment_id));

        event::load_branch_thread(&db_path, &br.name, &mut app)
            .await
            .expect("load thread");

        let thread = app.thread.as_ref().expect("thread");
        let orig = thread
            .comments
            .iter()
            .find(|tc| tc.comment_id == c.branch_comment_id)
            .expect("comment")
            .content
            .clone();
        assert_eq!(orig, "original");

        apply_edit(
            &db_path,
            &mut app,
            Some("edited branch comment".to_string()),
        )
        .await
        .expect("apply edit");

        let thread = app.thread.as_ref().expect("thread after edit");
        let updated = thread
            .comments
            .iter()
            .find(|tc| tc.comment_id == c.branch_comment_id)
            .expect("comment")
            .content
            .clone();
        assert_eq!(updated, "edited branch comment");
    }

    #[tokio::test]
    async fn apply_edit_none_content_shows_error_no_db_write() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let db_path = tmp.path().join("edit_none.db");
        let db_path = db_path.to_str().expect("path").to_string();

        let db = Database::open(&db_path).await.expect("open db");
        let conn = db.conn();
        let auth = author::find_or_create(conn, "Alice", Some("a@b.com"))
            .await
            .expect("author");
        let iss = issue::create(conn, None, "original desc", auth.author_id, None)
            .await
            .expect("issue");
        drop(db);

        let mut app = App::new("Alice".to_string(), Some("a@b.com".to_string()));
        app.screen = Screen::Thread;
        app.tui_mode = TuiMode::Issue;
        app.pending_edit = Some(EditTarget::Description);

        app.issues = vec![iss.clone()];
        app.selected_issue = 0;
        event::load_issue_thread(&db_path, &mut app)
            .await
            .expect("load thread");

        let result = apply_edit(&db_path, &mut app, None).await;
        assert!(result.is_ok());

        // Verify error was shown
        assert!(
            matches!(app.modal, Some(ModalState::Error(ref s)) if s == "edit cancelled: content was empty")
        );

        // Verify DB unchanged
        let db = Database::open(&db_path).await.expect("open verify");
        let conn = db.conn();
        let unchanged = issue::get_by_id(conn, iss.issue_id)
            .await
            .expect("get issue");
        assert_eq!(unchanged.description, "original desc");
    }

    #[tokio::test]
    async fn apply_edit_clamps_thread_selected_after_reload() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let db_path = tmp.path().join("edit_clamp.db");
        let db_path = db_path.to_str().expect("path").to_string();

        let db = Database::open(&db_path).await.expect("open db");
        let conn = db.conn();
        let auth = author::find_or_create(conn, "Alice", Some("a@b.com"))
            .await
            .expect("author");
        let iss = issue::create(conn, None, "desc", auth.author_id, None)
            .await
            .expect("issue");
        comment::create_issue_comment(conn, iss.issue_id, "c1", auth.author_id)
            .await
            .expect("comment 1");
        comment::create_issue_comment(conn, iss.issue_id, "c2", auth.author_id)
            .await
            .expect("comment 2");
        drop(db);

        let mut app = App::new("Alice".to_string(), Some("a@b.com".to_string()));
        app.screen = Screen::Thread;
        app.tui_mode = TuiMode::Issue;
        app.pending_edit = Some(EditTarget::Description);

        app.issues = vec![iss.clone()];
        app.selected_issue = 0;
        event::load_issue_thread(&db_path, &mut app)
            .await
            .expect("load thread");

        // Set thread_selected out of bounds (2 comments = max index 2, set to 5)
        app.thread_selected = 5;

        apply_edit(&db_path, &mut app, Some("updated".to_string()))
            .await
            .expect("apply edit");

        // After reload, thread_selected should be clamped to comments.len() = 2
        let thread = app.thread.as_ref().expect("thread");
        assert_eq!(thread.comments.len(), 2);
        assert!(
            app.thread_selected <= thread.comments.len(),
            "thread_selected={} should be <= comments.len()={}",
            app.thread_selected,
            thread.comments.len()
        );
    }
}
