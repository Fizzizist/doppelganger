use crate::tui::app::{App, Focus};
use hjkl_buffer::{
    Wrap,
    wrap::{segment_for_col, wrap_segments},
};
use hjkl_editor_tui::form::FormPalette;
use hjkl_engine::Host;
use hjkl_form::TextFieldEditor;
use hjkl_form::VimMode;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use unicode_width::UnicodeWidthStr;

pub fn input_box_height(editor: &TextFieldEditor, max_lines: u16, text_width: u16) -> u16 {
    if text_width == 0 {
        return 2;
    }
    let viewport = hjkl_buffer::Viewport {
        wrap: Wrap::Word,
        text_width,
        ..hjkl_buffer::Viewport::default()
    };
    let buffer = editor.buffer();
    let row_count = buffer.row_count();
    if row_count == 0 {
        return 2;
    }
    let screen_rows = buffer.screen_rows_between(&viewport, 0, row_count - 1);
    let content_lines = (screen_rows as u16).min(max_lines.saturating_sub(2));
    content_lines.saturating_add(2)
}

pub fn max_input_box_height(total_height: u16) -> u16 {
    total_height.saturating_sub(4).max(3)
}

pub fn inner_width(outer_width: u16) -> u16 {
    outer_width.saturating_sub(2)
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
    let text_width = inner.width;

    let cursor = match app.input_editor.as_mut() {
        Some(editor) => {
            update_viewport(editor, inner);

            let buffer = editor.buffer();
            let text = editor.text();
            let lines: Vec<Line> = if text.is_empty() && !focused {
                vec![placeholder_line(&palette)]
            } else {
                let selection = editor.editor.selection_highlight();
                let line_texts: Vec<&str> = text.split('\n').collect();
                let row_count = buffer.row_count();
                (0..row_count)
                    .flat_map(|row| {
                        let line_text = line_texts.get(row).copied().unwrap_or("");
                        let segments = wrap_segments(line_text, text_width, Wrap::Word);
                        match &selection {
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
                                        line_text.chars().count()
                                    };
                                    make_selection_lines(line_text, &segments, col_start, col_end)
                                } else {
                                    lines_from_segments(line_text, &segments)
                                }
                            }
                            _ => lines_from_segments(line_text, &segments),
                        }
                    })
                    .collect()
            };

            let cursor = if focused {
                cursor_xy(editor, inner, text_width)
            } else {
                None
            };
            let paragraph = Paragraph::new(lines).block(block);
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
    let text_width = rect.width;
    let v = editor.editor.host_mut().viewport_mut();
    v.wrap = Wrap::Word;
    v.text_width = text_width;
    v.width = rect.width;
    v.height = rect.height;
    v.top_row = 0;
    v.top_col = 0;
}

fn make_selection_lines(
    line_text: &str,
    segments: &[(usize, usize)],
    col_start: usize,
    col_end: usize,
) -> Vec<Line<'static>> {
    let mut result = Vec::new();

    for &(seg_start, seg_end) in segments {
        let seg_chars: Vec<char> = line_text
            .chars()
            .skip(seg_start)
            .take(seg_end - seg_start)
            .collect();
        let seg_len = seg_chars.len();

        let local_sel_start = col_start.saturating_sub(seg_start).min(seg_len);
        let local_sel_end = col_end.saturating_sub(seg_start).min(seg_len);

        if local_sel_start == 0 && local_sel_end >= seg_len {
            let seg_str: String = seg_chars.iter().collect();
            result.push(Line::from(Span::styled(
                seg_str,
                Style::default().add_modifier(Modifier::REVERSED),
            )));
        } else if local_sel_end <= local_sel_start || local_sel_start >= seg_len {
            let seg_str: String = seg_chars.iter().collect();
            result.push(Line::from(Span::raw(seg_str)));
        } else {
            let before: String = seg_chars.iter().take(local_sel_start).collect();
            let selected: String = seg_chars
                .iter()
                .skip(local_sel_start)
                .take(local_sel_end - local_sel_start)
                .collect();
            let after: String = seg_chars.iter().skip(local_sel_end).collect();

            let mut spans: Vec<Span<'static>> = Vec::new();
            if !before.is_empty() {
                spans.push(Span::raw(before));
            }
            if !selected.is_empty() {
                spans.push(Span::styled(
                    selected,
                    Style::default().add_modifier(Modifier::REVERSED),
                ));
            }
            if !after.is_empty() {
                spans.push(Span::raw(after));
            }
            result.push(Line::from(spans));
        }
    }

    if result.is_empty() {
        result.push(Line::from(""));
    }
    result
}

fn lines_from_segments(line_text: &str, segments: &[(usize, usize)]) -> Vec<Line<'static>> {
    let mut result = Vec::new();

    if segments.is_empty() {
        result.push(Line::from(""));
    } else {
        for &(start, end) in segments {
            let seg_str = segment_substr(line_text, start, end);
            result.push(Line::from(Span::raw(seg_str)));
        }
    }

    result
}

fn segment_substr(line_text: &str, start: usize, end: usize) -> String {
    line_text.chars().skip(start).take(end - start).collect()
}

fn placeholder_line(palette: &FormPalette) -> Line<'static> {
    Line::from(Span::styled("Ctrl+W j to comment", palette.placeholder))
}

fn cursor_xy(editor: &TextFieldEditor, rect: Rect, text_width: u16) -> Option<(u16, u16)> {
    let viewport = editor.editor.host().viewport();
    let buffer = editor.buffer();
    let (row, col) = editor.cursor();

    let text = editor.text();
    let line_text = text.split('\n').nth(row)?;
    let segments = wrap_segments(line_text, text_width, Wrap::Word);
    let seg_idx = segment_for_col(&segments, col);
    let &(seg_start, _seg_end) = segments.get(seg_idx)?;

    let prefix: String = line_text
        .chars()
        .skip(seg_start)
        .take(col - seg_start)
        .collect();
    let dx = UnicodeWidthStr::width(prefix.as_str()) as u16;

    let rows_before = if row > 0 {
        buffer.screen_rows_between(viewport, 0, row - 1)
    } else {
        0
    };
    let dy = (rows_before + seg_idx) as u16;

    if dy >= rect.height {
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
    fn input_box_height_minimum_includes_one_content_row() {
        let editor = TextFieldEditor::new(false);
        assert_eq!(input_box_height(&editor, 10, 40), 3);
    }

    #[test]
    fn input_box_height_adds_border_rows() {
        let mut editor = TextFieldEditor::new(false);
        editor.set_text("one\ntwo\nthree");
        assert_eq!(input_box_height(&editor, 10, 40), 5);
    }

    #[test]
    fn input_box_height_caps_at_max_lines() {
        let mut editor = TextFieldEditor::new(false);
        editor.set_text("1\n2\n3\n4\n5\n6");
        assert_eq!(input_box_height(&editor, 5, 40), 5);
    }

    #[test]
    fn input_box_height_wraps_long_line() {
        let mut editor = TextFieldEditor::new(false);
        editor.set_text("abcdefghijklmnopqrstuvwxyz");
        let height = input_box_height(&editor, 20, 10);
        assert!(
            height > 3,
            "a long line that wraps should produce more than 3 rows, got {height}"
        );
    }

    #[test]
    fn input_box_height_wraps_multiple_lines() {
        let mut editor = TextFieldEditor::new(false);
        editor.set_text("abcdefghijklmnop\nshort\nqrstuvwxyz123456");
        let height = input_box_height(&editor, 20, 10);
        assert!(
            height > 5,
            "multiple wrapping lines should produce more rows, got {height}"
        );
    }

    #[test]
    fn update_viewport_sets_wrap_and_text_width() {
        let mut editor = TextFieldEditor::new(false);
        let rect = Rect::new(0, 0, 40, 10);
        update_viewport(&mut editor, rect);
        let v = editor.editor.host().viewport();
        assert!(matches!(v.wrap, Wrap::Word), "wrap should be Word");
        assert_eq!(v.text_width, 40, "text_width should match inner width");
    }

    #[test]
    fn update_viewport_resets_top_row_and_col() {
        let mut editor = TextFieldEditor::new(false);
        editor.set_text(&"line\n".repeat(20));
        let rect = Rect::new(0, 0, 40, 3);
        update_viewport(&mut editor, rect);
        let v = editor.editor.host().viewport();
        assert_eq!(v.top_row, 0, "top_row should be 0 for input box");
        assert_eq!(v.top_col, 0, "top_col should be 0 with wrap active");
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
        assert_eq!(v.top_row, 0, "input box should not scroll vertically");
        assert_eq!(v.top_col, 0, "with wrap, top_col should be 0");
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
