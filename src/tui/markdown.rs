use minimad::{CompositeStyle, Line as MdLine};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};

pub fn render_markdown(src: &str) -> Text<'static> {
    let md = minimad::parse_text(src, minimad::Options::default());
    let mut lines: Vec<Line<'static>> = Vec::new();

    for md_line in &md.lines {
        match md_line {
            MdLine::Normal(composite) => {
                lines.push(composite_to_line(composite));
            }
            MdLine::CodeFence(composite) => {
                if composite.compounds.is_empty() {
                    lines.push(Line::from(""));
                } else {
                    let code_style = Style::default().fg(Color::Yellow);
                    let spans: Vec<Span<'static>> = composite
                        .compounds
                        .iter()
                        .map(|c| Span::styled(c.src.to_string(), code_style))
                        .collect();
                    lines.push(Line::from(spans));
                }
            }
            MdLine::HorizontalRule => {
                lines.push(Line::from("─".repeat(60)));
            }
            MdLine::TableRow(row) => {
                let text: String = row
                    .cells
                    .iter()
                    .map(|c| c.compounds.iter().map(|p| p.src).collect::<String>())
                    .collect::<Vec<_>>()
                    .join(" │ ");
                lines.push(Line::from(text));
            }
            MdLine::TableRule(_) => {}
        }
    }

    Text::from(lines)
}

fn composite_to_line(composite: &minimad::Composite) -> Line<'static> {
    match composite.style {
        CompositeStyle::Header(level) => {
            let prefix = format!("{} ", "#".repeat(level as usize));
            let bold = Style::default().add_modifier(Modifier::BOLD);
            let mut spans = vec![Span::styled(prefix, bold)];
            spans.extend(
                composite
                    .compounds
                    .iter()
                    .map(|c| compound_to_span_with_base(c, bold)),
            );
            Line::from(spans)
        }
        CompositeStyle::Code => {
            let code_style = Style::default().fg(Color::Yellow);
            let spans: Vec<Span<'static>> = composite
                .compounds
                .iter()
                .map(|c| Span::styled(c.src.to_string(), code_style))
                .collect();
            Line::from(spans)
        }
        CompositeStyle::ListItem(depth) => {
            let bullet = format!("{}• ", "  ".repeat(depth as usize));
            let mut spans = vec![Span::raw(bullet)];
            spans.extend(composite.compounds.iter().map(compound_to_span));
            Line::from(spans)
        }
        CompositeStyle::Quote => {
            let mut spans = vec![Span::styled("▌ ", Style::default().fg(Color::DarkGray))];
            spans.extend(composite.compounds.iter().map(compound_to_span));
            Line::from(spans)
        }
        CompositeStyle::Paragraph => {
            let spans: Vec<Span<'static>> =
                composite.compounds.iter().map(compound_to_span).collect();
            Line::from(spans)
        }
    }
}

fn compound_to_span(c: &minimad::Compound) -> Span<'static> {
    compound_to_span_with_base(c, Style::default())
}

fn compound_to_span_with_base(c: &minimad::Compound, base: Style) -> Span<'static> {
    let mut style = base;
    if c.code {
        style = style.fg(Color::Yellow);
    }
    if c.bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    if c.italic {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if c.strikeout {
        style = style.add_modifier(Modifier::CROSSED_OUT);
    }
    Span::styled(c.src.to_string(), style)
}
