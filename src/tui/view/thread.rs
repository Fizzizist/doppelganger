use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::tui::{app::App, markdown::render_markdown, model::Thread};

pub fn render(frame: &mut Frame, app: &App, thread: &Thread) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let title_widget = Paragraph::new(thread.title.clone())
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(title_widget, chunks[0]);

    let mut lines: Vec<Line<'static>> = Vec::new();

    lines.push(Line::from(vec![
        Span::styled(
            thread.author.clone(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  ·  "),
        Span::styled(
            thread.created_at.clone(),
            Style::default().fg(Color::DarkGray),
        ),
    ]));

    let desc = render_markdown(&thread.description);
    lines.extend(desc.lines);
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        "─".repeat(60),
        Style::default().fg(Color::DarkGray),
    )));

    for comment in &thread.comments {
        lines.push(Line::from(vec![
            Span::styled(
                comment.author.clone(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  ·  "),
            Span::styled(
                comment.created_at.clone(),
                Style::default().fg(Color::DarkGray),
            ),
        ]));

        let comment_text = render_markdown(&comment.content);
        lines.extend(comment_text.lines);
        lines.push(Line::from(""));
    }

    let hint = if app.has_issue_list {
        "j/k=scroll  Ctrl-u/d=page  h/q=back"
    } else {
        "j/k=scroll  Ctrl-u/d=page  q=quit"
    };

    let content = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Thread  ({})", hint)),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.scroll, 0));

    frame.render_widget(content, chunks[1]);
}
