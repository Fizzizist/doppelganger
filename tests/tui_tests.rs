mod common;

use doppelganger::{
    db::models::Issue,
    tui::thread::{Thread, ThreadComment},
    tui::ui,
    tui::{App, Screen},
};
use ratatui::{Terminal, backend::TestBackend};

fn sample_issue(id: i64, name: Option<&str>, updated_at: &str) -> Issue {
    Issue {
        issue_id: id,
        name: name.map(|s| s.to_string()),
        description: format!("Description for issue {id}"),
        author: "TestAuthor".to_string(),
        created_at: "2025-01-01 00:00:00".to_string(),
        updated_at: updated_at.to_string(),
    }
}

fn sample_thread(title: &str, comment_count: usize) -> Thread {
    let mut comments = Vec::new();
    for i in 0..comment_count {
        comments.push(ThreadComment {
            author: format!("Commenter{i}"),
            content: format!("Comment number {i}"),
            created_at: format!("2025-01-0{} 00:00:00", i + 1),
        });
    }
    Thread {
        title: title.to_string(),
        description: "A **markdown** description with _formatting_".to_string(),
        author: "ThreadAuthor".to_string(),
        created_at: "2025-01-01 00:00:00".to_string(),
        comments,
    }
}

fn buffer_to_string(buffer: &ratatui::buffer::Buffer) -> String {
    let mut lines = Vec::new();
    for y in 0..buffer.area.height {
        let mut line = String::new();
        for x in 0..buffer.area.width {
            let cell = buffer.cell(ratatui::layout::Position { x, y });
            line.push_str(cell.map(|c| c.symbol()).unwrap_or(" "));
        }
        let trimmed = line.trim_end();
        lines.push(trimmed.to_string());
    }
    lines.join("\n")
}

fn render_app(app: &App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("create test terminal");
    terminal.draw(|f| ui::draw(f, app)).expect("draw");
    buffer_to_string(terminal.backend().buffer())
}

#[test]
fn issue_list_empty() {
    let app = App::new(Vec::new());
    let output = render_app(&app, 80, 20);
    insta::assert_snapshot!("issue_list_empty", output);
}

#[test]
fn issue_list_single_item() {
    let issues = vec![sample_issue(1, Some("First issue"), "2025-06-01 12:00:00")];
    let app = App::new(issues);
    let output = render_app(&app, 80, 20);
    insta::assert_snapshot!("issue_list_single", output);
}

#[test]
fn issue_list_many_items() {
    let issues = vec![
        sample_issue(3, Some("Third issue"), "2025-06-03 12:00:00"),
        sample_issue(2, Some("Second issue"), "2025-06-02 12:00:00"),
        sample_issue(1, Some("First issue"), "2025-06-01 12:00:00"),
    ];
    let app = App::new(issues);
    let output = render_app(&app, 80, 20);
    insta::assert_snapshot!("issue_list_many", output);
}

#[test]
fn issue_list_unnamed_issue() {
    let issues = vec![sample_issue(1, None, "2025-06-01 12:00:00")];
    let app = App::new(issues);
    let output = render_app(&app, 80, 20);
    insta::assert_snapshot!("issue_list_unnamed", output);
}

#[test]
fn thread_view_basic() {
    let thread = sample_thread("My Thread", 0);
    let mut app = App::new(Vec::new());
    app.enter_thread(thread);
    assert_eq!(app.screen, Screen::Thread);

    let output = render_app(&app, 80, 20);
    insta::assert_snapshot!("thread_view_basic", output);
}

#[test]
fn thread_view_with_comments() {
    let thread = sample_thread("Discussion", 3);
    let mut app = App::new(Vec::new());
    app.enter_thread(thread);
    let output = render_app(&app, 80, 20);
    insta::assert_snapshot!("thread_view_with_comments", output);
}

#[test]
fn thread_view_scroll() {
    let thread = sample_thread("Long Thread", 5);
    let mut app = App::new(Vec::new());
    app.enter_thread(thread);
    app.scroll_down(5);
    let output = render_app(&app, 80, 20);
    insta::assert_snapshot!("thread_view_scrolled", output);
}

#[test]
fn app_navigation_select_next() {
    let issues = vec![
        sample_issue(1, Some("First"), "2025-06-01 12:00:00"),
        sample_issue(2, Some("Second"), "2025-06-02 12:00:00"),
    ];
    let mut app = App::new(issues);
    assert_eq!(app.selected, 0);
    app.select_next();
    assert_eq!(app.selected, 1);
    app.select_next();
    assert_eq!(app.selected, 1); // can't go past the end
}

#[test]
fn app_navigation_select_prev() {
    let issues = vec![
        sample_issue(1, Some("First"), "2025-06-01 12:00:00"),
        sample_issue(2, Some("Second"), "2025-06-02 12:00:00"),
    ];
    let mut app = App::new(issues);
    app.select_next();
    assert_eq!(app.selected, 1);
    app.select_prev();
    assert_eq!(app.selected, 0);
    app.select_prev();
    assert_eq!(app.selected, 0); // can't go below 0
}

#[test]
fn app_enter_and_exit_thread() {
    let issues = vec![sample_issue(1, Some("First"), "2025-06-01 12:00:00")];
    let mut app = App::new(issues);
    assert_eq!(app.screen, Screen::IssueList);

    let thread = Thread {
        title: "Test".to_string(),
        description: "desc".to_string(),
        author: "A".to_string(),
        created_at: "2025-01-01".to_string(),
        comments: vec![],
    };
    app.enter_thread(thread);
    assert_eq!(app.screen, Screen::Thread);
    assert!(app.thread.is_some());

    app.exit_thread();
    assert_eq!(app.screen, Screen::IssueList);
    assert!(app.thread.is_none());
    assert_eq!(app.thread_scroll, 0);
}

#[test]
fn app_scroll_up_down() {
    let thread = sample_thread("Scroll", 2);
    let mut app = App::new(Vec::new());
    app.enter_thread(thread);
    assert_eq!(app.thread_scroll, 0);

    app.scroll_down(10);
    assert_eq!(app.thread_scroll, 10);

    app.scroll_down(5);
    assert_eq!(app.thread_scroll, 15);

    app.scroll_up(8);
    assert_eq!(app.thread_scroll, 7);

    app.scroll_up(20);
    assert_eq!(app.thread_scroll, 0); // saturating_sub
}
