use crate::tui::model::Thread;
use hjkl_form::TextFieldEditor;

pub enum ModalState {
    NameInput,
    Error(String),
}

pub enum Screen {
    IssueList,
    Thread,
}

#[derive(Default)]
pub enum Focus {
    #[default]
    Thread,
    InputBox,
}

#[derive(Default)]
pub enum TuiMode {
    #[default]
    Issue,
    Branch {
        branch_name: String,
    },
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
    pub input_editor: Option<TextFieldEditor>,
    pub focus: Focus,
    pub ctrl_w_pending: bool,
    pub tui_mode: TuiMode,
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
            input_editor: None,
            focus: Focus::default(),
            ctrl_w_pending: false,
            tui_mode: TuiMode::default(),
        }
    }

    pub fn focus_input_box(&mut self) {
        self.focus = Focus::InputBox;
        let editor = self
            .input_editor
            .get_or_insert_with(|| TextFieldEditor::new(false));
        editor.enter_insert_at_end();
    }

    pub fn focus_thread(&mut self) {
        self.focus = Focus::Thread;
        if let Some(editor) = self.input_editor.as_mut() {
            editor.enter_normal();
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
            self.tui_mode = TuiMode::Issue;
        }
    }

    pub fn back(&mut self) {
        match self.screen {
            Screen::Thread => {
                self.screen = Screen::IssueList;
                self.thread = None;
                self.thread_scroll = 0;
                self.focus = Focus::Thread;
                self.ctrl_w_pending = false;
                self.input_editor = None;
            }
            Screen::IssueList => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::event::{KeyResult, handle_key, handle_modal_key};
    use crossterm::event::{KeyCode, KeyModifiers};
    use hjkl_engine::VimMode;

    fn sample_issues() -> Vec<crate::db::models::Issue> {
        vec![
            crate::db::models::Issue {
                issue_id: 1,
                name: Some("First".to_string()),
                description: "desc".to_string(),
                author: "Alice".to_string(),
                created_at: "2025-01-01 00:00:00".to_string(),
                updated_at: "2025-01-01 00:00:00".to_string(),
                remote_id: None,
            },
            crate::db::models::Issue {
                issue_id: 2,
                name: Some("Second".to_string()),
                description: "desc".to_string(),
                author: "Bob".to_string(),
                created_at: "2025-01-01 00:00:00".to_string(),
                updated_at: "2025-01-01 00:00:00".to_string(),
                remote_id: None,
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
        assert!(matches!(app.focus, Focus::Thread));
        assert!(app.input_editor.is_none());
    }

    #[test]
    fn back_clears_input_editor_and_focus() {
        let mut app = App::new("test".to_string(), None);
        app.issues = sample_issues();
        app.select_issue();
        app.focus_input_box();
        let editor = app.input_editor.as_mut().expect("editor");
        editor.set_text("partial comment");
        app.back();
        assert!(matches!(app.focus, Focus::Thread));
        assert!(app.input_editor.is_none());
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
        assert!(matches!(
            handle_key(&mut app, KeyCode::Char('q'), KeyModifiers::NONE),
            KeyResult::Quit
        ));
    }

    #[test]
    fn key_esc_quits_from_issue_list() {
        let mut app = App::default();
        assert!(matches!(
            handle_key(&mut app, KeyCode::Esc, KeyModifiers::NONE),
            KeyResult::Quit
        ));
    }

    #[test]
    fn key_j_moves_down() {
        let mut app = App::default();
        app.issues = sample_issues();
        assert!(matches!(
            handle_key(&mut app, KeyCode::Char('j'), KeyModifiers::NONE),
            KeyResult::Continue
        ));
        assert_eq!(app.selected_issue, 1);
    }

    #[test]
    fn key_k_moves_up() {
        let mut app = App::default();
        app.issues = sample_issues();
        app.selected_issue = 1;
        assert!(matches!(
            handle_key(&mut app, KeyCode::Char('k'), KeyModifiers::NONE),
            KeyResult::Continue
        ));
        assert_eq!(app.selected_issue, 0);
    }

    #[test]
    fn key_enter_selects_issue() {
        let mut app = App::default();
        app.issues = sample_issues();
        assert!(matches!(
            handle_key(&mut app, KeyCode::Enter, KeyModifiers::NONE),
            KeyResult::Continue
        ));
        assert!(matches!(app.screen, Screen::Thread));
    }

    #[test]
    fn key_h_back_from_thread() {
        let mut app = App::default();
        app.issues = sample_issues();
        app.screen = Screen::Thread;
        assert!(matches!(
            handle_key(&mut app, KeyCode::Char('h'), KeyModifiers::NONE),
            KeyResult::Continue
        ));
        assert!(matches!(app.screen, Screen::IssueList));
    }

    #[test]
    fn key_q_from_thread_returns_to_issue_list() {
        let mut app = App::default();
        app.issues = sample_issues();
        app.screen = Screen::Thread;
        assert!(matches!(
            handle_key(&mut app, KeyCode::Char('q'), KeyModifiers::NONE),
            KeyResult::Continue
        ));
        assert!(matches!(app.screen, Screen::IssueList));
    }

    #[test]
    fn key_j_scrolls_thread_down() {
        let mut app = App::default();
        app.screen = Screen::Thread;
        assert!(matches!(
            handle_key(&mut app, KeyCode::Char('j'), KeyModifiers::NONE),
            KeyResult::Continue
        ));
        assert_eq!(app.thread_scroll, 1);
    }

    #[test]
    fn key_k_scrolls_thread_up() {
        let mut app = App::default();
        app.screen = Screen::Thread;
        app.thread_scroll = 5;
        assert!(matches!(
            handle_key(&mut app, KeyCode::Char('k'), KeyModifiers::NONE),
            KeyResult::Continue
        ));
        assert_eq!(app.thread_scroll, 4);
    }

    #[test]
    fn ctrl_u_pages_up() {
        let mut app = App::default();
        app.screen = Screen::Thread;
        app.thread_scroll = 25;
        assert!(matches!(
            handle_key(&mut app, KeyCode::Char('u'), KeyModifiers::CONTROL),
            KeyResult::Continue
        ));
        assert_eq!(app.thread_scroll, 5);
    }

    #[test]
    fn ctrl_d_pages_down() {
        let mut app = App::new("test".to_string(), None);
        app.screen = Screen::Thread;
        assert!(matches!(
            handle_key(&mut app, KeyCode::Char('d'), KeyModifiers::CONTROL),
            KeyResult::Continue
        ));
        assert_eq!(app.thread_scroll, 20);
    }

    #[test]
    fn ctrl_w_j_focuses_input_box() {
        let mut app = App::new("test".to_string(), None);
        app.screen = Screen::Thread;
        assert!(matches!(
            handle_key(&mut app, KeyCode::Char('w'), KeyModifiers::CONTROL),
            KeyResult::Continue
        ));
        assert!(app.ctrl_w_pending);
        assert!(matches!(
            handle_key(&mut app, KeyCode::Char('j'), KeyModifiers::NONE),
            KeyResult::Continue
        ));
        assert!(!app.ctrl_w_pending);
        assert!(matches!(app.focus, Focus::InputBox));
        assert!(app.input_editor.is_some());
    }

    #[test]
    fn ctrl_w_k_focuses_thread() {
        let mut app = App::new("test".to_string(), None);
        app.screen = Screen::Thread;
        app.focus_input_box();
        assert!(matches!(app.focus, Focus::InputBox));
        assert!(matches!(
            handle_key(&mut app, KeyCode::Char('w'), KeyModifiers::CONTROL),
            KeyResult::Continue
        ));
        assert!(matches!(
            handle_key(&mut app, KeyCode::Char('k'), KeyModifiers::NONE),
            KeyResult::Continue
        ));
        assert!(!app.ctrl_w_pending);
        assert!(matches!(app.focus, Focus::Thread));
    }

    #[test]
    fn ctrl_w_other_cancels_prefix() {
        let mut app = App::new("test".to_string(), None);
        app.screen = Screen::Thread;
        handle_key(&mut app, KeyCode::Char('w'), KeyModifiers::CONTROL);
        assert!(app.ctrl_w_pending);
        assert!(matches!(
            handle_key(&mut app, KeyCode::Char('x'), KeyModifiers::NONE),
            KeyResult::Continue
        ));
        assert!(!app.ctrl_w_pending);
        assert!(matches!(app.focus, Focus::Thread));
    }

    #[test]
    fn input_box_typing_inserts_text() {
        let mut app = App::new("test".to_string(), None);
        app.screen = Screen::Thread;
        app.focus_input_box();
        assert!(matches!(
            handle_key(&mut app, KeyCode::Char('h'), KeyModifiers::NONE),
            KeyResult::Continue
        ));
        let editor = app.input_editor.as_ref().expect("editor");
        assert_eq!(editor.text(), "h");
    }

    #[test]
    fn input_box_enter_in_normal_mode_with_text_submits() {
        let mut app = App::new("test".to_string(), None);
        app.screen = Screen::Thread;
        app.focus_input_box();
        let editor = app.input_editor.as_mut().expect("editor");
        editor.set_text("hello");
        editor.enter_normal();
        assert!(matches!(
            handle_key(&mut app, KeyCode::Enter, KeyModifiers::NONE),
            KeyResult::SubmitComment
        ));
    }

    #[test]
    fn input_box_enter_in_normal_mode_empty_is_noop() {
        let mut app = App::new("test".to_string(), None);
        app.screen = Screen::Thread;
        app.focus_input_box();
        let editor = app.input_editor.as_mut().expect("editor");
        editor.enter_normal();
        assert!(matches!(
            handle_key(&mut app, KeyCode::Enter, KeyModifiers::NONE),
            KeyResult::Continue
        ));
    }

    #[test]
    fn input_box_enter_in_insert_mode_inserts_newline() {
        let mut app = App::new("test".to_string(), None);
        app.screen = Screen::Thread;
        app.focus_input_box();
        assert!(matches!(
            app.input_editor.as_ref().unwrap().vim_mode(),
            VimMode::Insert
        ));
        assert!(matches!(
            handle_key(&mut app, KeyCode::Enter, KeyModifiers::NONE),
            KeyResult::Continue
        ));
        let editor = app.input_editor.as_ref().expect("editor");
        assert_eq!(editor.text(), "\n");
    }

    #[test]
    fn input_box_esc_in_insert_transitions_to_normal() {
        let mut app = App::new("test".to_string(), None);
        app.screen = Screen::Thread;
        app.focus_input_box();
        assert!(matches!(
            app.input_editor.as_ref().unwrap().vim_mode(),
            VimMode::Insert
        ));
        assert!(matches!(
            handle_key(&mut app, KeyCode::Esc, KeyModifiers::NONE),
            KeyResult::Continue
        ));
        assert!(matches!(
            app.input_editor.as_ref().unwrap().vim_mode(),
            VimMode::Normal
        ));
        assert!(matches!(app.focus, Focus::InputBox));
    }

    #[test]
    fn input_box_esc_in_normal_returns_focus_to_thread() {
        let mut app = App::new("test".to_string(), None);
        app.screen = Screen::Thread;
        app.focus_input_box();
        let editor = app.input_editor.as_mut().unwrap();
        editor.enter_normal();
        assert!(matches!(
            handle_key(&mut app, KeyCode::Esc, KeyModifiers::NONE),
            KeyResult::Continue
        ));
        assert!(matches!(app.focus, Focus::Thread));
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
        assert!(matches!(
            handle_key(&mut app, KeyCode::Char('n'), KeyModifiers::NONE),
            KeyResult::Continue
        ));
        assert!(matches!(app.modal, Some(ModalState::NameInput)));
    }

    #[test]
    fn modal_key_typing_appends_to_buffer() {
        let mut app = App::new("test".to_string(), None);
        app.start_name_input();
        assert!(!handle_modal_key(
            &mut app,
            KeyCode::Char('a'),
            KeyModifiers::NONE
        ));
        assert_eq!(app.input_buffer, "a");
        assert!(!handle_modal_key(
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
        assert!(!handle_modal_key(
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
        assert!(!handle_modal_key(
            &mut app,
            KeyCode::Esc,
            KeyModifiers::NONE
        ));
        assert!(app.modal.is_none());
        assert!(app.input_buffer.is_empty());
    }

    #[test]
    fn modal_enter_triggers_editor_flow() {
        let mut app = App::new("test".to_string(), None);
        app.start_name_input();
        app.input_buffer = "test issue".to_string();
        let quit = handle_modal_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
        assert!(!quit);
    }

    #[test]
    fn error_modal_esc_dismisses() {
        let mut app = App::new("test".to_string(), None);
        app.show_error("fail");
        assert!(!handle_modal_key(
            &mut app,
            KeyCode::Esc,
            KeyModifiers::NONE
        ));
        assert!(app.modal.is_none());
    }

    #[test]
    fn error_modal_enter_dismisses() {
        let mut app = App::new("test".to_string(), None);
        app.show_error("fail");
        assert!(!handle_modal_key(
            &mut app,
            KeyCode::Enter,
            KeyModifiers::NONE
        ));
        assert!(app.modal.is_none());
    }
}
