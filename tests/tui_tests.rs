mod common;

use doppelganger::db::models::{
    Branch, BranchComment, BranchWithComments, Issue, IssueComment, IssueWithComments,
};
use doppelganger::tui::app::{App, Screen};
use doppelganger::tui::model::Thread;
use doppelganger::tui::view;

fn make_issue(id: i64, name: Option<&str>, desc: &str, author: &str) -> Issue {
    Issue {
        issue_id: id,
        name: name.map(|s| s.to_string()),
        description: desc.to_string(),
        author: author.to_string(),
        created_at: "2025-01-01 00:00:00".to_string(),
        updated_at: "2025-01-02 12:00:00".to_string(),
        remote_id: None,
        archived_at: None,
    }
}

fn buffer_to_string(buffer: &ratatui::buffer::Buffer) -> String {
    let mut s = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            let cell = buffer.cell((x, y)).expect("cell");
            s.push_str(cell.symbol());
        }
        s.push('\n');
    }
    s
}

fn render_to_string(
    width: u16,
    height: u16,
    render_fn: impl FnOnce(&mut ratatui::Frame),
) -> String {
    let backend = ratatui::backend::TestBackend::new(width, height);
    let mut terminal = ratatui::Terminal::new(backend).expect("terminal");
    terminal.draw(render_fn).expect("draw");
    let buffer = terminal.backend().buffer().clone();
    buffer_to_string(&buffer)
}

#[test]
fn issue_list_render_with_items() {
    let mut app = App::default();
    app.issues = vec![
        make_issue(1, Some("Fix login bug"), "Users can't log in", "Alice"),
        make_issue(2, Some("Add dark mode"), "Implement dark theme", "Bob"),
        make_issue(
            3,
            None,
            "A very long description that exceeds the normal display width for testing truncation behavior",
            "Charlie",
        ),
    ];

    let output = render_to_string(80, 24, |f| view::issue_list::render(f, &app));
    insta::assert_snapshot!("issue_list_3_items", output);
}

#[test]
fn issue_list_render_empty() {
    let app = App::default();

    let output = render_to_string(80, 24, |f| view::issue_list::render(f, &app));
    insta::assert_snapshot!("issue_list_empty", output);
}

#[test]
fn thread_render_issue() {
    let issue = make_issue(
        1,
        Some("Fix login bug"),
        "Users can't **log in** on mobile.\n\nSteps:\n1. Open app\n2. Tap login",
        "Alice",
    );
    let comments = vec![
        IssueComment {
            issue_comment_id: 1,
            content: "I can reproduce this".to_string(),
            author: "Bob".to_string(),
            issue_id: 1,
            created_at: "2025-01-01 10:00:00".to_string(),
            updated_at: "2025-01-01 10:00:00".to_string(),
        },
        IssueComment {
            issue_comment_id: 2,
            content: "Fix is in progress".to_string(),
            author: "Charlie".to_string(),
            issue_id: 1,
            created_at: "2025-01-01 11:00:00".to_string(),
            updated_at: "2025-01-01 11:00:00".to_string(),
        },
    ];
    let thread = Thread::from(&IssueWithComments { issue, comments });

    let mut app = App::default();
    app.screen = Screen::Thread;
    app.thread = Some(thread);

    let output = render_to_string(80, 24, |f| view::thread::render(f, &mut app));
    insta::assert_snapshot!("thread_issue", output);
}

#[test]
fn thread_render_branch() {
    let branch = Branch {
        branch_id: 1,
        name: "feature/dark-mode".to_string(),
        description: "Implement the **dark theme** toggle.\n\nSee design doc for details."
            .to_string(),
        author: "Alice".to_string(),
        issue_id: 1,
        created_at: "2025-01-01 00:00:00".to_string(),
        updated_at: "2025-01-02 12:00:00".to_string(),
    };
    let comments = vec![BranchComment {
        branch_comment_id: 1,
        content: "Looking good".to_string(),
        author: "Bob".to_string(),
        branch_id: 1,
        created_at: "2025-01-01 10:00:00".to_string(),
        updated_at: "2025-01-01 10:00:00".to_string(),
    }];
    let thread = Thread::from(&BranchWithComments { branch, comments });

    let mut app = App::default();
    app.screen = Screen::Thread;
    app.thread = Some(thread);

    let output = render_to_string(80, 24, |f| view::thread::render(f, &mut app));
    insta::assert_snapshot!("thread_branch", output);
}

#[test]
fn thread_render_scroll() {
    let issue = make_issue(
        1,
        Some("Scrolled"),
        "First line\nSecond line\nThird line\nFourth line",
        "Alice",
    );
    let thread = Thread::from(&IssueWithComments {
        issue,
        comments: Vec::new(),
    });

    let mut app = App::default();
    app.screen = Screen::Thread;
    app.thread = Some(thread);
    app.thread_scroll = 2;

    let output = render_to_string(80, 10, |f| view::thread::render(f, &mut app));
    insta::assert_snapshot!("thread_scroll", output);
}

#[tokio::test]
async fn concurrent_writes_while_polling() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let db_path = tmp.path().join("concurrent.db");
    let db_path_str = db_path.to_str().expect("valid path").to_string();

    let db = doppelganger::db::Database::open(&db_path_str)
        .await
        .expect("open db");
    let conn = db.conn();
    let author = doppelganger::db::author::find_or_create(conn, "Writer", Some("w@b.com"))
        .await
        .expect("create author");
    drop(db);

    let mut handles = Vec::new();
    for i in 0..20 {
        let path = db_path_str.clone();
        let author_id = author.author_id;
        handles.push(tokio::spawn(async move {
            let db = doppelganger::db::Database::open(&path)
                .await
                .expect("open for write");
            doppelganger::db::issue::create(
                db.conn(),
                None,
                &format!("concurrent issue {i}"),
                author_id,
                None,
            )
            .await
            .expect("create issue");
            db.checkpoint().await.expect("checkpoint");
        }));
    }

    for handle in handles {
        handle.await.expect("write task panicked");
    }

    let db = doppelganger::db::Database::open(&db_path_str)
        .await
        .expect("open for verify");
    let issues = doppelganger::db::issue::list(db.conn())
        .await
        .expect("list");
    assert_eq!(issues.len(), 20, "all 20 concurrent writes should succeed");
}

use common::TestRepo;

#[tokio::test]
async fn branch_tui_no_branch_record() {
    let repo = TestRepo::new_with_commit();
    let output = repo
        .dg_command()
        .arg("branch")
        .arg("tui")
        .output()
        .expect("command failed to execute");
    assert!(
        !output.status.success(),
        "branch tui should fail when no branch record exists"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found") || stderr.contains("branch create"),
        "stderr should explain the missing branch, got: {stderr}"
    );
}

#[test]
fn thread_issue_number_display() {
    let issue = make_issue(42, Some("Fix login bug"), "Description here", "Alice");
    let thread = Thread::from(&IssueWithComments {
        issue,
        comments: Vec::new(),
    });

    let mut app = App::default();
    app.screen = Screen::Thread;
    app.thread = Some(thread);

    let output = render_to_string(80, 10, |f| view::thread::render(f, &mut app));
    assert!(
        output.contains("#42"),
        "header should contain #42, got:\n{output}"
    );
    insta::assert_snapshot!("thread_issue_number_display", output);

    // Verify the issue number is rendered in bold
    let backend = ratatui::backend::TestBackend::new(80, 10);
    let mut terminal = ratatui::Terminal::new(backend).expect("terminal");
    terminal
        .draw(|f| view::thread::render(f, &mut app))
        .expect("draw");
    let buffer = terminal.backend().buffer().clone();
    let hash_cell = buffer.cell((0, 0)).expect("cell at #");
    assert!(
        hash_cell
            .style()
            .add_modifier
            .contains(ratatui::style::Modifier::BOLD),
        "issue number should be bold, got style: {:?}",
        hash_cell.style(),
    );
}

#[test]
fn issue_list_with_name_input_modal() {
    let mut app = App::default();
    app.issues = vec![
        make_issue(1, Some("First"), "desc", "Alice"),
        make_issue(2, Some("Second"), "desc", "Bob"),
    ];
    app.start_name_input();
    app.input_buffer = "My Bug Report".to_string();

    let output = render_to_string(80, 24, |f| {
        view::issue_list::render(f, &app);
        view::modal::render(f, &app);
    });
    insta::assert_snapshot!("issue_list_with_name_input_modal", output);
}

#[test]
fn issue_list_with_empty_name_input_modal() {
    let mut app = App::default();
    app.issues = vec![make_issue(1, Some("First"), "desc", "Alice")];
    app.start_name_input();

    let output = render_to_string(80, 24, |f| {
        view::issue_list::render(f, &app);
        view::modal::render(f, &app);
    });
    insta::assert_snapshot!("issue_list_with_empty_name_input_modal", output);
}

#[test]
fn issue_list_with_error_modal() {
    let mut app = App::default();
    app.issues = vec![make_issue(1, Some("First"), "desc", "Alice")];
    app.show_error("editor exited with non-zero status; assuming cancel");

    let output = render_to_string(80, 24, |f| {
        view::issue_list::render(f, &app);
        view::modal::render(f, &app);
    });
    insta::assert_snapshot!("issue_list_with_error_modal", output);
}

#[test]
fn thread_input_box_focused() {
    let issue = make_issue(1, Some("Test issue"), "Description", "Alice");
    let thread = Thread::from(&IssueWithComments {
        issue,
        comments: Vec::new(),
    });

    let mut app = App::default();
    app.screen = Screen::Thread;
    app.thread = Some(thread);
    app.focus_input_box();

    let output = render_to_string(80, 24, |f| view::thread::render(f, &mut app));
    insta::assert_snapshot!("thread_input_box_focused", output);

    let backend = ratatui::backend::TestBackend::new(80, 24);
    let mut terminal = ratatui::Terminal::new(backend).expect("terminal");
    terminal
        .draw(|f| view::thread::render(f, &mut app))
        .expect("draw");
    let buffer = terminal.backend().buffer().clone();
    let border_cell = buffer.cell((0, 21)).expect("cell at border");
    assert!(
        border_cell.style().fg == Some(ratatui::style::Color::Cyan),
        "focused input box border should be cyan, got: {:?}",
        border_cell.style(),
    );
}

#[test]
fn thread_input_box_with_text() {
    let issue = make_issue(1, Some("Test issue"), "Description", "Alice");
    let thread = Thread::from(&IssueWithComments {
        issue,
        comments: Vec::new(),
    });

    let mut app = App::default();
    app.screen = Screen::Thread;
    app.thread = Some(thread);
    app.focus_input_box();
    let editor = app.input_editor.as_mut().expect("editor");
    editor.set_text("Hello world");

    let output = render_to_string(80, 24, |f| view::thread::render(f, &mut app));
    assert!(
        output.contains("Hello world"),
        "input box should contain typed text, got:\n{output}"
    );
    insta::assert_snapshot!("thread_input_box_with_text", output);
}

#[test]
fn thread_input_box_multiline_text() {
    let issue = make_issue(1, Some("Test issue"), "Description", "Alice");
    let thread = Thread::from(&IssueWithComments {
        issue,
        comments: Vec::new(),
    });

    let mut app = App::default();
    app.screen = Screen::Thread;
    app.thread = Some(thread);
    app.focus_input_box();
    let editor = app.input_editor.as_mut().expect("editor");
    editor.set_text("Line one\nLine two\nLine three");

    let output = render_to_string(80, 24, |f| view::thread::render(f, &mut app));
    assert!(
        output.contains("Line one"),
        "input box should contain first line, got:\n{output}"
    );
    assert!(
        output.contains("Line two"),
        "input box should contain second line, got:\n{output}"
    );
    insta::assert_snapshot!("thread_input_box_multiline_text", output);
}

#[test]
fn thread_input_box_word_wrap() {
    let issue = make_issue(1, Some("Test issue"), "Description", "Alice");
    let thread = Thread::from(&IssueWithComments {
        issue,
        comments: Vec::new(),
    });

    let mut app = App::default();
    app.screen = Screen::Thread;
    app.thread = Some(thread);
    app.focus_input_box();
    let editor = app.input_editor.as_mut().expect("editor");
    editor.set_text("abcdefghijklmnopqrstuvwxyz");

    let output = render_to_string(20, 24, |f| view::thread::render(f, &mut app));
    insta::assert_snapshot!("thread_input_box_word_wrap", output);
}

#[test]
fn thread_selection_on_description() {
    let issue = make_issue(1, Some("Test issue"), "Description text", "Alice");
    let comments = vec![IssueComment {
        issue_comment_id: 1,
        content: "A comment".to_string(),
        author: "Bob".to_string(),
        issue_id: 1,
        created_at: "2025-01-01 10:00:00".to_string(),
        updated_at: "2025-01-01 10:00:00".to_string(),
    }];
    let thread = Thread::from(&IssueWithComments { issue, comments });

    let mut app = App::default();
    app.screen = Screen::Thread;
    app.thread = Some(thread);
    app.thread_selected = 0;

    let output = render_to_string(80, 24, |f| view::thread::render(f, &mut app));
    insta::assert_snapshot!("thread_selection_on_description", output);
}

#[test]
fn thread_selection_on_comment() {
    let issue = make_issue(1, Some("Test issue"), "Description text", "Alice");
    let comments = vec![IssueComment {
        issue_comment_id: 1,
        content: "A comment".to_string(),
        author: "Bob".to_string(),
        issue_id: 1,
        created_at: "2025-01-01 10:00:00".to_string(),
        updated_at: "2025-01-01 10:00:00".to_string(),
    }];
    let thread = Thread::from(&IssueWithComments { issue, comments });

    let mut app = App::default();
    app.screen = Screen::Thread;
    app.thread = Some(thread);
    app.thread_selected = 1;

    let output = render_to_string(80, 24, |f| view::thread::render(f, &mut app));
    insta::assert_snapshot!("thread_selection_on_comment", output);
}

#[test]
fn thread_selection_not_shown_in_input_focus() {
    let issue = make_issue(1, Some("Test issue"), "Description text", "Alice");
    let comments = vec![IssueComment {
        issue_comment_id: 1,
        content: "A comment".to_string(),
        author: "Bob".to_string(),
        issue_id: 1,
        created_at: "2025-01-01 10:00:00".to_string(),
        updated_at: "2025-01-01 10:00:00".to_string(),
    }];
    let thread = Thread::from(&IssueWithComments { issue, comments });

    let mut app = App::default();
    app.screen = Screen::Thread;
    app.thread = Some(thread);
    app.thread_selected = 0;
    app.focus_input_box();

    let output = render_to_string(80, 24, |f| view::thread::render(f, &mut app));
    insta::assert_snapshot!("thread_selection_not_shown_in_input_focus", output);
}
