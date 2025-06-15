// window.rs
use crate::app::Request;
use crossterm::event::KeyEvent;
use ratatui::{Frame, prelude::*};

pub trait Window {
    fn name(&self) -> &str;
    fn render(&mut self, f: &mut Frame, area: Rect) -> Option<Vec<Request>>;
    fn handle_input(&mut self, key: KeyEvent) -> Option<Vec<Request>>;
}

pub enum WindowTransition {
    Push(Box<dyn Window>),
    Pop,
    None,
}
