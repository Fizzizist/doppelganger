pub mod app;
pub mod event;
pub mod highlight;
pub mod model;
pub mod terminal;
pub mod view;

pub use app::App;
pub use model::{Thread, ThreadComment};

use crate::db::Database;
use crate::tui::app::Screen;
use crossterm::event::{EventStream, KeyCode};

fn log_path_from_db_path(db_path: &str) -> String {
    if let Some(stripped) = db_path.strip_suffix(".db") {
        format!("{stripped}.log")
    } else {
        format!("{db_path}.log")
    }
}

pub async fn run_issue_tui(db_path: &str) -> crate::error::Result<()> {
    crate::logging::init(&log_path_from_db_path(db_path));

    let mut guard = terminal::TuiGuard::init()?;
    let mut app = App::new();
    let mut events = EventStream::new();

    event::load_issues(db_path, &mut app).await?;

    loop {
        {
            let term = guard.terminal();
            term.draw(|f| match app.screen {
                Screen::IssueList => view::issue_list::render(f, &app),
                Screen::Thread => view::thread::render(f, &app),
            })
            .map_err(|e| crate::error::Error::Tui(e.to_string()))?;
        }

        let evt = event::next_event(&mut events).await?;
        match evt {
            event::AppEvent::Key(code, modifiers) => {
                if event::handle_key(&mut app, code, modifiers) {
                    break;
                }
                if matches!(app.screen, Screen::Thread)
                    && app.thread.is_none()
                    && let Err(e) = event::load_issue_thread(db_path, &mut app).await
                {
                    tracing::warn!("failed to load thread: {e}");
                }
            }
            event::AppEvent::Tick => match app.screen {
                Screen::IssueList => {
                    if let Err(e) = event::load_issues(db_path, &mut app).await {
                        tracing::warn!("failed to reload issues: {e}");
                    }
                }
                Screen::Thread => {
                    if let Err(e) = event::load_issue_thread(db_path, &mut app).await {
                        tracing::warn!("failed to reload thread: {e}");
                    }
                }
            },
        }
    }

    Ok(())
}

pub async fn run_branch_tui(db_path: &str, repo: &git2::Repository) -> crate::error::Result<()> {
    crate::logging::init(&log_path_from_db_path(db_path));

    let branch_name = crate::git::current_branch(repo)?;

    let db = Database::open(db_path).await?;
    let br = crate::db::branch::get_by_name(db.conn(), &branch_name).await?;
    let comments = crate::db::comment::list_branch_comments(db.conn(), br.branch_id).await?;
    drop(db);

    let thread = Thread::from(&crate::db::models::BranchWithComments {
        branch: br,
        comments,
    });

    let mut guard = terminal::TuiGuard::init()?;
    let mut app = App::new();
    app.screen = Screen::Thread;
    app.thread = Some(thread);

    let mut events = EventStream::new();

    loop {
        {
            let term = guard.terminal();
            term.draw(|f| view::thread::render(f, &app))
                .map_err(|e| crate::error::Error::Tui(e.to_string()))?;
        }

        let evt = event::next_event(&mut events).await?;
        match evt {
            event::AppEvent::Key(code, modifiers) => match code {
                KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('h') => break,
                KeyCode::Char('j') | KeyCode::Down => {
                    app.thread_scroll = app.thread_scroll.saturating_add(1);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    app.thread_scroll = app.thread_scroll.saturating_sub(1);
                }
                KeyCode::Char('u')
                    if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) =>
                {
                    app.thread_scroll = app.thread_scroll.saturating_sub(20);
                }
                KeyCode::Char('d')
                    if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) =>
                {
                    app.thread_scroll = app.thread_scroll.saturating_add(20);
                }
                _ => {}
            },
            event::AppEvent::Tick => {
                if let Err(e) = event::load_branch_thread(db_path, &branch_name, &mut app).await {
                    tracing::warn!("failed to reload branch thread: {e}");
                }
            }
        }
    }

    Ok(())
}
