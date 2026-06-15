use crate::tui::app::{App, Focus};
use hjkl_buffer::{
    Viewport, Wrap,
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
    let viewport = Viewport {
        wrap: Wrap::Word,
        text_width,
        ..Viewport::default()
    };
    let buffer = editor.buffer();
    let row_count = buffer.row_count();
    if row_count == 0 {
        return 2;
    }
    let screen_rows = buffer.screen_rows_between(&viewport, 0, row_count - 1);
    let content_lines = screen_rows
        .min(u16::MAX as usize)
        .min(max_lines.saturating_sub(2) as usize) as u16;
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
    {
        let v = editor.editor.host_mut().viewport_mut();
        v.wrap = Wrap::Word;
        v.text_width = text_width;
        v.width = rect.width;
        v.height = rect.height;
    }
    let mut viewport = *editor.editor.host().viewport();
    editor
        .editor
        .buffer_mut()
        .ensure_cursor_visible(&mut viewport);
    *editor.editor.host_mut().viewport_mut() = viewport;
}

// Slices `line_text` by char-index range [start, end).
// wrap_segments and cursor positions use char indices; selection
// Pos::col is documented as grapheme-indexed but in practice the
// engine moves by char, so char-based slicing is consistent here.
fn char_slice(line_text: &str, start: usize, end: usize) -> String {
    line_text
        .chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
}

fn make_selection_lines(
    line_text: &str,
    segments: &[(usize, usize)],
    col_start: usize,
    col_end: usize,
) -> Vec<Line<'static>> {
    segments
        .iter()
        .map(|&(seg_start, seg_end)| {
            let seg_len = seg_end.saturating_sub(seg_start);
            let local_sel_start = col_start.saturating_sub(seg_start).min(seg_len);
            let local_sel_end = col_end.saturating_sub(seg_start).min(seg_len);

            if local_sel_start == 0 && local_sel_end >= seg_len {
                Line::from(Span::styled(
                    char_slice(line_text, seg_start, seg_end),
                    Style::default().add_modifier(Modifier::REVERSED),
                ))
            } else if local_sel_end <= local_sel_start {
                Line::from(Span::raw(char_slice(line_text, seg_start, seg_end)))
            } else {
                let mut spans: Vec<Span<'static>> = Vec::new();
                let before = char_slice(line_text, seg_start, seg_start + local_sel_start);
                if !before.is_empty() {
                    spans.push(Span::raw(before));
                }
                let selected = char_slice(
                    line_text,
                    seg_start + local_sel_start,
                    seg_start + local_sel_end,
                );
                if !selected.is_empty() {
                    spans.push(Span::styled(
                        selected,
                        Style::default().add_modifier(Modifier::REVERSED),
                    ));
                }
                let after = char_slice(line_text, seg_start + local_sel_end, seg_end);
                if !after.is_empty() {
                    spans.push(Span::raw(after));
                }
                Line::from(spans)
            }
        })
        .collect()
}

fn lines_from_segments(line_text: &str, segments: &[(usize, usize)]) -> Vec<Line<'static>> {
    if segments.is_empty() {
        vec![Line::from("")]
    } else {
        segments
            .iter()
            .map(|&(start, end)| Line::from(Span::raw(char_slice(line_text, start, end))))
            .collect()
    }
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
        .take(col.saturating_sub(seg_start))
        .collect();
    let dx = UnicodeWidthStr::width(prefix.as_str()) as u16;

    let dy = if row == 0 {
        seg_idx as u16
    } else {
        let rows_before = buffer.screen_rows_between(viewport, 0, row - 1);
        (rows_before.min(u16::MAX as usize) as u16).saturating_add(seg_idx as u16)
    };

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
    fn input_box_height_wraps_long_line_exact() {
        let mut editor = TextFieldEditor::new(false);
        editor.set_text("abcdefghijklmnopqrstuvwxyz");
        // 26 chars at width 10: "abcdefghij" (10) + "klmnopqrst" (10) + "uvwxyz" (6) = 3 screen rows
        // height = min(3, 20-2) + 2 = 5
        assert_eq!(input_box_height(&editor, 20, 10), 5);
    }

    #[test]
    fn input_box_height_wraps_multiple_lines_exact() {
        let mut editor = TextFieldEditor::new(false);
        // "abcdefghijklmnop" at width 10: 2 screen rows
        // "short" at width 10: 1 screen row
        // "qrstuvwxyz123456" at width 10: 2 screen rows
        // total = 5 screen rows + 2 border = 7
        editor.set_text("abcdefghijklmnop\nshort\nqrstuvwxyz123456");
        assert_eq!(input_box_height(&editor, 20, 10), 7);
    }

    #[test]
    fn input_box_height_clamps_overflow() {
        let mut editor = TextFieldEditor::new(false);
        // Many lines wrapping to exceed u16 range should not panic
        editor.set_text(&"abcdefghij\n".repeat(7000));
        let height = input_box_height(&editor, u16::MAX, 10);
        assert!(height <= u16::MAX, "height should not overflow u16");
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
    fn update_viewport_delegates_to_ensure_cursor_visible() {
        let mut editor = TextFieldEditor::new(false);
        editor.set_text(&"line\n".repeat(20));
        let rect = Rect::new(0, 0, 40, 3);
        update_viewport(&mut editor, rect);
        let v = editor.editor.host().viewport();
        assert!(
            v.top_row <= 19,
            "top_row should be clamped so cursor row 19 stays visible, got {}",
            v.top_row
        );
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
    fn cursor_xy_on_first_line() {
        let mut editor = TextFieldEditor::new(false);
        editor.set_text("hello");
        let rect = Rect::new(0, 0, 40, 10);
        update_viewport(&mut editor, rect);
        let result = cursor_xy(&editor, rect, 38);
        // cursor should be at col 0 after set_text (cursor lands at end)
        assert!(result.is_some(), "cursor should be visible");
    }

    #[test]
    fn cursor_xy_on_wrapped_line_second_segment() {
        let mut editor = TextFieldEditor::new(false);
        // 20 chars at width 10: wraps into 2 segments
        editor.set_text("abcdefghijklmnopqrst");
        let rect = Rect::new(0, 0, 12, 5);
        update_viewport(&mut editor, rect);
        // cursor is at end (col 20) which is on the second segment
        let result = cursor_xy(&editor, rect, 10);
        assert!(result.is_some(), "cursor on wrapped line should be visible");
        let (_x, y) = result.expect("cursor position");
        assert!(
            y >= 1,
            "cursor on second visual row should have y >= 1, got {y}"
        );
    }

    #[test]
    fn make_selection_lines_full_segment_selected() {
        let segments: Vec<(usize, usize)> = vec![(0, 5), (5, 10)];
        let result = make_selection_lines("abcdefghij", &segments, 0, 10);
        assert_eq!(result.len(), 2, "should produce 2 lines for 2 segments");
    }

    #[test]
    fn make_selection_lines_partial_selection() {
        let segments: Vec<(usize, usize)> = vec![(0, 5), (5, 10)];
        let result = make_selection_lines("abcdefghij", &segments, 2, 8);
        // First segment: cols 0-5, selection 2-5 -> before "ab" + selected "cde"
        // Second segment: cols 5-10, selection 5-8 -> selected "fgh" + after "ij"
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn make_selection_lines_spanning_segments() {
        let segments: Vec<(usize, usize)> = vec![(0, 3), (3, 6), (6, 9)];
        let result = make_selection_lines("abcdefghi", &segments, 2, 7);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn lines_from_segments_empty() {
        let result = lines_from_segments("hello", &[]);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn lines_from_segments_nonempty() {
        let segments: Vec<(usize, usize)> = vec![(0, 5)];
        let result = lines_from_segments("hello", &segments);
        assert_eq!(result.len(), 1);
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
