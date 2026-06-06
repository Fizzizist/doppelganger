use crate::db::models::Issue;
use crate::tui::thread::Thread;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    IssueList,
    Thread,
}

pub struct App {
    pub screen: Screen,
    pub issues: Vec<Issue>,
    pub selected: usize,
    pub thread: Option<Thread>,
    pub thread_scroll: u16,
    pub last_poll_timestamp: Option<String>,
}

impl App {
    pub fn new(issues: Vec<Issue>) -> Self {
        App {
            screen: Screen::IssueList,
            issues,
            selected: 0,
            thread: None,
            thread_scroll: 0,
            last_poll_timestamp: None,
        }
    }

    pub fn select_next(&mut self) {
        if !self.issues.is_empty() {
            self.selected = (self.selected + 1).min(self.issues.len() - 1);
        }
    }

    pub fn select_prev(&mut self) {
        if !self.issues.is_empty() && self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn scroll_up(&mut self, amount: u16) {
        self.thread_scroll = self.thread_scroll.saturating_sub(amount);
    }

    pub fn scroll_down(&mut self, amount: u16) {
        self.thread_scroll = self.thread_scroll.saturating_add(amount);
    }

    pub fn enter_thread(&mut self, thread: Thread) {
        self.thread = Some(thread);
        self.thread_scroll = 0;
        self.screen = Screen::Thread;
    }

    pub fn exit_thread(&mut self) {
        self.thread = None;
        self.thread_scroll = 0;
        self.screen = Screen::IssueList;
    }
}
