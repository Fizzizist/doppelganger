use crate::tui::model::Thread;

pub enum ModalState {
    NameInput,
    Error(String),
}

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
    pub modal: Option<ModalState>,
    pub input_buffer: String,
    pub author_name: String,
    pub author_email: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        Self::new("default".to_string(), None)
    }
}

impl App {
    pub fn new(author_name: String, author_email: Option<String>) -> Self {
        Self {
            screen: Screen::IssueList,
            issues: Vec::new(),
            selected_issue: 0,
            thread: None,
            thread_scroll: 0,
            last_fingerprint: String::new(),
            modal: None,
            input_buffer: String::new(),
            author_name,
            author_email,
        }
    }

    pub fn start_name_input(&mut self) {
        self.modal = Some(ModalState::NameInput);
        self.input_buffer.clear();
    }

    pub fn cancel_modal(&mut self) {
        self.modal = None;
        self.input_buffer.clear();
    }

    pub fn confirm_name_input(&mut self) -> String {
        let name = self.input_buffer.clone();
        self.modal = None;
        self.input_buffer.clear();
        name
    }

    pub fn show_error(&mut self, msg: impl Into<String>) {
        self.modal = Some(ModalState::Error(msg.into()));
        self.input_buffer.clear();
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
        let mut app = App::default();
        app.issues = sample_issues();
        app.select_issue();
        assert!(matches!(app.screen, Screen::Thread));
        assert_eq!(app.thread_scroll, 0);
    }

    #[test]
    fn select_issue_empty_list_does_nothing() {
        let mut app = App::default();
        app.select_issue();
        assert!(matches!(app.screen, Screen::IssueList));
    }

    #[test]
    fn back_from_thread_goes_to_issue_list() {
        let mut app = App::default();
        app.issues = sample_issues();
        app.select_issue();
        app.back();
        assert!(matches!(app.screen, Screen::IssueList));
        assert!(app.thread.is_none());
        assert_eq!(app.thread_scroll, 0);
    }

    #[test]
    fn back_from_issue_list_does_nothing() {
        let mut app = App::default();
        app.back();
        assert!(matches!(app.screen, Screen::IssueList));
    }

    #[test]
    fn key_q_quits_from_issue_list() {
        let mut app = App::default();
        assert!(handle_key(&mut app, KeyCode::Char('q'), KeyModifiers::NONE));
    }

    #[test]
    fn key_esc_quits_from_issue_list() {
        let mut app = App::default();
        assert!(handle_key(&mut app, KeyCode::Esc, KeyModifiers::NONE));
    }

    #[test]
    fn key_j_moves_down() {
        let mut app = App::default();
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
        let mut app = App::default();
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
        let mut app = App::default();
        app.issues = sample_issues();
        assert!(!handle_key(&mut app, KeyCode::Enter, KeyModifiers::NONE));
        assert!(matches!(app.screen, Screen::Thread));
    }

    #[test]
    fn key_h_back_from_thread() {
        let mut app = App::default();
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
        let mut app = App::default();
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
        let mut app = App::default();
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
        let mut app = App::default();
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
        let mut app = App::default();
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
        let mut app = App::new("test".to_string(), None);
        app.screen = Screen::Thread;
        assert!(!handle_key(
            &mut app,
            KeyCode::Char('d'),
            KeyModifiers::CONTROL
        ));
        assert_eq!(app.thread_scroll, 20);
    }

    #[test]
    fn start_name_input_sets_modal_and_clears_buffer() {
        let mut app = App::new("test".to_string(), None);
        app.input_buffer = "leftover".to_string();
        app.start_name_input();
        assert!(matches!(app.modal, Some(ModalState::NameInput)));
        assert!(app.input_buffer.is_empty());
    }

    #[test]
    fn cancel_modal_clears_modal_and_buffer() {
        let mut app = App::new("test".to_string(), None);
        app.start_name_input();
        app.input_buffer = "hello".to_string();
        app.cancel_modal();
        assert!(app.modal.is_none());
        assert!(app.input_buffer.is_empty());
    }

    #[test]
    fn confirm_name_input_returns_buffer_and_clears() {
        let mut app = App::new("test".to_string(), None);
        app.start_name_input();
        app.input_buffer = "my issue".to_string();
        let name = app.confirm_name_input();
        assert_eq!(name, "my issue");
        assert!(app.modal.is_none());
        assert!(app.input_buffer.is_empty());
    }

    #[test]
    fn show_error_sets_error_modal() {
        let mut app = App::new("test".to_string(), None);
        app.show_error("something went wrong");
        assert!(matches!(app.modal, Some(ModalState::Error(ref s)) if s == "something went wrong"));
        assert!(app.input_buffer.is_empty());
    }

    #[test]
    fn key_n_starts_name_input_from_issue_list() {
        let mut app = App::new("test".to_string(), None);
        app.issues = sample_issues();
        assert!(!handle_key(
            &mut app,
            KeyCode::Char('n'),
            KeyModifiers::NONE
        ));
        assert!(matches!(app.modal, Some(ModalState::NameInput)));
    }

    #[test]
    fn modal_key_typing_appends_to_buffer() {
        let mut app = App::new("test".to_string(), None);
        app.start_name_input();
        assert!(!handle_key(
            &mut app,
            KeyCode::Char('a'),
            KeyModifiers::NONE
        ));
        assert_eq!(app.input_buffer, "a");
        assert!(!handle_key(
            &mut app,
            KeyCode::Char('b'),
            KeyModifiers::NONE
        ));
        assert_eq!(app.input_buffer, "ab");
    }

    #[test]
    fn modal_backspace_removes_last_char() {
        let mut app = App::new("test".to_string(), None);
        app.start_name_input();
        app.input_buffer = "abc".to_string();
        assert!(!handle_key(
            &mut app,
            KeyCode::Backspace,
            KeyModifiers::NONE
        ));
        assert_eq!(app.input_buffer, "ab");
    }

    #[test]
    fn modal_esc_cancels() {
        let mut app = App::new("test".to_string(), None);
        app.start_name_input();
        app.input_buffer = "typed".to_string();
        assert!(!handle_key(&mut app, KeyCode::Esc, KeyModifiers::NONE));
        assert!(app.modal.is_none());
        assert!(app.input_buffer.is_empty());
    }

    #[test]
    fn modal_enter_confirms_name() {
        let mut app = App::new("test".to_string(), None);
        app.start_name_input();
        app.input_buffer = "test issue".to_string();
        let quit = handle_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
        assert!(!quit);
        assert!(app.modal.is_none());
        assert!(app.input_buffer.is_empty());
    }

    #[test]
    fn error_modal_esc_dismisses() {
        let mut app = App::new("test".to_string(), None);
        app.show_error("fail");
        assert!(!handle_key(&mut app, KeyCode::Esc, KeyModifiers::NONE));
        assert!(app.modal.is_none());
    }
}
