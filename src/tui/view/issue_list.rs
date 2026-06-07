use crate::tui::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table, TableState},
};

fn display_name(issue: &crate::db::models::Issue) -> &str {
    match issue.name.as_deref() {
        Some(n) if !n.is_empty() => n,
        _ => "untitled",
    }
}

pub fn render(f: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    let header = Row::new(vec![
        Line::from(Span::styled(
            "#",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Name",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Author",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Updated",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ])
    .style(Style::default().fg(Color::Cyan));

    let rows: Vec<Row> = app
        .issues
        .iter()
        .map(|issue| {
            Row::new(vec![
                Line::from(format!("{}", issue.issue_id)),
                Line::from(display_name(issue)),
                Line::from(issue.author.clone()),
                Line::from(issue.updated_at.clone()),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(5),
        Constraint::Min(10),
        Constraint::Min(10),
        Constraint::Length(19),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Issues"))
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    let mut state = TableState::default();
    if !app.issues.is_empty() {
        state.select(Some(app.selected_issue));
    }

    f.render_stateful_widget(table, chunks[0], &mut state);

    let help = Paragraph::new("j/k: navigate  Enter/l: select  q: quit")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[1]);
}
