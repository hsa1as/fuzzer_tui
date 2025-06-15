// app.rs
use crate::{
    popup::*,
    window::{Window, WindowTransition},
};
use crossterm::event::KeyEvent;
use ratatui::{Frame, prelude::*, widgets::Paragraph};
use std::collections::{HashMap, VecDeque};

pub fn centered_rect(percent_x: u16, percent_y: u16, parent_area: Rect) -> Rect {
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(parent_area);

    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical_chunks[1]);

    horizontal_chunks[1]
}

pub trait Property: std::fmt::Debug {
    fn id(&self) -> &str;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub enum Request {
    Popup(Popup),
}

pub struct App {
    window_stack: VecDeque<Box<dyn Window>>,
    pub properties: HashMap<String, Box<dyn Property>>,
    popup: Option<Popup>,
}

impl App {
    pub fn new(initial_window: Box<dyn Window>) -> Self {
        let mut stack = VecDeque::new();
        stack.push_back(initial_window);
        App {
            window_stack: stack,
            properties: HashMap::new(),
            popup: None,
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> bool {
        if self.popup.is_some() {
            // Handle popup input
            self.popup = None; // Reassign the popup if needed
            return true;
        }
        match key.code {
            crossterm::event::KeyCode::Char('q') => return false, // Quit
            crossterm::event::KeyCode::Char('b') => {
                self.window_stack.pop_back();
                if self.window_stack.is_empty() {
                    return false;
                }
            }
            _ => {
                if let Some(current) = self.window_stack.back_mut() {
                    let reqs = current.handle_input(key);
                    if let Some(reqs) = reqs {
                        for req in reqs {
                            self.handle_request(req);
                        }
                    }
                }
            }
        }
        true
    }

    pub fn render(&mut self, f: &mut Frame) {
        let size = f.size();

        // Layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(size);

        // Header
        let header = Paragraph::new("FlashFuzz v1.0")
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        f.render_widget(header, chunks[0]);

        // Footer
        let name = self
            .window_stack
            .back()
            .map(|w| w.name())
            .unwrap_or("Unknown Window");
        let footer = Paragraph::new(name)
            .style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::ITALIC),
            )
            .alignment(Alignment::Center);
        f.render_widget(footer, chunks[2]);

        // Current window
        if let Some(current) = self.window_stack.back_mut() {
            let v = current.render(f, chunks[1]);
            if let Some(reqs) = v {
                for r in reqs {
                    self.handle_request(r);
                }
            }
        }
        if let Some(p) = self.popup.as_mut() {
            let area_popup = centered_rect(30, 20, size);
            p.render(f, area_popup);
        }
    }

    pub fn handle_request(&mut self, req: Request) {
        match req {
            Request::Popup(popup) => {
                self.popup = Some(popup);
            }
        }
    }
}
