use crate::tui::app::App;
use crate::tui::highlight::build_renderer;
use crate::tui::input_box::{
    inner_width, input_box_height, max_input_box_height, render_input_box,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// Approximates the number of visual rows a line occupies once ratatui
/// word-wraps it at `width` columns (with `Wrap { trim: false }`). ratatui does
/// not expose its wrap mapping, so this greedy word-wrap is a best-effort match
/// used to drive selection scroll-follow.
fn wrapped_row_count(line: &Line, width: u16) -> u16 {
    use unicode_width::UnicodeWidthStr;

    let width = width.max(1) as usize;
    let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
    if text.is_empty() {
        return 1;
    }

    let mut rows: u16 = 1;
    let mut col: usize = 0;
    for word in text.split_inclusive(' ') {
        let w = UnicodeWidthStr::width(word);
        if w > width {
            // Word longer than a row: hard-break across multiple rows.
            if col > 0 {
                rows = rows.saturating_add(1);
            }
            let full = w / width;
            let rem = w % width;
            if rem == 0 {
                rows = rows.saturating_add(full.saturating_sub(1) as u16);
                col = width;
            } else {
                rows = rows.saturating_add(full as u16);
                col = rem;
            }
        } else if col + w > width {
            rows = rows.saturating_add(1);
            col = w;
        } else {
            col += w;
        }
    }
    rows
}

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
    let total_width = f.area().width;
    let input_text_width = inner_width(total_width);
    let input_height = match &app.input_editor {
        Some(editor) => {
            input_box_height(editor, max_input_box_height(total_height), input_text_width)
        }
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

    // Clone what we need from the thread to avoid borrow conflicts
    let thread_data = match &app.thread {
        Some(t) => (
            t.issue_id,
            t.title.clone(),
            t.author.clone(),
            t.created_at.clone(),
            t.updated_at.clone(),
            t.description.clone(),
            t.comments
                .iter()
                .map(|c| {
                    (
                        c.comment_id,
                        c.author.clone(),
                        c.created_at.clone(),
                        c.content.clone(),
                    )
                })
                .collect::<Vec<_>>(),
        ),
        None => return,
    };

    let (issue_id, title, author, _created_at, updated_at, description, comments) = thread_data;

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("#{} ", issue_id),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(title, Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(author, Style::default().fg(Color::Yellow)),
        Span::raw("  "),
        Span::styled(updated_at, Style::default().fg(Color::DarkGray)),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    let renderer = build_renderer(markdown_theme());

    // Track which item each body line belongs to (0 = description, 1.. = comments)
    let mut item_for_line: Vec<usize> = Vec::new();
    let mut body_lines: Vec<Line> = Vec::new();

    // Item 0: description
    let desc_text = the_other_tui_markdown::into_text_with_renderer(&description, &renderer);
    item_for_line.extend(std::iter::repeat_n(0, desc_text.lines.len()));
    for line in desc_text.lines {
        body_lines.push(line);
    }

    // Items 1..n: comments
    for (ci, (_comment_id, author, created_at, content)) in comments.iter().enumerate() {
        let item_idx = ci + 1;
        item_for_line.push(item_idx); // blank separator line
        body_lines.push(Line::from(""));
        item_for_line.push(item_idx); // header line
        body_lines.push(Line::from(vec![
            Span::styled(author.clone(), Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled(created_at.clone(), Style::default().fg(Color::DarkGray)),
        ]));
        let comment_text = the_other_tui_markdown::into_text_with_renderer(content, &renderer);
        for line in comment_text.lines {
            item_for_line.push(item_idx);
            body_lines.push(line);
        }
    }

    // Apply gutter highlight (only when Focus::Thread)
    let selected = app.thread_selected;
    let is_thread_focus = matches!(app.focus, crate::tui::app::Focus::Thread);

    let body_lines: Vec<Line> = body_lines
        .into_iter()
        .enumerate()
        .map(|(i, mut line)| {
            let item = item_for_line.get(i).copied().unwrap_or(0);
            if is_thread_focus && item == selected {
                line.spans
                    .insert(0, Span::styled("│ ", Style::default().fg(Color::Yellow)));
            } else {
                line.spans.insert(0, Span::raw("  "));
            }
            line
        })
        .collect();

    // Compute the visual (post-wrap) row offset of each item's first line so
    // that j/k scroll-follow lands on the right row even when items wrap past
    // the viewport height. Wrapping is recomputed here at the real body width
    // because ratatui does not expose its own wrap mapping.
    let body_width = chunks[1].width;
    let item_count = comments.len() + 1;
    let mut item_starts = vec![0usize; item_count];
    let mut seen = vec![false; item_count];
    let mut cum_rows = 0usize;
    for (i, line) in body_lines.iter().enumerate() {
        let item = item_for_line.get(i).copied().unwrap_or(0);
        if item < item_count && !seen[item] {
            item_starts[item] = cum_rows;
            seen[item] = true;
        }
        cum_rows += wrapped_row_count(line, body_width) as usize;
    }
    app.item_line_starts = item_starts;

    let body = Paragraph::new(body_lines)
        .scroll((app.thread_scroll, 0))
        .wrap(Wrap { trim: false });
    f.render_widget(body, chunks[1]);

    render_input_box(f, app, chunks[2]);
}

#[cfg(test)]
mod tests {
    use super::wrapped_row_count;
    use ratatui::text::Line;

    #[test]
    fn empty_line_is_one_row() {
        assert_eq!(wrapped_row_count(&Line::from(""), 10), 1);
    }

    #[test]
    fn short_line_fits_one_row() {
        assert_eq!(wrapped_row_count(&Line::from("hello"), 10), 1);
    }

    #[test]
    fn word_wraps_to_next_row() {
        // "hello world" = 11 cols at width 8 -> "hello " then "world"
        assert_eq!(wrapped_row_count(&Line::from("hello world"), 8), 2);
    }

    #[test]
    fn long_text_wraps_to_multiple_rows() {
        let text = "one two three four five six seven eight nine ten";
        // width 10 forces several wraps; logical line count would be 1
        assert!(wrapped_row_count(&Line::from(text), 10) >= 4);
    }

    #[test]
    fn single_word_longer_than_width_hard_breaks() {
        // 25-char unbroken word at width 10 -> ceil(25/10) = 3 rows
        assert_eq!(wrapped_row_count(&Line::from("a".repeat(25)), 10), 3);
    }

    #[test]
    fn zero_width_does_not_panic() {
        assert!(wrapped_row_count(&Line::from("hello"), 0) >= 1);
    }
}
