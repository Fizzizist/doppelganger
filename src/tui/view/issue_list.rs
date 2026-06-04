use ratatui::{
    Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::tui::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let items: Vec<ListItem> = app
        .issues
        .iter()
        .map(|issue| {
            let title = issue.name.as_deref().unwrap_or("(untitled)").to_string();
            let label = format!("{title}  #{}", issue.issue_id);
            ListItem::new(Line::from(Span::raw(label)))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.selected));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Issues  (j/k=navigate  Enter/l=open  q=quit)"),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, area, &mut state);
}
