use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListState, Paragraph, Wrap},
};

use crate::tui::app::{App, Screen};
use crate::tui::thread::Thread;

pub fn draw(f: &mut Frame, app: &App) {
    match app.screen {
        Screen::IssueList => draw_issue_list(f, app),
        Screen::Thread => draw_thread(f, app),
    }
}

fn draw_issue_list(f: &mut Frame, app: &App) {
    let items: Vec<Line> = app
        .issues
        .iter()
        .map(|issue| {
            let id = format!("#{}", issue.issue_id);
            let name = issue.name.as_deref().unwrap_or("(unnamed)");
            let updated = &issue.updated_at;
            Line::from(vec![
                Span::styled(format!("{id:>4} "), Style::default().fg(Color::Cyan)),
                Span::styled(format!("{name:<40} "), Style::default()),
                Span::styled(updated.clone(), Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::NONE)
            .title(" Issues ")
            .title_style(Style::default().add_modifier(Modifier::BOLD)),
    );

    let mut state = ListState::default();
    if !app.issues.is_empty() {
        state.select(Some(app.selected));
    }

    f.render_stateful_widget(list, f.area(), &mut state);
}

pub fn draw_thread(f: &mut Frame, app: &App) {
    let thread = match &app.thread {
        Some(t) => t,
        None => return,
    };

    let area = f.area();
    let content = render_thread_content(thread, area);
    let paragraph = Paragraph::new(content).wrap(Wrap { trim: false }).block(
        Block::default()
            .borders(Borders::NONE)
            .title(format!(" {} ", thread.title))
            .title_style(Style::default().add_modifier(Modifier::BOLD)),
    );

    f.render_widget(paragraph.scroll((app.thread_scroll, 0)), area);
}

fn render_thread_content(thread: &Thread, area: Rect) -> Vec<Line<'_>> {
    let width = area.width as usize;
    let mut lines: Vec<Line> = Vec::new();

    let header = format!("{} — {}", thread.author, thread.created_at);
    lines.push(Line::from(Span::styled(
        header,
        Style::default().fg(Color::Yellow),
    )));

    let desc_text = tui_markdown::from_str(&thread.description);
    let mut desc_lines: Vec<Line> = desc_text.into_iter().collect();
    lines.append(&mut desc_lines);

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "─".repeat(width),
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(""));

    for comment in &thread.comments {
        let comment_header = format!("{} — {}", comment.author, comment.created_at);
        lines.push(Line::from(Span::styled(
            comment_header,
            Style::default().fg(Color::Cyan),
        )));

        let comment_text = tui_markdown::from_str(&comment.content);
        let mut comment_lines: Vec<Line> = comment_text.into_iter().collect();
        lines.append(&mut comment_lines);
        lines.push(Line::from(""));
    }

    lines
}
