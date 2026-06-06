use doppelganger::tui::thread_view::{ThreadViewScreen, render_thread_view_snapshot};
use doppelganger::tui::{Thread, ThreadComment};
use ratatui::layout::Rect;

fn make_thread() -> Thread {
    Thread {
        thread_id: 1,
        title: "Bug report".to_string(),
        description: "This is a **bug** that needs fixing.\n\nSteps to reproduce:\n\n1. Do thing\n2. See error".to_string(),
        author: "Alice".to_string(),
        created_at: "2025-01-01 00:00:00".to_string(),
        updated_at: "2025-01-02 00:00:00".to_string(),
    }
}

fn make_comments() -> Vec<ThreadComment> {
    vec![
        ThreadComment {
            comment_id: 1,
            thread_id: 1,
            content: "I can reproduce this.".to_string(),
            author: "Bob".to_string(),
            created_at: "2025-01-01 12:00:00".to_string(),
            updated_at: "2025-01-01 12:00:00".to_string(),
        },
        ThreadComment {
            comment_id: 2,
            thread_id: 1,
            content: "Fixed in #42.".to_string(),
            author: "Carol".to_string(),
            created_at: "2025-01-02 12:00:00".to_string(),
            updated_at: "2025-01-02 12:00:00".to_string(),
        },
    ]
}

#[test]
fn thread_view_issue_render() {
    let thread = make_thread();
    let comments = make_comments();
    let area = Rect::new(0, 0, 80, 30);
    let buffer = render_thread_view_snapshot(thread, comments, area);
    insta::assert_debug_snapshot!(buffer);
}

#[test]
fn thread_view_branch_render() {
    let thread = Thread {
        thread_id: 5,
        title: "feature-branch".to_string(),
        description:
            "Working on the feature.\n\n| Status | Progress |\n|--------|----------|\n| Dev | 50% |"
                .to_string(),
        author: "Dave".to_string(),
        created_at: "2025-02-01 00:00:00".to_string(),
        updated_at: "2025-02-01 00:00:00".to_string(),
    };
    let comments = vec![ThreadComment {
        comment_id: 10,
        thread_id: 5,
        content: "Almost done.".to_string(),
        author: "Eve".to_string(),
        created_at: "2025-02-02 00:00:00".to_string(),
        updated_at: "2025-02-02 00:00:00".to_string(),
    }];
    let area = Rect::new(0, 0, 80, 30);
    let buffer = render_thread_view_snapshot(thread, comments, area);
    insta::assert_debug_snapshot!(buffer);
}

#[test]
fn thread_view_scroll_clamps() {
    let thread = Thread {
        thread_id: 1,
        title: "Long thread".to_string(),
        description: "A thread with many comments.".to_string(),
        author: "Alice".to_string(),
        created_at: "2025-01-01 00:00:00".to_string(),
        updated_at: "2025-01-02 00:00:00".to_string(),
    };
    let comments: Vec<ThreadComment> = (0..50)
        .map(|i| ThreadComment {
            comment_id: i,
            thread_id: 1,
            content: format!("Comment number {i} with some text to fill the screen."),
            author: "User".to_string(),
            created_at: "2025-01-01 00:00:00".to_string(),
            updated_at: "2025-01-01 00:00:00".to_string(),
        })
        .collect();
    let mut screen = ThreadViewScreen::new(thread, comments);

    let area = Rect::new(0, 0, 80, 24);
    let backend = ratatui::backend::TestBackend::new(area.width, area.height);
    let mut terminal = ratatui::Terminal::new(backend).expect("create test terminal");
    terminal
        .draw(|frame| screen.render(frame, frame.area()))
        .expect("draw");

    assert!(screen.max_scroll > 0, "content should exceed viewport");
    assert_eq!(screen.scroll, 0);
    screen.scroll_down(5);
    assert_eq!(screen.scroll, 5);
    screen.scroll_up(3);
    assert_eq!(screen.scroll, 2);
    screen.scroll_up(10);
    assert_eq!(screen.scroll, 0);
}
