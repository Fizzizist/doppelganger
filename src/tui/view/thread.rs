use crate::tui::app::App;
use crate::tui::highlight::build_renderer;
use crate::tui::input_box::{input_box_height, max_input_box_height, render_input_box};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

fn markdown_theme() -> the_other_tui_markdown::Theme {
    the_other_tui_markdown::Theme {
        code_block: Style::new().fg(Color::Gray),
        code_block_lang: Style::new()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
        inline_code: Style::default().fg(Color::Gray),
        ..the_other_tui_markdown::Theme::default()
    }
}

pub fn render(f: &mut ratatui::Frame, app: &mut App) {
    let total_height = f.area().height;
    let input_height = match &app.input_editor {
        Some(editor) => input_box_height(editor, max_input_box_height(total_height)),
        None => 3,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(input_height),
        ])
        .split(f.area());

    {
        let thread = match &app.thread {
            Some(t) => t,
            None => return,
        };

        let header = Paragraph::new(Line::from(vec![
            Span::styled(
                format!("#{} ", thread.issue_id),
                Style::default().add_modifier(Modifier::BOLD),
            ),
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

        let renderer = build_renderer(markdown_theme());
        let mut body_lines: Vec<Line> = Vec::new();

        let desc_text =
            the_other_tui_markdown::into_text_with_renderer(&thread.description, &renderer);
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
            let comment_text =
                the_other_tui_markdown::into_text_with_renderer(&comment.content, &renderer);
            for line in comment_text.lines {
                body_lines.push(line);
            }
        }

        let body = Paragraph::new(body_lines)
            .scroll((app.thread_scroll, 0))
            .wrap(Wrap { trim: false });
        f.render_widget(body, chunks[1]);
    }

    render_input_box(f, app, chunks[2]);
}
