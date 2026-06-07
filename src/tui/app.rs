use crate::tui::model::Thread;

pub enum Screen {
    IssueList,
    Thread,
}

pub struct App {
    pub screen: Screen,
    pub issues: Vec<crate::db::models::Issue>,
    pub selected_issue: usize,
    pub issue_scroll: u16,
    pub thread: Option<Thread>,
    pub thread_scroll: u16,
    pub last_fingerprint: String,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::IssueList,
            issues: Vec::new(),
            selected_issue: 0,
            issue_scroll: 0,
            thread: None,
            thread_scroll: 0,
            last_fingerprint: String::new(),
        }
    }

    pub fn select_issue(&mut self) {
        if !self.issues.is_empty() {
            self.screen = Screen::Thread;
            self.thread_scroll = 0;
        }
    }

    pub fn back(&mut self) {
        match self.screen {
            Screen::Thread => {
                self.screen = Screen::IssueList;
                self.thread = None;
                self.thread_scroll = 0;
            }
            Screen::IssueList => {}
        }
    }
}
