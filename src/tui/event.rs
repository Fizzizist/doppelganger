use ratatui::crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use std::time::Duration;

pub enum AppEvent {
    Key(KeyEvent),
    Tick,
}

pub async fn next_event(tick_interval: Duration) -> crate::error::Result<AppEvent> {
    if event::poll(tick_interval).map_err(|e| crate::error::Error::Tui(e.to_string()))? {
        match event::read().map_err(|e| crate::error::Error::Tui(e.to_string()))? {
            CrosstermEvent::Key(key) => Ok(AppEvent::Key(key)),
            _ => Ok(AppEvent::Tick),
        }
    } else {
        Ok(AppEvent::Tick)
    }
}
