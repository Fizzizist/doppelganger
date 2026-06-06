use crate::db::Database;
use crate::db::{comment, issue};
use crate::error::{Error, Result};
use crate::tui::{Thread, ThreadComment, event, issue_list, thread_view};
use crossterm::{
    event::{KeyCode, KeyModifiers},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;
use std::time::Duration;

pub enum Screen {
    IssueList(issue_list::IssueListScreen),
    ThreadView(thread_view::ThreadViewScreen),
}

pub struct App {
    screen: Screen,
    db_path: String,
}

impl App {
    pub fn new_issue_list(issues: Vec<crate::db::models::Issue>, db_path: String) -> Self {
        Self {
            screen: Screen::IssueList(issue_list::IssueListScreen::new(issues)),
            db_path,
        }
    }

    pub fn new_thread_view(thread: Thread, comments: Vec<ThreadComment>, db_path: String) -> Self {
        Self {
            screen: Screen::ThreadView(thread_view::ThreadViewScreen::new(thread, comments)),
            db_path,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let _guard = TerminalGuard::setup()?;
        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend).map_err(|e| Error::Tui(e.to_string()))?;
        terminal.clear().map_err(|e| Error::Tui(e.to_string()))?;

        let tick = Duration::from_millis(1000);
        loop {
            match &mut self.screen {
                Screen::IssueList(screen) => {
                    terminal
                        .draw(|frame| screen.render(frame, frame.area()))
                        .map_err(|e| Error::Tui(e.to_string()))?;

                    match event::next_key(tick) {
                        Ok(Some(key)) => match key.code {
                            KeyCode::Char('j') | KeyCode::Down => screen.select_down(),
                            KeyCode::Char('k') | KeyCode::Up => screen.select_up(),
                            KeyCode::Enter => {
                                if let Some(issue_id) = screen.selected_issue_id() {
                                    let thread_view = self.load_issue_thread(issue_id).await?;
                                    self.screen = Screen::ThreadView(thread_view);
                                }
                            }
                            KeyCode::Char('q') => return Ok(()),
                            _ => {}
                        },
                        Ok(None) => {}
                        Err(e) => return Err(Error::Tui(e.to_string())),
                    }
                }
                Screen::ThreadView(screen) => {
                    terminal
                        .draw(|frame| screen.render(frame, frame.area()))
                        .map_err(|e| Error::Tui(e.to_string()))?;

                    match event::next_key(tick) {
                        Ok(Some(key)) => match key.code {
                            KeyCode::Char('j') | KeyCode::Down => screen.scroll_down(1),
                            KeyCode::Char('k') | KeyCode::Up => screen.scroll_up(1),
                            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                screen.scroll_down(10)
                            }
                            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                screen.scroll_up(10)
                            }
                            KeyCode::Char('q') => {
                                let issues = self.load_issues().await?;
                                self.screen =
                                    Screen::IssueList(issue_list::IssueListScreen::new(issues));
                            }
                            _ => {}
                        },
                        Ok(None) => {}
                        Err(e) => return Err(Error::Tui(e.to_string())),
                    }
                }
            }
        }
    }

    async fn load_issue_thread(&self, issue_id: i64) -> Result<thread_view::ThreadViewScreen> {
        let db = Database::open(&self.db_path).await?;
        let conn = db.conn();
        let iss = issue::get_by_id(conn, issue_id).await?;
        let comments = comment::list_issue_comments(conn, iss.issue_id).await?;
        let thread = Thread::from(iss);
        let thread_comments: Vec<ThreadComment> =
            comments.into_iter().map(ThreadComment::from).collect();
        Ok(thread_view::ThreadViewScreen::new(thread, thread_comments))
    }

    async fn load_issues(&self) -> Result<Vec<crate::db::models::Issue>> {
        let db = Database::open(&self.db_path).await?;
        let conn = db.conn();
        issue::list_issues(conn).await
    }
}

struct TerminalGuard;

impl TerminalGuard {
    fn setup() -> Result<Self> {
        terminal::enable_raw_mode().map_err(|e| Error::Tui(e.to_string()))?;
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, EnterAlternateScreen).map_err(|e| Error::Tui(e.to_string()))?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = crossterm::execute!(io::stdout(), LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}
