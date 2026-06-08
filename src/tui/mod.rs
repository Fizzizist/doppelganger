pub mod app;
pub mod event;
pub mod highlight;
pub mod model;
pub mod terminal;
pub mod view;

pub use app::App;
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

pub async fn run_issue_tui(db_path: &str) -> Result<()> {
    crate::logging::init();
    let mut app = App::new();
    if let Err(e) = event::load_issues(db_path, &mut app).await {
        tracing::warn!("initial issue load failed: {e}");
    }
    let mut guard = terminal::TuiGuard::init()?;
    run_loop(db_path, TuiMode::Issue, &mut guard, &mut app).await
}

pub async fn run_branch_tui(db_path: &str, repo: &git2::Repository) -> Result<()> {
    crate::logging::init();
    let branch_name = crate::git::current_branch(repo)?;
    let mut app = App::new();
    // Pre-flight: load branch data before entering raw terminal mode so errors surface cleanly.
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
                        if dispatch_key(app, &mode, key.code, key.modifiers) {
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

fn draw(guard: &mut terminal::TuiGuard, app: &App) -> Result<()> {
    guard
        .terminal()
        .draw(|f| match app.screen {
            Screen::IssueList => view::issue_list::render(f, app),
            Screen::Thread => view::thread::render(f, app),
        })
        .map_err(|e| Error::Tui(e.to_string()))?;
    Ok(())
}

fn dispatch_key(app: &mut App, mode: &TuiMode, code: KeyCode, modifiers: KeyModifiers) -> bool {
    match mode {
        TuiMode::Issue => event::handle_key(app, code, modifiers),
        TuiMode::Branch { .. } => {
            match code {
                KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('h') => return true,
                _ => event::handle_thread_scroll(app, code, modifiers),
            }
            false
        }
    }
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
