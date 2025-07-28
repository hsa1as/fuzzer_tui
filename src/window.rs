use std::any::Any;

// window.rs
use crate::app::Request;
use crossterm::event::KeyEvent;
use ratatui::{prelude::*, Frame};

pub trait Window {
    fn name(&self) -> &str;
    fn render(&mut self, f: &mut Frame, area: Rect) -> Option<Vec<Request>>;
    fn handle_input(&mut self, key: KeyEvent) -> Option<Vec<Request>>;
    fn capture_all_input(&self) -> bool {
        false
    }
    fn send_property(&mut self, name: String, _property: &dyn Any) {
        return;
    }
}
