use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui_crossterm::FromCrossterm;
use termimad::{CompoundStyle, FmtLine, FmtText, MadSkin};

pub fn render_markdown(src: &str, width: u16) -> Text<'static> {
    let skin = MadSkin::default();
    render_markdown_with_skin(src, &skin, width)
}

fn render_markdown_with_skin(src: &str, skin: &MadSkin, width: u16) -> Text<'static> {
    let fmt_text = FmtText::from(skin, src, Some(width as usize));
    let mut lines: Vec<Line<'static>> = Vec::new();

    for fmt_line in &fmt_text.lines {
        match fmt_line {
            FmtLine::Normal(fc) => {
                let line_style = skin.line_style(fc.kind);
                let base_style = compound_style_to_ratatui(&line_style.compound_style);

                match fc.kind {
                    termimad::CompositeKind::ListItem(depth) => {
                        let bullet = format!("{}• ", "  ".repeat(depth as usize));
                        let mut spans = vec![Span::styled(
                            bullet,
                            compound_style_to_ratatui(&skin.paragraph.compound_style),
                        )];
                        spans.extend(
                            fc.compounds
                                .iter()
                                .map(|c| compound_to_span(c, &base_style, skin)),
                        );
                        lines.push(Line::from(spans));
                    }
                    termimad::CompositeKind::Quote => {
                        let quote_style =
                            compound_style_to_ratatui(skin.quote_mark.compound_style());
                        let mut spans = vec![Span::styled(
                            format!("{} ", skin.quote_mark.nude_char()),
                            quote_style,
                        )];
                        spans.extend(
                            fc.compounds
                                .iter()
                                .map(|c| compound_to_span(c, &base_style, skin)),
                        );
                        lines.push(Line::from(spans));
                    }
                    _ => {
                        let spans: Vec<Span<'static>> = fc
                            .compounds
                            .iter()
                            .map(|c| compound_to_span(c, &base_style, skin))
                            .collect();
                        lines.push(Line::from(spans));
                    }
                }
            }
            FmtLine::TableRow(row) => {
                let border_style = compound_style_to_ratatui(&skin.table.compound_style);
                let sep = Span::styled(skin.table_border_chars.vertical.to_string(), border_style);

                let mut spans = Vec::new();
                spans.push(sep.clone());
                for cell in &row.cells {
                    let cell_base = compound_style_to_ratatui(&skin.table.compound_style);
                    let padding = cell
                        .spacing
                        .map_or(0, |s| s.width.saturating_sub(cell.visible_length));
                    let (left_pad, right_pad) = match cell.spacing {
                        Some(sp) => match sp.align {
                            termimad::Alignment::Center => {
                                let l = padding / 2;
                                (l, padding - l)
                            }
                            termimad::Alignment::Right => (padding, 0),
                            _ => (0, padding),
                        },
                        None => (0, 0),
                    };

                    spans.push(Span::styled(" ".repeat(left_pad), cell_base));
                    spans.extend(
                        cell.compounds
                            .iter()
                            .map(|c| compound_to_span(c, &cell_base, skin)),
                    );
                    spans.push(Span::styled(" ".repeat(right_pad), cell_base));
                    spans.push(sep.clone());
                }
                lines.push(Line::from(spans));
            }
            FmtLine::TableRule(rule) => {
                let border_style = compound_style_to_ratatui(&skin.table.compound_style);
                let h_char = skin.table_border_chars.horizontal;
                let left_corner = match rule.position {
                    termimad::RelativePosition::Top => skin.table_border_chars.top_left_corner,
                    termimad::RelativePosition::Bottom => {
                        skin.table_border_chars.bottom_left_corner
                    }
                    _ => skin.table_border_chars.left_junction,
                };
                let right_corner = match rule.position {
                    termimad::RelativePosition::Top => skin.table_border_chars.top_right_corner,
                    termimad::RelativePosition::Bottom => {
                        skin.table_border_chars.bottom_right_corner
                    }
                    _ => skin.table_border_chars.right_junction,
                };
                let junction = match rule.position {
                    termimad::RelativePosition::Top => skin.table_border_chars.top_junction,
                    termimad::RelativePosition::Bottom => skin.table_border_chars.bottom_junction,
                    _ => skin.table_border_chars.cross,
                };

                let mut segments = vec![Span::styled(left_corner.to_string(), border_style)];
                for (i, &w) in rule.widths.iter().enumerate() {
                    if i > 0 {
                        segments.push(Span::styled(junction.to_string(), border_style));
                    }
                    segments.push(Span::styled(h_char.to_string().repeat(w), border_style));
                }
                segments.push(Span::styled(right_corner.to_string(), border_style));
                lines.push(Line::from(segments));
            }
            FmtLine::HorizontalRule => {
                let rule_char = skin.horizontal_rule.nude_char();
                let rule_style = compound_style_to_ratatui(skin.horizontal_rule.compound_style());
                lines.push(Line::from(Span::styled(
                    rule_char.to_string().repeat(width as usize),
                    rule_style,
                )));
            }
        }
    }

    Text::from(lines)
}

fn compound_to_span(
    c: &termimad::minimad::Compound<'_>,
    base: &Style,
    skin: &MadSkin,
) -> Span<'static> {
    let line_style = termimad::LineStyle::default();
    let termimad_style = skin.compound_style(&line_style, c);
    let style = compound_style_to_ratatui(&termimad_style);

    // If termimad gave no explicit fg, fall back to the line's base style.
    let final_style = if termimad_style.object_style.foreground_color.is_none() && base.fg.is_some()
    {
        style.patch(*base)
    } else {
        style
    };

    Span::styled(c.as_str().to_string(), final_style)
}

fn compound_style_to_ratatui(cs: &CompoundStyle) -> Style {
    Style::from_crossterm(cs.object_style)
}
