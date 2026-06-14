pub mod app;
pub mod editor;
pub mod event;
pub mod highlight;
pub mod input_box;
pub mod model;
pub mod terminal;
pub mod view;

pub use app::{App, Focus, ModalState, TuiMode};
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
    run_loop(db_path, TuiMode::Issue, &mut guard, &mut app).await
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
    run_loop(
        db_path,
        TuiMode::Branch { branch_name },
        &mut guard,
        &mut app,
    )
    .await
}

async fn run_loop(
    db_path: &str,
    mode: TuiMode,
    guard: &mut terminal::TuiGuard,
    app: &mut App,
) -> Result<()> {
    let mut events = EventStream::new();
    let mut tick = interval_at(tokio::time::Instant::now() + POLL_INTERVAL, POLL_INTERVAL);
    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);

    draw(guard, app)?;

    loop {
        tokio::select! {
            _ = tick.tick() => {
                if poll_db(db_path, &mode, app).await {
                    draw(guard, app)?;
                }
            }
            maybe_event = events.next() => {
                match maybe_event {
                    Some(Ok(CrosstermEvent::Key(key))) => {
                        match handle_key_event(db_path, &mode, guard, app, key.code, key.modifiers).await? {
                            KeyResult::Quit => break,
                            KeyResult::SubmitComment => {
                                if let Err(e) = submit_comment(db_path, &mode, app).await {
                                    app.show_error(e.to_string());
                                }
                            }
                            KeyResult::Continue => {}
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
    mode: &TuiMode,
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

            let content_result = editor::spawn_editor(&editor);

            guard.resume()?;

            match content_result {
                Ok(description) => {
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

    if matches!(mode, TuiMode::Branch { .. })
        && was_thread
        && matches!(app.screen, Screen::IssueList)
    {
        return Ok(KeyResult::Quit);
    }

    Ok(result)
}

async fn submit_comment(db_path: &str, mode: &TuiMode, app: &mut App) -> Result<()> {
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

    match mode {
        TuiMode::Issue => {
            let issue_idx = app.selected_issue;
            if issue_idx >= app.issues.len() {
                return Ok(());
            }
            let issue_id = app.issues[issue_idx].issue_id;
            db::comment::create_issue_comment(conn, issue_id, &content, author.author_id).await?;
            event::load_issue_thread(db_path, app).await?;
        }
        TuiMode::Branch { branch_name } => {
            let branch = db::branch::get_by_name(conn, branch_name).await?;
            db::comment::create_branch_comment(conn, branch.branch_id, &content, author.author_id)
                .await?;
            event::load_branch_thread(db_path, branch_name, app).await?;
        }
    }

    db.checkpoint().await?;

    if let Some(editor) = app.input_editor.as_mut() {
        editor.set_text("");
        editor.enter_normal();
    }
    app.focus_thread();

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

async fn poll_db(db_path: &str, mode: &TuiMode, app: &mut App) -> bool {
    let result = match mode {
        TuiMode::Issue => match app.screen {
            Screen::IssueList => event::load_issues(db_path, app).await,
            Screen::Thread => event::load_issue_thread(db_path, app).await,
        },
        TuiMode::Branch { branch_name } => {
            event::load_branch_thread(db_path, branch_name, app).await
        }
    };
    if let Err(ref e) = result {
        tracing::warn!("db poll failed: {e}");
    }
    result.unwrap_or(false)
}
