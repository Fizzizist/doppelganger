use crate::tui::app::{App, Focus};
use hjkl_editor_tui::form::FormPalette;
use hjkl_engine::Host;
use hjkl_form::TextFieldEditor;
use hjkl_form::VimMode;
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
    total_height.saturating_sub(4).max(3)
}

pub fn render_input_box(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
    let palette = FormPalette::dark();
    let focused = matches!(app.focus, Focus::InputBox);

    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = match app.input_editor.as_ref() {
        Some(editor) if focused => {
            let mode_label = match editor.vim_mode() {
                VimMode::Normal => "NORMAL",
                VimMode::Insert => "INSERT",
                VimMode::Visual => "VISUAL",
                VimMode::VisualLine => "VISUAL LINE",
                VimMode::VisualBlock => "VISUAL BLOCK",
            };
            format!(" Comment [{mode_label}] ")
        }
        _ => " Comment ".to_string(),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(title, border_style));

    let inner = block.inner(area);

    let cursor = match app.input_editor.as_mut() {
        Some(editor) => {
            update_viewport(editor, inner);

            let text = editor.text();
            let lines: Vec<Line> = if text.is_empty() && !focused {
                vec![placeholder_line(&palette)]
            } else {
                let selection = editor.editor.selection_highlight();
                text.lines()
                    .enumerate()
                    .map(|(row, line)| match &selection {
                        Some(sel) if focused => {
                            let sel_start = sel.range.start.line as usize;
                            let sel_end = sel.range.end.line as usize;
                            if row >= sel_start && row <= sel_end {
                                let col_start = if row == sel_start {
                                    sel.range.start.col as usize
                                } else {
                                    0
                                };
                                let col_end = if row == sel_end {
                                    (sel.range.end.col as usize).saturating_add(1)
                                } else {
                                    line.chars().count()
                                };
                                make_selection_line(line, col_start, col_end)
                            } else {
                                Line::from(line.to_string())
                            }
                        }
                        _ => Line::from(line.to_string()),
                    })
                    .collect()
            };

            let cursor = if focused {
                cursor_xy(editor, inner)
            } else {
                None
            };
            let paragraph = Paragraph::new(lines)
                .block(block)
                .wrap(Wrap { trim: false });
            f.render_widget(paragraph, area);

            cursor
        }
        None => {
            let paragraph = Paragraph::new(vec![placeholder_line(&palette)]).block(block);
            f.render_widget(paragraph, area);

            None
        }
    };

    if let Some((x, y)) = cursor {
        f.set_cursor_position(ratatui::layout::Position { x, y });
    }
}

fn update_viewport(editor: &mut TextFieldEditor, rect: Rect) {
    let cursor = editor.editor.buffer().cursor();
    let v = editor.editor.host_mut().viewport_mut();
    v.width = rect.width;
    v.height = rect.height;
    if cursor.col < v.top_col {
        v.top_col = cursor.col;
    }
    if rect.width > 0 && cursor.col >= v.top_col + rect.width as usize {
        v.top_col = cursor.col + 1 - rect.width as usize;
    }
    if cursor.row < v.top_row {
        v.top_row = cursor.row;
    }
    if rect.height > 0 && cursor.row >= v.top_row + rect.height as usize {
        v.top_row = cursor.row + 1 - rect.height as usize;
    }
}

fn make_selection_line(text: &str, col_start: usize, col_end: usize) -> Line<'static> {
    let chars: Vec<char> = text.chars().collect();
    let mut spans: Vec<Span> = Vec::new();

    let before: String = chars.iter().take(col_start).collect();
    if !before.is_empty() {
        spans.push(Span::raw(before));
    }

    let selected: String = chars
        .iter()
        .skip(col_start)
        .take(col_end.saturating_sub(col_start))
        .collect();
    if !selected.is_empty() {
        spans.push(Span::styled(
            selected,
            Style::default().add_modifier(Modifier::REVERSED),
        ));
    }

    let after: String = chars.iter().skip(col_end).collect();
    if !after.is_empty() {
        spans.push(Span::raw(after));
    }

    if spans.is_empty() {
        Line::from("")
    } else {
        Line::from(spans)
    }
}

fn placeholder_line(palette: &FormPalette) -> Line<'static> {
    Line::from(Span::styled("Ctrl+W j to comment", palette.placeholder))
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
    fn focused_render_shows_mode_in_title() {
        let mut app = App::default();
        app.focus_input_box();
        let backend = backend_for(app, 50, 10);
        let buf = backend.buffer();
        assert!(buf_contains(buf, "[INSERT]"));
    }

    #[test]
    fn viewport_scrolls_to_keep_cursor_visible() {
        let mut editor = TextFieldEditor::new(false);
        editor.set_text(&"line\n".repeat(20));
        let rect = Rect::new(0, 0, 40, 3);
        update_viewport(&mut editor, rect);
        let v = editor.editor.host().viewport();
        assert_eq!(v.width, 40);
        assert_eq!(v.height, 3);
        assert!(
            v.top_row <= 19,
            "top_row should be clamped so cursor row 19 stays visible, got {}",
            v.top_row
        );
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
