use crate::tui::app::{App, Focus};
use hjkl_engine::Host;
use hjkl_form::TextFieldEditor;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn input_box_height(editor: &TextFieldEditor, max_lines: u16) -> u16 {
    let line_count = editor.buffer().row_count().max(1) as u16;
    let content_lines = line_count.min(max_lines.saturating_sub(2));
    content_lines.saturating_add(2)
}

pub fn max_input_box_height(total_height: u16) -> u16 {
    total_height.saturating_sub(5).max(3)
}

pub fn render_input_box(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
    let focused = matches!(app.focus, Focus::InputBox);

    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = if focused {
        " Comment (i: insert, Esc: normal, Enter: submit) "
    } else {
        " Comment "
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(title, border_style));

    let inner = block.inner(area);

    let (cursor, placeholder) = match app.input_editor.as_mut() {
        Some(editor) => {
            editor.set_viewport_width(inner.width);
            editor.set_viewport_height(inner.height);

            let text = editor.text();
            let lines: Vec<Line> = if text.is_empty() && !focused {
                vec![placeholder_line()]
            } else {
                text.lines().map(|l| Line::from(l.to_string())).collect()
            };

            let cursor = focused.then(|| cursor_xy(editor, inner));
            let paragraph = Paragraph::new(lines)
                .block(block)
                .wrap(Wrap { trim: false });
            f.render_widget(paragraph, area);

            (cursor, false)
        }
        None => {
            let paragraph = Paragraph::new(vec![placeholder_line()]).block(block);
            f.render_widget(paragraph, area);

            (None, true)
        }
    };

    if placeholder {
        let _ = placeholder;
    }

    if let Some(Some(pos)) = cursor {
        f.set_cursor_position(pos);
    }
}

fn placeholder_line() -> Line<'static> {
    Line::from(Span::styled(
        "Ctrl+W j to comment",
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
    ))
}

fn cursor_xy(editor: &TextFieldEditor, rect: Rect) -> Option<(u16, u16)> {
    let (row, col) = editor.cursor();
    let viewport = editor.editor.host().viewport();

    if row < viewport.top_row || col < viewport.top_col {
        return None;
    }

    let dy = (row - viewport.top_row) as u16;
    let dx = (col - viewport.top_col) as u16;

    if dy >= rect.height || dx >= rect.width {
        return None;
    }

    Some((rect.x.saturating_add(dx), rect.y.saturating_add(dy)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hjkl_form::TextFieldEditor;

    fn backend_for(mut app: App, width: u16, height: u16) -> ratatui::backend::TestBackend {
        let backend = ratatui::backend::TestBackend::new(width, height);
        let mut terminal = ratatui::Terminal::new(backend.clone()).expect("test terminal");
        let _ = terminal.draw(|f| render_input_box(f, &mut app, f.area()));
        terminal.backend().clone()
    }

    #[test]
    fn input_box_height_minimum_is_border_rows() {
        let editor = TextFieldEditor::new(false);
        assert_eq!(input_box_height(&editor, 1), 2);
    }

    #[test]
    fn input_box_height_adds_border_rows() {
        let mut editor = TextFieldEditor::new(false);
        editor.set_text("one\ntwo\nthree");
        assert_eq!(input_box_height(&editor, 10), 5);
    }

    #[test]
    fn input_box_height_caps_at_max_lines() {
        let mut editor = TextFieldEditor::new(false);
        editor.set_text("1\n2\n3\n4\n5\n6");
        assert_eq!(input_box_height(&editor, 5), 5);
    }

    #[test]
    fn unfocused_render_shows_placeholder() {
        let app = {
            let mut app = App::default();
            app.focus = Focus::Thread;
            app
        };
        let backend = backend_for(app, 40, 10);
        let buf = backend.buffer();
        assert!(buf_contains(buf, "Ctrl+W j to comment"));
    }

    #[test]
    fn focused_render_shows_hint_title() {
        let app = {
            let mut app = App::default();
            app.focus = Focus::InputBox;
            app
        };
        let backend = backend_for(app, 50, 10);
        let buf = backend.buffer();
        assert!(buf_contains(
            buf,
            "Comment (i: insert, Esc: normal, Enter: submit)"
        ));
    }

    fn buf_contains(buf: &ratatui::buffer::Buffer, needle: &str) -> bool {
        for y in 0..buf.area.height {
            let mut row = String::new();
            for x in 0..buf.area.width {
                row.push_str(buf[(x, y)].symbol());
            }
            if row.contains(needle) {
                return true;
            }
        }
        false
    }
}
