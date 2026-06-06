use crate::db::models::Issue;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, HighlightSpacing, Row, Table, TableState},
};

pub struct IssueListScreen {
    state: TableState,
    issues: Vec<Issue>,
}

impl IssueListScreen {
    pub fn new(issues: Vec<Issue>) -> Self {
        let mut state = TableState::new();
        if !issues.is_empty() {
            state.select(Some(0));
        }
        Self { state, issues }
    }

    pub fn selected(&self) -> Option<&Issue> {
        self.state.selected().and_then(|i| self.issues.get(i))
    }

    pub fn selected_issue_id(&self) -> Option<i64> {
        self.selected().map(|iss| iss.issue_id)
    }

    pub fn select_down(&mut self) {
        if self.issues.is_empty() {
            return;
        }
        let max = self.issues.len().saturating_sub(1);
        let current = self.state.selected().unwrap_or(0);
        if current < max {
            self.state.select(Some(current + 1));
        }
    }

    pub fn select_up(&mut self) {
        if self.issues.is_empty() {
            return;
        }
        let current = self.state.selected().unwrap_or(0);
        if current > 0 {
            self.state.select(Some(current - 1));
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let header = Row::new(vec![
            Cell::from("#"),
            Cell::from("Name"),
            Cell::from("Author"),
            Cell::from("Updated"),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

        let rows: Vec<Row<'_>> = self
            .issues
            .iter()
            .map(|iss| {
                let name_display = iss.name.clone().unwrap_or_else(|| "(untitled)".to_string());
                Row::new(vec![
                    Cell::from(iss.issue_id.to_string()),
                    Cell::from(name_display),
                    Cell::from(iss.author.clone()),
                    Cell::from(iss.updated_at.clone()),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(5),
                Constraint::Min(20),
                Constraint::Length(15),
                Constraint::Min(20),
            ],
        )
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Issues"))
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_spacing(HighlightSpacing::Always)
        .highlight_symbol(">> ");

        frame.render_stateful_widget(table, area, &mut self.state);
    }
}

pub fn render_issue_list_snapshot(issues: Vec<Issue>, area: Rect) -> ratatui::buffer::Buffer {
    let mut screen = IssueListScreen::new(issues);
    let backend = ratatui::backend::TestBackend::new(area.width, area.height);
    let mut terminal = ratatui::Terminal::new(backend).expect("create test terminal");
    terminal
        .draw(|frame| screen.render(frame, frame.area()))
        .expect("draw");
    terminal.backend().buffer().clone()
}
