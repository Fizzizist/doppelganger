pub mod issue_list;
pub mod thread;

use ratatui::Frame;

use super::app::{App, Screen};

pub fn render(frame: &mut Frame, app: &App) {
    match &app.screen {
        Screen::IssueList => issue_list::render(frame, app),
        Screen::Thread(t) => thread::render(frame, app, t),
    }
}
