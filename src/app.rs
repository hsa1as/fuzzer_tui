// app.rs
#![allow(dead_code)]
use crate::utils::centered_rect::centered_rect;
use crate::{
    popup::*,
    window::Window,                         // Removed WindowTransition
    windows::project_window::ProjectWindow, // Added
};
use crossterm::event::KeyEvent;
use ratatui::widgets::{Block, BorderType, Borders};
use ratatui::{prelude::*, widgets::Paragraph, Frame};
use std::any::Any;
use std::collections::{HashMap, VecDeque};

pub enum Request {
    Popup(Popup),
    PushWindow(Box<dyn Window>), // New request to push a window
    PopWindow,
    PushProperty(String, Box<dyn Any>), // New request to push a property
    GetProperty(String),                // New request to get a property
}

pub struct App {
    window_stack: VecDeque<Box<dyn Window>>,
    pub properties: HashMap<String, Box<dyn Any>>,
    popup: Option<Popup>,
}

impl App {
    pub fn new() -> Self {
        let mut stack: VecDeque<Box<dyn Window>> = VecDeque::new();
        stack.push_back(Box::new(ProjectWindow::new()));
        let mut properties: HashMap<String, Box<dyn Any>> = HashMap::new();
        properties.insert("port".into(), Box::new(1337u16) as Box<dyn Any>); // Default port
        App {
            window_stack: stack,
            properties,
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
                Constraint::Length(1),
                Constraint::Fill(1),
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
        self.render_footer(f, chunks[2]);
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
            Request::PushProperty(s, p) => {
                self.properties.insert(s, p);
            }
            Request::GetProperty(s) => {
                let p = match self.properties.get(&s) {
                    None => {
                        self.window_stack.pop_back();
                        self.popup = Some(Popup::new(
                            PopupType::Warning,
                            format!(
                                "The requested property was not found in the application state: {}",
                                s
                            ),
                        ));
                        return;
                    }
                    Some(v) => v,
                };
                let w = self.window_stack.back_mut().unwrap();
                w.send_property(s.clone(), &**p);
            }
        }
    }

    fn render_footer(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 3); 3])
            .split(area); // Changed from size

        // Current window
        let window_name = self
            .window_stack
            .back()
            .map(|w| w.name())
            .unwrap_or("Unknown Window");

        // Attempt to get the current port
        let port = self
            .properties
            .get("port")
            .and_then(|p| p.downcast_ref::<u16>())
            .map_or("1337".to_string(), |p| p.to_string());

        // Get current project name
        let project = self
            .properties
            .get("project_name")
            .and_then(|p| p.downcast_ref::<String>())
            .map_or("No project".to_string(), |p| p.clone());

        // Make paragraph widgets
        let window_para = Paragraph::new(window_name)
            .alignment(Alignment::Center)
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::ITALIC),
                    ),
            );

        let project_para = Paragraph::new(format!("Project: {}", project))
            .alignment(Alignment::Center)
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(
                        Style::default()
                            .fg(Color::Indexed(6))
                            .add_modifier(Modifier::ITALIC),
                    ),
            );

        let port_para = Paragraph::new(format!("Port: {}", port))
            .alignment(Alignment::Center)
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(
                        Style::default()
                            .fg(Color::Indexed(6))
                            .add_modifier(Modifier::ITALIC),
                    ),
            );

        // Render
        f.render_widget(project_para, chunks[0]);
        f.render_widget(window_para, chunks[1]);
        f.render_widget(port_para, chunks[2]);
    }
}
