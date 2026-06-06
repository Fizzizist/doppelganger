use crate::tui::{Thread, ThreadComment, markdown};
use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

pub struct ThreadViewScreen {
    thread: Thread,
    comments: Vec<ThreadComment>,
    pub scroll: u16,
    pub max_scroll: u16,
}

impl ThreadViewScreen {
    pub fn new(thread: Thread, comments: Vec<ThreadComment>) -> Self {
        Self {
            thread,
            comments,
            scroll: 0,
            max_scroll: 0,
        }
    }

    pub fn scroll_down(&mut self, amount: u16) {
        self.scroll = self.scroll.saturating_add(amount).min(self.max_scroll);
    }

    pub fn scroll_up(&mut self, amount: u16) {
        self.scroll = self.scroll.saturating_sub(amount);
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let inner = area.inner(Margin::new(1, 1));
        let scroll = self.scroll;

        let title_line = Line::from(vec![
            Span::styled(self.thread.title.clone(), Style::default().bold()),
            Span::raw("  "),
            Span::styled(format!("by {}", self.thread.author), Style::default().dim()),
        ]);

        let mut content_lines: Vec<Line<'static>> = vec![title_line, Line::default()];

        append_markdown_blocks(&self.thread.description, &mut content_lines);

        for comment in &self.comments {
            let author_line = Line::from(vec![
                Span::styled(comment.author.clone(), Style::default().bold()),
                Span::raw("  "),
                Span::styled(comment.created_at.clone(), Style::default().dim()),
            ]);
            content_lines.push(author_line);

            append_markdown_blocks(&comment.content, &mut content_lines);
            content_lines.push(Line::default());
        }

        let total_height = content_lines.len() as u16;
        self.max_scroll = total_height.saturating_sub(inner.height);

        let title = self.thread.title.clone();
        let paragraph = Paragraph::new(content_lines)
            .block(Block::default().borders(Borders::ALL).title(title))
            .scroll((scroll, 0));

        frame.render_widget(paragraph, area);

        if total_height > inner.height {
            let mut scrollbar_state =
                ScrollbarState::new(total_height as usize).position(scroll as usize);
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight),
                area.inner(Margin::new(0, 1)),
                &mut scrollbar_state,
            );
        }
    }
}

fn append_markdown_blocks(markdown_text: &str, lines: &mut Vec<Line<'static>>) {
    let blocks = markdown::parse_markdown(markdown_text);
    for block in &blocks {
        match block {
            markdown::RenderBlock::Prose(md_str) => {
                let rendered = markdown::render_prose(md_str);
                for line in rendered.lines {
                    lines.push(line);
                }
                lines.push(Line::default());
            }
            markdown::RenderBlock::Table { header, rows } => {
                render_table_inline(header, rows, lines);
                lines.push(Line::default());
            }
        }
    }
}

fn render_table_inline(header: &[String], rows: &[Vec<String>], lines: &mut Vec<Line<'static>>) {
    let header_row = Line::from(
        header
            .iter()
            .map(|h| Span::styled(h.clone(), Style::default().bold().underlined()))
            .collect::<Vec<_>>(),
    );
    lines.push(header_row);

    for row in rows {
        let line = Line::from(
            row.iter()
                .map(|cell| Span::raw(cell.clone()))
                .collect::<Vec<_>>(),
        );
        lines.push(line);
    }
}

pub fn render_thread_view_snapshot(
    thread: Thread,
    comments: Vec<ThreadComment>,
    area: Rect,
) -> ratatui::buffer::Buffer {
    let mut screen = ThreadViewScreen::new(thread, comments);
    let backend = ratatui::backend::TestBackend::new(area.width, area.height);
    let mut terminal = ratatui::Terminal::new(backend).expect("create test terminal");
    terminal
        .draw(|frame| screen.render(frame, frame.area()))
        .expect("draw");
    terminal.backend().buffer().clone()
}
