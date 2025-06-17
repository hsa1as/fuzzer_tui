// app.rs
use crate::utils::centered_rect::centered_rect;
use crate::{
    popup::*,
    window::Window,                         // Removed WindowTransition
    windows::project_window::ProjectWindow, // Added
};
use crossterm::event::KeyEvent;
use ratatui::{prelude::*, widgets::Paragraph, Frame};
use std::collections::{HashMap, VecDeque};

pub trait Property: std::fmt::Debug {
    fn id(&self) -> &str;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub enum Request {
    Popup(Popup),
    PushWindow(Box<dyn Window>), // New request to push a window
    PopWindow,
}

pub struct App {
    window_stack: VecDeque<Box<dyn Window>>,
    pub properties: HashMap<String, Box<dyn Property>>,
    popup: Option<Popup>,
}

impl App {
    pub fn new() -> Self {
        // Removed initial_window parameter
        let mut stack: VecDeque<Box<dyn Window>> = VecDeque::new(); // Explicitly typed
                                                                    // Push ProjectWindow first
        stack.push_back(Box::new(ProjectWindow::new())); // Coercion will happen here
        App {
            window_stack: stack,
            properties: HashMap::new(),
            popup: None,
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> bool {
        let mut reqs = None;
        if self.popup.is_some() {
            // Handle popup input
            self.popup = None; // Reassign the popup if needed
            return true;
        }
        if self.window_stack.back_mut().is_some()
            && self.window_stack.back().unwrap().capture_all_input()
        {
            // If there's a current window, check if it captures all input
            reqs = self.window_stack.back_mut().unwrap().handle_input(key);
        } else {
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
                        reqs = current.handle_input(key);
                    }
                }
            }
        }
        if let Some(reqs) = reqs {
            for req in reqs {
                self.handle_request(req);
            }
        }
        true
    }

    pub fn render(&mut self, f: &mut Frame) {
        let area = f.area(); // Changed from f.size()

        // Layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area); // Changed from size

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
            let area_popup = centered_rect(30, 20, area); // Changed from size
            p.render(f, area_popup);
        }
    }

    pub fn handle_request(&mut self, req: Request) {
        match req {
            Request::Popup(popup) => {
                self.popup = Some(popup);
            }
            Request::PushWindow(new_window) => {
                // New handler
                self.window_stack.push_back(new_window);
            }
            Request::PopWindow => {
                if self.window_stack.len() > 1 {
                    self.window_stack.pop_back();
                } else {
                }
            }
        }
    }
}
