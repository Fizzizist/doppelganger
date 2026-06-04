use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::model::Thread;

pub enum Screen {
    IssueList,
    Thread(Thread),
}

pub enum Action {
    OpenIssueThread(i64),
    BackToList,
    Quit,
}

pub struct App {
    pub screen: Screen,
    pub issues: Vec<crate::db::models::Issue>,
    pub selected: usize,
    pub scroll: u16,
    pub should_quit: bool,
    pub has_issue_list: bool,
    pending_action: Option<Action>,
}

impl App {
    pub fn new_issue_list(issues: Vec<crate::db::models::Issue>) -> Self {
        Self {
            screen: Screen::IssueList,
            issues,
            selected: 0,
            scroll: 0,
            should_quit: false,
            has_issue_list: true,
            pending_action: None,
        }
    }

    pub fn new_thread(thread: Thread) -> Self {
        Self {
            screen: Screen::Thread(thread),
            issues: Vec::new(),
            selected: 0,
            scroll: 0,
            should_quit: false,
            has_issue_list: false,
            pending_action: None,
        }
    }

    pub fn take_action(&mut self) -> Option<Action> {
        self.pending_action.take()
    }

    pub fn set_issues(&mut self, issues: Vec<crate::db::models::Issue>) {
        self.issues = issues;
        self.selected = 0;
    }

    pub fn transition_to_thread(&mut self, thread: Thread) {
        self.screen = Screen::Thread(thread);
        self.scroll = 0;
    }

    pub fn replace_thread(&mut self, thread: Thread) {
        // Live reload of the current thread; preserve the scroll position.
        self.screen = Screen::Thread(thread);
    }

    pub fn transition_to_list(&mut self) {
        self.screen = Screen::IssueList;
        self.selected = 0;
        self.scroll = 0;
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match &self.screen {
            Screen::IssueList => self.handle_issue_list_key(key),
            Screen::Thread(_) => self.handle_thread_key(key),
        }
    }

    fn handle_issue_list_key(&mut self, key: KeyEvent) {
        if key.code != KeyCode::Char('j')
            && key.code != KeyCode::Char('k')
            && key.code != KeyCode::Enter
            && key.code != KeyCode::Char('l')
            && key.code != KeyCode::Char('q')
        {
            return;
        }

        match key.code {
            KeyCode::Char('j') if self.selected + 1 < self.issues.len() => {
                self.selected += 1;
            }
            KeyCode::Char('j') => {}
            KeyCode::Char('k') if self.selected > 0 => {
                self.selected -= 1;
            }
            KeyCode::Char('k') => {}
            KeyCode::Enter | KeyCode::Char('l') => {
                if let Some(issue) = self.issues.get(self.selected) {
                    self.pending_action = Some(Action::OpenIssueThread(issue.issue_id));
                }
            }
            KeyCode::Char('q') => {
                self.pending_action = Some(Action::Quit);
            }
            _ => {}
        }
    }

    fn handle_thread_key(&mut self, key: KeyEvent) {
        if key.code != KeyCode::Char('j')
            && key.code != KeyCode::Char('k')
            && key.code != KeyCode::Char('u')
            && key.code != KeyCode::Char('d')
            && key.code != KeyCode::Char('q')
            && key.code != KeyCode::Char('h')
        {
            return;
        }

        match key.code {
            KeyCode::Char('j') => {
                self.scroll = self.scroll.saturating_add(1);
            }
            KeyCode::Char('k') => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll = self.scroll.saturating_sub(10);
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll = self.scroll.saturating_add(10);
            }
            KeyCode::Char('q') | KeyCode::Char('h') => {
                if self.has_issue_list {
                    self.pending_action = Some(Action::BackToList);
                } else {
                    self.pending_action = Some(Action::Quit);
                }
            }
            _ => {}
        }
    }
}
