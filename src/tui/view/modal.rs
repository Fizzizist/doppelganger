use crate::tui::app::{App, ModalState};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area
        .width
        .saturating_sub(width)
        .saturating_div(2)
        .min(area.width.saturating_sub(width));
    let y = area
        .height
        .saturating_sub(height)
        .saturating_div(2)
        .min(area.height.saturating_sub(height));
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect::new(area.x + x, area.y + y, width, height)
}

pub fn render(f: &mut ratatui::Frame, app: &App) {
    let area = centered_rect(40, 5, f.area());
    match &app.modal {
        Some(ModalState::NameInput) => {
            let input = Paragraph::new(Line::from(Span::styled(
                app.input_buffer.clone(),
                Style::default().fg(Color::White),
            )))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("New issue name (blank = untitled)")
                    .style(Style::default().fg(Color::Cyan)),
            );
            f.render_widget(Clear, area);
            f.render_widget(input, area);
        }
        Some(ModalState::Error(msg)) => {
            let text = Paragraph::new(msg.clone())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Error")
                        .style(Style::default().fg(Color::Red)),
                )
                .wrap(Wrap { trim: true });
            let err_area = centered_rect(50, 7, f.area());
            f.render_widget(Clear, err_area);
            f.render_widget(text, err_area);
        }
        None => {}
    }
}
