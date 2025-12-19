use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;

pub fn read_key_event(timeout: Duration) -> std::io::Result<Option<KeyEvent>> {
    if event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            return Ok(Some(key));
        }
    }
    Ok(None)
}
