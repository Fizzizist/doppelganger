use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

pub enum RenderBlock {
    Prose(String),
    Table {
        header: Vec<String>,
        rows: Vec<Vec<String>>,
    },
}

pub fn parse_markdown(markdown: &str) -> Vec<RenderBlock> {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(markdown, opts);

    let mut blocks = Vec::new();
    let mut prose_events: Vec<Event<'_>> = Vec::new();
    let mut in_table = false;
    let mut table_header: Vec<String> = Vec::new();
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut in_header = false;
    let mut current_cell_text = String::new();

    for event in parser {
        match &event {
            Event::Start(Tag::Table(_)) => {
                flush_prose(&mut prose_events, &mut blocks);
                in_table = true;
                table_header = Vec::new();
                table_rows = Vec::new();
            }
            Event::Start(Tag::TableHead) => {
                in_header = true;
            }
            Event::End(TagEnd::TableHead) => {
                in_header = false;
            }
            Event::Start(Tag::TableRow) => {
                table_rows.push(Vec::new());
            }
            Event::Start(Tag::TableCell) => {
                current_cell_text = String::new();
            }
            Event::End(TagEnd::TableCell) => {
                let cell = current_cell_text.trim().to_string();
                if in_header {
                    table_header.push(cell);
                } else if let Some(row) = table_rows.last_mut() {
                    row.push(cell);
                }
            }
            Event::End(TagEnd::Table) => {
                blocks.push(RenderBlock::Table {
                    header: table_header.clone(),
                    rows: table_rows.clone(),
                });
                in_table = false;
                table_header = Vec::new();
                table_rows = Vec::new();
            }
            _ => {}
        }

        if in_table {
            if let Event::Text(t) | Event::Code(t) = &event {
                current_cell_text.push_str(t.as_ref());
            }
        } else if !matches!(
            &event,
            Event::Start(Tag::Table(_))
                | Event::Start(Tag::TableHead)
                | Event::Start(Tag::TableRow)
                | Event::Start(Tag::TableCell)
                | Event::End(TagEnd::Table)
                | Event::End(TagEnd::TableHead)
                | Event::End(TagEnd::TableRow)
                | Event::End(TagEnd::TableCell)
        ) {
            prose_events.push(event);
        }
    }

    flush_prose(&mut prose_events, &mut blocks);
    blocks
}

fn flush_prose(events: &mut Vec<Event<'_>>, blocks: &mut Vec<RenderBlock>) {
    if events.is_empty() {
        return;
    }
    let text = events_to_markdown(events);
    if !text.trim().is_empty() {
        blocks.push(RenderBlock::Prose(text));
    }
    events.clear();
}

fn events_to_markdown(events: &[Event<'_>]) -> String {
    let mut output = String::new();
    for event in events {
        match event {
            Event::Start(Tag::Paragraph) => {
                if !output.is_empty() && !output.ends_with('\n') {
                    output.push('\n');
                }
                output.push('\n');
            }
            Event::End(TagEnd::Paragraph) => {
                output.push('\n');
            }
            Event::Start(Tag::Heading { level, .. }) => {
                if !output.is_empty() && !output.ends_with('\n') {
                    output.push('\n');
                }
                for _ in 0..*level as u8 {
                    output.push('#');
                }
                output.push(' ');
            }
            Event::End(TagEnd::Heading(_)) => {
                output.push('\n');
            }
            Event::Start(Tag::BlockQuote(_)) => {
                output.push_str("> ");
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                output.push('\n');
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                output.push_str("```");
                if let pulldown_cmark::CodeBlockKind::Fenced(lang) = kind {
                    output.push_str(lang.as_ref());
                }
                output.push('\n');
            }
            Event::End(TagEnd::CodeBlock) => {
                output.push_str("```\n");
            }
            Event::Start(Tag::List(_)) => {}
            Event::End(TagEnd::List(_)) => {
                output.push('\n');
            }
            Event::Start(Tag::Item) => {
                output.push_str("- ");
            }
            Event::End(TagEnd::Item) => {
                output.push('\n');
            }
            Event::Start(Tag::Emphasis) => {
                output.push('*');
            }
            Event::End(TagEnd::Emphasis) => {
                output.push('*');
            }
            Event::Start(Tag::Strong) => {
                output.push_str("**");
            }
            Event::End(TagEnd::Strong) => {
                output.push_str("**");
            }
            Event::Start(Tag::Strikethrough) => {
                output.push_str("~~");
            }
            Event::End(TagEnd::Strikethrough) => {
                output.push_str("~~");
            }
            Event::Start(Tag::Link { .. }) => {
                output.push('[');
            }
            Event::End(TagEnd::Link) => {
                output.push(']');
            }
            Event::Text(t) => {
                output.push_str(t.as_ref());
            }
            Event::Code(c) => {
                output.push('`');
                output.push_str(c.as_ref());
                output.push('`');
            }
            Event::SoftBreak => {
                output.push(' ');
            }
            Event::HardBreak => {
                output.push('\n');
            }
            Event::Rule => {
                output.push_str("---\n");
            }
            _ => {}
        }
    }
    output
}

pub fn render_prose(markdown: &str) -> ratatui::text::Text<'static> {
    let text = tui_markdown::from_str(markdown);
    owned_text(text)
}

fn owned_text(text: ratatui::text::Text<'_>) -> ratatui::text::Text<'static> {
    ratatui::text::Text::from_iter(text.lines.into_iter().map(owned_line))
}

fn owned_line(line: ratatui::text::Line<'_>) -> ratatui::text::Line<'static> {
    ratatui::text::Line::from_iter(line.spans.into_iter().map(owned_span))
}

fn owned_span(span: ratatui::text::Span<'_>) -> ratatui::text::Span<'static> {
    ratatui::text::Span::styled(span.content.into_owned(), span.style)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_produces_single_prose() {
        let blocks = parse_markdown("Hello world");
        assert_eq!(
            blocks.len(),
            1,
            "plain text should produce exactly one block"
        );
        match &blocks[0] {
            RenderBlock::Prose(text) => {
                assert!(!text.trim().is_empty(), "prose should have content");
            }
            RenderBlock::Table { .. } => panic!("expected Prose, got Table"),
        }
    }

    #[test]
    fn table_produces_table_block() {
        let md = "\
| Name | Age |
|------|-----|
| Alice | 30 |
| Bob | 25 |
";
        let blocks = parse_markdown(md);
        assert!(!blocks.is_empty(), "should produce at least one block");
        let tables: Vec<_> = blocks
            .iter()
            .filter(|b| matches!(b, RenderBlock::Table { .. }))
            .collect();
        assert_eq!(tables.len(), 1, "should have exactly one table block");

        match &tables[0] {
            RenderBlock::Table { header, rows } => {
                assert_eq!(header, &vec!["Name".to_string(), "Age".to_string()]);
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0], vec!["Alice".to_string(), "30".to_string()]);
                assert_eq!(rows[1], vec!["Bob".to_string(), "25".to_string()]);
            }
            RenderBlock::Prose(_) => panic!("expected Table"),
        }
    }

    #[test]
    fn mixed_content_has_prose_and_table() {
        let md = "\
Some intro text.

| H1 | H2 |
|----|----|
| a  | b  |

Closing text.
";
        let blocks = parse_markdown(md);
        assert!(
            blocks.len() >= 3,
            "should produce at least 3 blocks, got {}",
            blocks.len()
        );
        let kinds: Vec<&str> = blocks
            .iter()
            .map(|b| match b {
                RenderBlock::Prose(_) => "prose",
                RenderBlock::Table { .. } => "table",
            })
            .collect();
        assert!(
            kinds.contains(&"prose"),
            "should have at least one prose block"
        );
        assert!(
            kinds.contains(&"table"),
            "should have at least one table block"
        );
    }

    #[test]
    fn render_prose_returns_text() {
        let text = render_prose("Hello **world**");
        assert!(!text.lines.is_empty(), "rendered text should have lines");
    }
}
