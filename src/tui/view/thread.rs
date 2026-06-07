use crate::tui::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn render(f: &mut ratatui::Frame, app: &App) {
    let thread = match &app.thread {
        Some(t) => t,
        None => return,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            thread.title.clone(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(thread.author.clone(), Style::default().fg(Color::Yellow)),
        Span::raw("  "),
        Span::styled(
            thread.updated_at.clone(),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    let mut body_lines: Vec<Line> = Vec::new();

    let desc_text = the_other_tui_markdown::into_text(&thread.description);
    for line in desc_text.lines {
        body_lines.push(line);
    }

    for comment in &thread.comments {
        body_lines.push(Line::from(""));
        body_lines.push(Line::from(vec![
            Span::styled(comment.author.clone(), Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled(
                comment.created_at.clone(),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        let comment_text = the_other_tui_markdown::into_text(&comment.content);
        for line in comment_text.lines {
            body_lines.push(line);
        }
    }

    let body = Paragraph::new(body_lines)
        .scroll((app.thread_scroll, 0))
        .wrap(Wrap { trim: false });
    f.render_widget(body, chunks[1]);

    let help = Paragraph::new("j/k: scroll  Ctrl+u/Ctrl+d: page  h/q/Esc: back")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[2]);
}
