use crate::tui::model::Thread;

pub enum Screen {
    IssueList,
    Thread,
}

pub struct App {
    pub screen: Screen,
    pub issues: Vec<crate::db::models::Issue>,
    pub selected_issue: usize,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::event::handle_key;
    use crossterm::event::{KeyCode, KeyModifiers};

    fn sample_issues() -> Vec<crate::db::models::Issue> {
        vec![
            crate::db::models::Issue {
                issue_id: 1,
                name: Some("First".to_string()),
                description: "desc".to_string(),
                author: "Alice".to_string(),
                created_at: "2025-01-01 00:00:00".to_string(),
                updated_at: "2025-01-01 00:00:00".to_string(),
            },
            crate::db::models::Issue {
                issue_id: 2,
                name: Some("Second".to_string()),
                description: "desc".to_string(),
                author: "Bob".to_string(),
                created_at: "2025-01-01 00:00:00".to_string(),
                updated_at: "2025-01-01 00:00:00".to_string(),
            },
        ]
    }

    #[test]
    fn select_issue_transitions_to_thread() {
        let mut app = App::new();
        app.issues = sample_issues();
        app.select_issue();
        assert!(matches!(app.screen, Screen::Thread));
        assert_eq!(app.thread_scroll, 0);
    }

    #[test]
    fn select_issue_empty_list_does_nothing() {
        let mut app = App::new();
        app.select_issue();
        assert!(matches!(app.screen, Screen::IssueList));
    }

    #[test]
    fn back_from_thread_goes_to_issue_list() {
        let mut app = App::new();
        app.issues = sample_issues();
        app.select_issue();
        app.back();
        assert!(matches!(app.screen, Screen::IssueList));
        assert!(app.thread.is_none());
        assert_eq!(app.thread_scroll, 0);
    }

    #[test]
    fn back_from_issue_list_does_nothing() {
        let mut app = App::new();
        app.back();
        assert!(matches!(app.screen, Screen::IssueList));
    }

    #[test]
    fn key_q_quits_from_issue_list() {
        let mut app = App::new();
        assert!(handle_key(&mut app, KeyCode::Char('q'), KeyModifiers::NONE));
    }

    #[test]
    fn key_esc_quits_from_issue_list() {
        let mut app = App::new();
        assert!(handle_key(&mut app, KeyCode::Esc, KeyModifiers::NONE));
    }

    #[test]
    fn key_j_moves_down() {
        let mut app = App::new();
        app.issues = sample_issues();
        assert!(!handle_key(
            &mut app,
            KeyCode::Char('j'),
            KeyModifiers::NONE
        ));
        assert_eq!(app.selected_issue, 1);
    }

    #[test]
    fn key_k_moves_up() {
        let mut app = App::new();
        app.issues = sample_issues();
        app.selected_issue = 1;
        assert!(!handle_key(
            &mut app,
            KeyCode::Char('k'),
            KeyModifiers::NONE
        ));
        assert_eq!(app.selected_issue, 0);
    }

    #[test]
    fn key_enter_selects_issue() {
        let mut app = App::new();
        app.issues = sample_issues();
        assert!(!handle_key(&mut app, KeyCode::Enter, KeyModifiers::NONE));
        assert!(matches!(app.screen, Screen::Thread));
    }

    #[test]
    fn key_h_back_from_thread() {
        let mut app = App::new();
        app.issues = sample_issues();
        app.screen = Screen::Thread;
        assert!(!handle_key(
            &mut app,
            KeyCode::Char('h'),
            KeyModifiers::NONE
        ));
        assert!(matches!(app.screen, Screen::IssueList));
    }

    #[test]
    fn key_q_from_thread_returns_to_issue_list() {
        let mut app = App::new();
        app.issues = sample_issues();
        app.screen = Screen::Thread;
        assert!(!handle_key(
            &mut app,
            KeyCode::Char('q'),
            KeyModifiers::NONE
        ));
        assert!(matches!(app.screen, Screen::IssueList));
    }

    #[test]
    fn key_j_scrolls_thread_down() {
        let mut app = App::new();
        app.screen = Screen::Thread;
        assert!(!handle_key(
            &mut app,
            KeyCode::Char('j'),
            KeyModifiers::NONE
        ));
        assert_eq!(app.thread_scroll, 1);
    }

    #[test]
    fn key_k_scrolls_thread_up() {
        let mut app = App::new();
        app.screen = Screen::Thread;
        app.thread_scroll = 5;
        assert!(!handle_key(
            &mut app,
            KeyCode::Char('k'),
            KeyModifiers::NONE
        ));
        assert_eq!(app.thread_scroll, 4);
    }

    #[test]
    fn ctrl_u_pages_up() {
        let mut app = App::new();
        app.screen = Screen::Thread;
        app.thread_scroll = 25;
        assert!(!handle_key(
            &mut app,
            KeyCode::Char('u'),
            KeyModifiers::CONTROL
        ));
        assert_eq!(app.thread_scroll, 5);
    }

    #[test]
    fn ctrl_d_pages_down() {
        let mut app = App::new();
        app.screen = Screen::Thread;
        assert!(!handle_key(
            &mut app,
            KeyCode::Char('d'),
            KeyModifiers::CONTROL
        ));
        assert_eq!(app.thread_scroll, 20);
    }
}
