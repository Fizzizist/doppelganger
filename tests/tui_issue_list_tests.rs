use doppelganger::db::models::Issue;
use doppelganger::tui::issue_list::{IssueListScreen, render_issue_list_snapshot};
use ratatui::layout::Rect;

fn make_issues() -> Vec<Issue> {
    vec![
        Issue {
            issue_id: 1,
            name: Some("First issue".to_string()),
            description: "Desc 1".to_string(),
            author: "Alice".to_string(),
            created_at: "2025-01-01 00:00:00".to_string(),
            updated_at: "2025-01-03 00:00:00".to_string(),
        },
        Issue {
            issue_id: 2,
            name: Some("Second issue".to_string()),
            description: "Desc 2".to_string(),
            author: "Bob".to_string(),
            created_at: "2025-01-02 00:00:00".to_string(),
            updated_at: "2025-01-02 00:00:00".to_string(),
        },
        Issue {
            issue_id: 3,
            name: None,
            description: "Desc 3".to_string(),
            author: "Carol".to_string(),
            created_at: "2025-01-01 00:00:00".to_string(),
            updated_at: "2025-01-01 00:00:00".to_string(),
        },
    ]
}

#[test]
fn issue_list_renders() {
    let issues = make_issues();
    let area = Rect::new(0, 0, 80, 20);
    let buffer = render_issue_list_snapshot(issues, area);
    insta::assert_debug_snapshot!(buffer);
}

#[test]
fn issue_list_navigation() {
    let issues = make_issues();
    let mut screen = IssueListScreen::new(issues);

    assert_eq!(screen.selected_issue_id(), Some(1));
    screen.select_down();
    assert_eq!(screen.selected_issue_id(), Some(2));
    screen.select_down();
    assert_eq!(screen.selected_issue_id(), Some(3));
    screen.select_down();
    assert_eq!(screen.selected_issue_id(), Some(3));
    screen.select_up();
    assert_eq!(screen.selected_issue_id(), Some(2));
    screen.select_up();
    assert_eq!(screen.selected_issue_id(), Some(1));
    screen.select_up();
    assert_eq!(screen.selected_issue_id(), Some(1));
}

#[test]
fn issue_list_empty() {
    let area = Rect::new(0, 0, 80, 20);
    let buffer = render_issue_list_snapshot(vec![], area);
    insta::assert_debug_snapshot!(buffer);
}
