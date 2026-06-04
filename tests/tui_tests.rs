mod common;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use doppelganger::{
    db::{Database, author, branch, comment, issue, models::*},
    error::Error,
    tui::{app::*, model::*},
};
use turso::Value;

// ---------------------------------------------------------------------------
// list_issues ordering
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_issues_returns_newest_first() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();

    let alice = author::find_or_create(conn, "Alice", Some("a@b.com"))
        .await
        .expect("create author");

    let _a = issue::create(conn, None, "issue a", alice.author_id)
        .await
        .expect("create A");
    let b = issue::create(conn, None, "issue b", alice.author_id)
        .await
        .expect("create B");
    let _c = issue::create(conn, None, "issue c", alice.author_id)
        .await
        .expect("create C");

    // Bump B's updated_at so it is unambiguously newest.
    conn.execute(
        "UPDATE issue SET updated_at = datetime('now', '+1 day') WHERE issue_id = ?1",
        turso::params::Params::Positional(vec![Value::Integer(b.issue_id)]),
    )
    .await
    .expect("update B timestamp");

    let issues = issue::list_issues(conn).await.expect("list issues");
    assert_eq!(issues.len(), 3);
    assert_eq!(issues[0].issue_id, b.issue_id);
}

// ---------------------------------------------------------------------------
// Thread From impls
// ---------------------------------------------------------------------------

#[test]
fn thread_from_issue_with_comments() {
    let issue_with_comments = IssueWithComments {
        issue: Issue {
            issue_id: 1,
            name: Some("my issue".to_string()),
            description: "desc".to_string(),
            author: "alice".to_string(),
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        },
        comments: vec![
            IssueComment {
                issue_comment_id: 10,
                content: "comment one".to_string(),
                author: "bob".to_string(),
                issue_id: 1,
                created_at: "2024-01-02".to_string(),
                updated_at: "2024-01-02".to_string(),
            },
            IssueComment {
                issue_comment_id: 11,
                content: "comment two".to_string(),
                author: "carol".to_string(),
                issue_id: 1,
                created_at: "2024-01-03".to_string(),
                updated_at: "2024-01-03".to_string(),
            },
        ],
    };

    let thread: Thread = issue_with_comments.into();

    assert_eq!(thread.title, "my issue");
    assert_eq!(thread.issue_id, Some(1));
    assert_eq!(thread.branch_id, None);
    assert_eq!(thread.comments.len(), 2);
}

#[test]
fn thread_from_issue_without_name() {
    let issue_with_comments = IssueWithComments {
        issue: Issue {
            issue_id: 5,
            name: None,
            description: "no name".to_string(),
            author: "anon".to_string(),
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        },
        comments: vec![],
    };

    let thread: Thread = issue_with_comments.into();

    assert_eq!(thread.title, "Issue #5");
    assert_eq!(thread.issue_id, Some(5));
}

#[test]
fn thread_from_branch_with_comments() {
    let branch_with_comments = BranchWithComments {
        branch: Branch {
            branch_id: 42,
            name: "feature-x".to_string(),
            description: "work on x".to_string(),
            author: "dev".to_string(),
            issue_id: 1,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        },
        comments: vec![BranchComment {
            branch_comment_id: 99,
            content: "branch comment".to_string(),
            author: "reviewer".to_string(),
            branch_id: 42,
            created_at: "2024-01-02".to_string(),
            updated_at: "2024-01-02".to_string(),
        }],
    };

    let thread: Thread = branch_with_comments.into();

    assert_eq!(thread.title, "feature-x");
    assert_eq!(thread.branch_id, Some(42));
    assert_eq!(thread.issue_id, None);
    assert_eq!(thread.comments.len(), 1);
}

// ---------------------------------------------------------------------------
// Keybinding state transitions
// ---------------------------------------------------------------------------

fn char_key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
}

fn ctrl_key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

#[test]
fn issue_list_j_k_navigation() {
    let issues = vec![
        Issue {
            issue_id: 1,
            name: Some("a".into()),
            description: "".into(),
            author: "x".into(),
            created_at: "".into(),
            updated_at: "".into(),
        },
        Issue {
            issue_id: 2,
            name: Some("b".into()),
            description: "".into(),
            author: "x".into(),
            created_at: "".into(),
            updated_at: "".into(),
        },
        Issue {
            issue_id: 3,
            name: Some("c".into()),
            description: "".into(),
            author: "x".into(),
            created_at: "".into(),
            updated_at: "".into(),
        },
    ];
    let mut app = App::new_issue_list(issues);

    app.handle_key(char_key('j'));
    assert_eq!(app.selected, 1);

    app.handle_key(char_key('j'));
    assert_eq!(app.selected, 2);

    // At end — should not advance.
    app.handle_key(char_key('j'));
    assert_eq!(app.selected, 2);

    app.handle_key(char_key('k'));
    assert_eq!(app.selected, 1);
}

#[test]
fn issue_list_select_action() {
    let issues = vec![
        Issue {
            issue_id: 10,
            name: Some("first".into()),
            description: "".into(),
            author: "x".into(),
            created_at: "".into(),
            updated_at: "".into(),
        },
        Issue {
            issue_id: 20,
            name: Some("second".into()),
            description: "".into(),
            author: "x".into(),
            created_at: "".into(),
            updated_at: "".into(),
        },
    ];
    let mut app = App::new_issue_list(issues);

    app.handle_key(char_key('j'));
    app.handle_key(KeyCode::Enter.into());

    let action = app.take_action();
    match action {
        Some(Action::OpenIssueThread(id)) => assert_eq!(id, 20),
        Some(Action::BackToList) | Some(Action::Quit) => {
            panic!("expected OpenIssueThread(20) but got wrong variant")
        }
        None => panic!("expected Some(action), got None"),
    }
}

#[test]
fn issue_list_quit_action() {
    let issues = vec![Issue {
        issue_id: 1,
        name: Some("a".into()),
        description: "".into(),
        author: "x".into(),
        created_at: "".into(),
        updated_at: "".into(),
    }];
    let mut app = App::new_issue_list(issues);

    app.handle_key(char_key('q'));

    let action = app.take_action();
    match action {
        Some(Action::Quit) => {}
        Some(Action::OpenIssueThread(_)) | Some(Action::BackToList) => {
            panic!("expected Quit, got unexpected variant")
        }
        None => panic!("expected Quit, got None"),
    }
}

#[test]
fn thread_scroll() {
    let thread = Thread {
        title: "t".into(),
        description: "d".into(),
        author: "a".into(),
        created_at: "".into(),
        updated_at: "".into(),
        comments: vec![],
        issue_id: None,
        branch_id: None,
    };
    let mut app = App::new_thread(thread);

    for _ in 0..3 {
        app.handle_key(char_key('j'));
    }
    assert_eq!(app.scroll, 3);

    app.handle_key(char_key('k'));
    assert_eq!(app.scroll, 2);

    app.handle_key(ctrl_key('d'));
    assert_eq!(app.scroll, 12);

    app.handle_key(ctrl_key('u'));
    assert_eq!(app.scroll, 2);

    for _ in 0..5 {
        app.handle_key(char_key('k'));
    }
    assert_eq!(app.scroll, 0);
}

#[test]
fn thread_back_to_list_action() {
    let thread = Thread {
        title: "t".into(),
        description: "d".into(),
        author: "a".into(),
        created_at: "".into(),
        updated_at: "".into(),
        comments: vec![],
        issue_id: None,
        branch_id: None,
    };
    let mut app = App::new_thread(thread);
    app.has_issue_list = true;

    app.handle_key(char_key('h'));

    let action = app.take_action();
    match action {
        Some(Action::BackToList) => {}
        Some(Action::OpenIssueThread(_)) | Some(Action::Quit) => {
            panic!("expected BackToList, got unexpected variant")
        }
        None => panic!("expected BackToList, got None"),
    }
}

#[test]
fn thread_quit_when_no_list() {
    let thread = Thread {
        title: "t".into(),
        description: "d".into(),
        author: "a".into(),
        created_at: "".into(),
        updated_at: "".into(),
        comments: vec![],
        issue_id: None,
        branch_id: None,
    };
    let mut app = App::new_thread(thread);

    app.handle_key(char_key('q'));

    let action = app.take_action();
    match action {
        Some(Action::Quit) => {}
        Some(Action::OpenIssueThread(_)) | Some(Action::BackToList) => {
            panic!("expected Quit, got unexpected variant")
        }
        None => panic!("expected Quit, got None"),
    }
}

// ---------------------------------------------------------------------------
// ratatui TestBackend snapshot tests
// ---------------------------------------------------------------------------

fn buffer_to_string(buffer: &ratatui::buffer::Buffer) -> String {
    let area = buffer.area();
    let mut result = String::new();
    for y in 0..area.height {
        for x in 0..area.width {
            let cell = buffer
                .cell(ratatui::layout::Position { x, y })
                .expect("valid position");
            result.push_str(cell.symbol());
        }
        result.push('\n');
    }
    result
}

fn make_issues(count: usize) -> Vec<Issue> {
    (0..count)
        .map(|i| Issue {
            issue_id: i as i64 + 1,
            name: Some(format!("issue {i}")),
            description: "".into(),
            author: "tester".into(),
            created_at: "2024-01-01".into(),
            updated_at: "2024-01-01".into(),
        })
        .collect()
}

#[test]
fn snapshot_issue_list_render() {
    let issues = make_issues(2);
    let app = App::new_issue_list(issues);

    let backend = ratatui::backend::TestBackend::new(60, 20);
    let mut terminal = ratatui::Terminal::new(backend).expect("create terminal");

    terminal
        .draw(|frame| {
            doppelganger::tui::view::render(frame, &app);
        })
        .expect("draw");

    let backend = terminal.backend();
    let buffer = backend.buffer();
    insta::assert_snapshot!("issue_list_render", buffer_to_string(buffer));
}

#[test]
fn snapshot_thread_render_with_markdown() {
    let thread = Thread {
        title: "test thread".into(),
        description: "**bold**\n# header\n- list item".into(),
        author: "alice".into(),
        created_at: "2024-06-15".into(),
        updated_at: "2024-06-15".into(),
        comments: vec![ThreadComment {
            author: "bob".into(),
            content: "This is a *comment* with `code`.".into(),
            created_at: "2024-06-16".into(),
        }],
        issue_id: Some(1),
        branch_id: None,
    };
    let mut app = App::new_thread(thread);
    app.scroll = 0;

    let backend = ratatui::backend::TestBackend::new(80, 24);
    let mut terminal = ratatui::Terminal::new(backend).expect("create terminal");

    terminal
        .draw(|frame| {
            if let Screen::Thread(t) = &app.screen {
                doppelganger::tui::view::thread::render(frame, &app, t);
            }
        })
        .expect("draw");

    let backend = terminal.backend();
    let buffer = backend.buffer();
    insta::assert_snapshot!("thread_render_with_markdown", buffer_to_string(buffer));
}

#[test]
fn snapshot_thread_render_empty_comments() {
    let thread = Thread {
        title: "no comments".into(),
        description: "just a description".into(),
        author: "solo".into(),
        created_at: "2024-01-01".into(),
        updated_at: "2024-01-01".into(),
        comments: vec![],
        issue_id: Some(42),
        branch_id: None,
    };
    let app = App::new_thread(thread);

    let backend = ratatui::backend::TestBackend::new(80, 24);
    let mut terminal = ratatui::Terminal::new(backend).expect("create terminal");

    terminal
        .draw(|frame| {
            if let Screen::Thread(t) = &app.screen {
                doppelganger::tui::view::thread::render(frame, &app, t);
            }
        })
        .expect("draw");

    let backend = terminal.backend();
    let buffer = backend.buffer();
    insta::assert_snapshot!("thread_render_empty_comments", buffer_to_string(buffer));
}

// ---------------------------------------------------------------------------
// Change-detection test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn change_detection_fingerprint_changes_after_insert() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();

    let author = author::find_or_create(conn, "Alice", Some("a@b.com"))
        .await
        .expect("create author");
    let iss = issue::create(conn, None, "fp issue", author.author_id)
        .await
        .expect("create issue");

    // Fingerprint before: count comments on this issue.
    let mut rows = conn
        .query(
            "SELECT COUNT(issue_comment_id) FROM issue_comment WHERE issue_id = ?1",
            turso::params::Params::Positional(vec![Value::Integer(iss.issue_id)]),
        )
        .await
        .expect("fingerprint query");
    let row = rows.next().await.expect("next row");
    let row = row.expect("row exists");
    let count1: i64 = row.get(0).expect("count");
    assert_eq!(count1, 0, "should have 0 comments initially");

    // Insert a comment.
    comment::create_issue_comment(conn, iss.issue_id, "new comment", author.author_id)
        .await
        .expect("insert comment");

    // Fingerprint after.
    let mut rows = conn
        .query(
            "SELECT COUNT(issue_comment_id) FROM issue_comment WHERE issue_id = ?1",
            turso::params::Params::Positional(vec![Value::Integer(iss.issue_id)]),
        )
        .await
        .expect("fingerprint query");
    let row = rows.next().await.expect("next row");
    let row = row.expect("row exists");
    let count2: i64 = row.get(0).expect("count");
    assert_eq!(count2, 1, "should have 1 comment after insert");
    assert_ne!(
        count1, count2,
        "fingerprint must differ after comment insert"
    );
}

// ---------------------------------------------------------------------------
// Error path tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn branch_tui_missing_branch_returns_error() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();

    let result = branch::get_by_name(conn, "feature-x").await;
    assert!(result.is_err(), "should error for missing branch");
    match result.expect_err("should be error") {
        Error::BranchNotFound(name) => {
            assert_eq!(name, "feature-x");
        }
        other => panic!("expected BranchNotFound, got {other:?}"),
    }
}

#[tokio::test]
async fn issue_not_found_error() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();

    let result = issue::get_by_id(conn, 42).await;
    assert!(result.is_err(), "should error for missing issue");
    match result.expect_err("should be error") {
        Error::IssueNotFound(id) => {
            assert_eq!(id, 42);
        }
        other => panic!("expected IssueNotFound, got {other:?}"),
    }
}

#[tokio::test]
async fn list_issues_on_empty_db() {
    let db = Database::open_in_memory().await.expect("open in-memory db");
    let conn = db.conn();

    let issues = issue::list_issues(conn).await.expect("list issues");
    assert!(issues.is_empty(), "empty db should return no issues");
}
