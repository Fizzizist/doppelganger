pub mod app;
pub mod event;
pub mod model;
pub mod terminal;
pub mod view;

pub use app::App;
pub use model::{Thread, ThreadComment};

use crate::db::Database;
use crate::tui::app::Screen;

pub async fn run_issue_tui(db_path: &str) -> crate::error::Result<()> {
    crate::logging::init();

    let mut guard = terminal::TuiGuard::init()?;
    let mut app = App::new();

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

        let evt = event::next_event().await?;
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
    crate::logging::init();

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

    loop {
        {
            let term = guard.terminal();
            term.draw(|f| view::thread::render(f, &app))
                .map_err(|e| crate::error::Error::Tui(e.to_string()))?;
        }

        let evt = event::next_event().await?;
        match evt {
            event::AppEvent::Key(code, modifiers) => {
                if event::handle_key(&mut app, code, modifiers) {
                    break;
                }
                if matches!(app.screen, Screen::IssueList) {
                    break;
                }
            }
            event::AppEvent::Tick => {
                if let Err(e) = event::load_branch_thread(db_path, &branch_name, &mut app).await {
                    tracing::warn!("failed to reload branch thread: {e}");
                }
            }
        }
    }

    Ok(())
}
