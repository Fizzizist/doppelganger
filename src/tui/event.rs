use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;

pub fn next_key(timeout: Duration) -> std::io::Result<Option<KeyEvent>> {
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(key) => Ok(Some(key)),
            _ => Ok(None),
        }
    } else {
        Ok(None)
    }
}
