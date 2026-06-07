use crate::tui::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

fn truncate_to_60(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= 60 {
        s.to_string()
    } else {
        let truncated: String = chars.into_iter().take(60).collect();
        format!("{truncated}...")
    }
}

pub fn render(f: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    let items: Vec<ListItem> = app
        .issues
        .iter()
        .map(|issue| {
            let title = match issue.name.as_deref() {
                Some(n) => n,
                None => issue.description.as_str(),
            };
            let display_title = truncate_to_60(title);

            let line = Line::from(vec![
                Span::styled(
                    format!("#{} ", issue.issue_id),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(display_title),
                Span::raw("  "),
                Span::styled(issue.author.clone(), Style::default().fg(Color::Yellow)),
                Span::raw("  "),
                Span::styled(
                    issue.updated_at.clone(),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Issues"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = ListState::default();
    if !app.issues.is_empty() {
        state.select(Some(app.selected_issue));
    }

    f.render_stateful_widget(list, chunks[0], &mut state);

    let help = Paragraph::new("j/k: navigate  Enter/l: select  q: quit")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[1]);
}
