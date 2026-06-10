pub mod app;
pub mod editor;
pub mod event;
pub mod highlight;
pub mod model;
pub mod terminal;
pub mod view;

pub use app::{App, ModalState};
pub use model::{Thread, ThreadComment};

use crossterm::event::{Event as CrosstermEvent, EventStream, KeyCode, KeyModifiers};
use futures_util::StreamExt;
use tokio::time::{MissedTickBehavior, interval_at};

use crate::error::{Error, Result};
use crate::tui::app::Screen;

const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

enum TuiMode {
    Issue,
    Branch { branch_name: String },
}

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
                        let should_quit = handle_key_event(db_path, &mode, guard, app, key.code, key.modifiers).await?;
                        if should_quit {
                            break;
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
) -> Result<bool> {
    if app.modal.is_some() {
        if matches!(&app.modal, Some(ModalState::NameInput)) && code == KeyCode::Enter {
            let name = app.confirm_name_input();
            let name_opt = if name.is_empty() {
                None
            } else {
                Some(name.as_str())
            };

            guard.suspend()?;

            let editor = crate::config::load_or_init()
                .ok()
                .and_then(|outcome| match outcome {
                    crate::config::LoadOutcome::Loaded(c) => Some(c.editor),
                    crate::config::LoadOutcome::Created(_) => None,
                })
                .unwrap_or_else(crate::config::default_editor);

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
                    let created =
                        crate::db::issue::create(conn, name_opt, &description, author.author_id)
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

            return Ok(false);
        }

        return Ok(event::handle_modal_key(app, code, modifiers));
    }

    if matches!(mode, TuiMode::Branch { .. }) {
        match code {
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('h') => return Ok(true),
            _ => {
                event::handle_thread_scroll(app, code, modifiers);
                return Ok(false);
            }
        }
    }

    Ok(event::handle_key(app, code, modifiers))
}

fn draw(guard: &mut terminal::TuiGuard, app: &App) -> Result<()> {
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
    match mode {
        TuiMode::Issue => match app.screen {
            Screen::IssueList => event::load_issues(db_path, app).await.unwrap_or_else(|e| {
                tracing::warn!("DB poll failed: {e}");
                false
            }),
            Screen::Thread => event::load_issue_thread(db_path, app)
                .await
                .unwrap_or_else(|e| {
                    tracing::warn!("DB poll failed: {e}");
                    false
                }),
        },
        TuiMode::Branch { branch_name } => event::load_branch_thread(db_path, branch_name, app)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!("DB poll failed: {e}");
                false
            }),
    }
}
