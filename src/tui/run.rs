use ratatui::crossterm::{
    event::{KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

use crate::db::{Database, branch, comment, issue};
use crate::error::Result;
use crate::tui::app::{App, Screen};
use crate::tui::event::AppEvent;
use crate::tui::thread::Thread;
use crate::tui::ui;

struct TuiGuard;

impl TuiGuard {
    fn init() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        Ok(TuiGuard)
    }
}

impl Drop for TuiGuard {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

pub async fn run_tui(db_path: &str) -> Result<()> {
    let db = Database::open(db_path).await?;
    let issues = issue::list(db.conn()).await?;
    drop(db);

    let guard = TuiGuard::init().map_err(|e| crate::error::Error::Tui(e.to_string()))?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal =
        Terminal::new(backend).map_err(|e| crate::error::Error::Tui(e.to_string()))?;

    let mut app = App::new(issues);
    let result = event_loop(&mut terminal, db_path, &mut app).await;
    drop(guard);
    result
}

pub async fn run_branch_tui(db_path: &str, branch_name: &str) -> Result<()> {
    let db = Database::open(db_path).await?;
    let br = branch::get_by_name(db.conn(), branch_name).await?;
    let comments = comment::list_branch_comments(db.conn(), br.branch_id).await?;
    let thread = Thread::from((&br, comments));
    drop(db);

    let guard = TuiGuard::init().map_err(|e| crate::error::Error::Tui(e.to_string()))?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal =
        Terminal::new(backend).map_err(|e| crate::error::Error::Tui(e.to_string()))?;

    let mut app = App::new(Vec::new());
    app.enter_thread(thread);

    let result = branch_event_loop(&mut terminal, db_path, &mut app, branch_name).await;
    drop(guard);
    result
}

async fn event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    db_path: &str,
    app: &mut App,
) -> Result<()> {
    let tick_interval = std::time::Duration::from_millis(250);

    loop {
        terminal
            .draw(|f| ui::draw(f, app))
            .map_err(|e| crate::error::Error::Tui(e.to_string()))?;

        let event = crate::tui::event::next_event(tick_interval).await?;

        match event {
            AppEvent::Key(key) => match key.code {
                KeyCode::Char('q') => {
                    if app.screen == Screen::Thread {
                        app.exit_thread();
                    } else {
                        return Ok(());
                    }
                }
                KeyCode::Char('h') | KeyCode::Esc if app.screen == Screen::Thread => {
                    app.exit_thread();
                }
                KeyCode::Char('j') | KeyCode::Down if app.screen == Screen::IssueList => {
                    app.select_next();
                }
                KeyCode::Char('k') | KeyCode::Up if app.screen == Screen::IssueList => {
                    app.select_prev();
                }
                KeyCode::Char('l') | KeyCode::Enter
                    if app.screen == Screen::IssueList && !app.issues.is_empty() =>
                {
                    let issue_id = app.issues[app.selected].issue_id;
                    let db = Database::open(db_path).await?;
                    let selected_issue = issue::get_by_id(db.conn(), issue_id).await?;
                    let comments = comment::list_issue_comments(db.conn(), issue_id).await?;
                    drop(db);
                    let thread = Thread::from((&selected_issue, comments));
                    app.enter_thread(thread);
                }
                KeyCode::Char('u')
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && app.screen == Screen::Thread =>
                {
                    app.scroll_up(10);
                }
                KeyCode::Char('d')
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && app.screen == Screen::Thread =>
                {
                    app.scroll_down(10);
                }
                _ => {}
            },
            AppEvent::Tick => {
                if app.screen == Screen::IssueList {
                    let _ = poll_issue_list(db_path, app).await;
                } else if app.screen == Screen::Thread {
                    let _ = poll_thread_update(db_path, app).await;
                }
            }
        }
    }
}

async fn poll_issue_list(db_path: &str, app: &mut App) -> Result<()> {
    use crate::db::row::extract_optional_text;

    let db = Database::open(db_path).await?;
    let conn = db.conn();
    let mut rows = conn.query("SELECT MAX(updated_at) FROM issue", ()).await?;

    let max_ts = match rows.next().await? {
        Some(row) => extract_optional_text(&row, 0)?,
        None => return Ok(()),
    };

    if app.last_poll_timestamp.as_ref() != max_ts.as_ref() {
        app.issues = issue::list(conn).await?;
        app.last_poll_timestamp = max_ts;
    }
    Ok(())
}

async fn poll_thread_update(db_path: &str, app: &mut App) -> Result<()> {
    let issue_id = match app.issues.get(app.selected) {
        Some(i) => i.issue_id,
        None => return Ok(()),
    };

    let db = Database::open(db_path).await?;
    let conn = db.conn();
    let comments = comment::list_issue_comments(conn, issue_id).await?;

    let max_ts = comments
        .iter()
        .map(|c| c.updated_at.as_str())
        .max()
        .map(String::from);

    if max_ts.as_ref() == app.last_poll_timestamp.as_ref() {
        return Ok(());
    }

    let selected_issue = issue::get_by_id(conn, issue_id).await?;
    drop(db);
    let updated_thread = Thread::from((&selected_issue, comments));
    app.thread = Some(updated_thread);
    app.last_poll_timestamp = max_ts;
    Ok(())
}

async fn branch_event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    db_path: &str,
    app: &mut App,
    branch_name: &str,
) -> Result<()> {
    let tick_interval = std::time::Duration::from_millis(250);

    loop {
        terminal
            .draw(|f| ui::draw(f, app))
            .map_err(|e| crate::error::Error::Tui(e.to_string()))?;

        let event = crate::tui::event::next_event(tick_interval).await?;

        match event {
            AppEvent::Key(key) => match key.code {
                KeyCode::Char('q') | KeyCode::Char('h') | KeyCode::Esc => {
                    return Ok(());
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.scroll_up(10);
                }
                KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.scroll_down(10);
                }
                _ => {}
            },
            AppEvent::Tick => {
                let db = match Database::open(db_path).await {
                    Ok(db) => db,
                    Err(_) => continue,
                };
                let br = match branch::get_by_name(db.conn(), branch_name).await {
                    Ok(b) => b,
                    Err(_) => continue,
                };
                let comments = comment::list_branch_comments(db.conn(), br.branch_id).await?;
                let max_ts = comments
                    .iter()
                    .map(|c| c.updated_at.as_str())
                    .max()
                    .map(String::from);
                if max_ts.as_ref() != app.last_poll_timestamp.as_ref() {
                    let updated_thread = Thread::from((&br, comments));
                    app.thread = Some(updated_thread);
                    app.last_poll_timestamp = max_ts;
                }
            }
        }
    }
}
