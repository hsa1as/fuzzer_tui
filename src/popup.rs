use crate::app::Request;
use crate::window::Window; // Removed WindowTransition
use crossterm::event::KeyEvent;
use ratatui::widgets::{BorderType, Clear, Wrap};
use ratatui::{
    // backend::Backend, // Removed
    layout::{Alignment, Rect}, // Removed Constraint, Direction, Layout
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[derive(Debug, Clone, Copy)]
pub enum PopupType {
    Info,
    Warning,
    Success,
}

impl PopupType {
    pub fn title(&self) -> &'static str {
        match self {
            PopupType::Info => "Info",
            PopupType::Warning => "Warning",
            PopupType::Success => "Success",
        }
    }

    pub fn style(&self) -> ratatui::style::Style {
        // Removed redundant use statement: use ratatui::style::{Color, Modifier, Style};
        match self {
            PopupType::Info => Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            PopupType::Warning => Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            PopupType::Success => Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        }
    }
}

pub struct Popup {
    popup_type: PopupType,
    message: String,
}

impl Popup {
    pub fn new(popup_type: PopupType, message: impl Into<String>) -> Self {
        Self {
            popup_type,
            message: message.into(),
        }
    }
}

impl Window for Popup {
    fn name(&self) -> &str {
        self.popup_type.title()
    }

    fn render(&mut self, f: &mut Frame, popup_area: Rect) -> Option<Vec<Request>> {
        f.render_widget(Clear, popup_area);
        let ret = None;
        let block = Block::default()
            .title(self.popup_type.title())
            .title_style(self.popup_type.style())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(Color::Black).fg(Color::White));

        let paragraph = Paragraph::new(self.message.as_str())
            .wrap(Wrap { trim: true })
            .block(block)
            .alignment(Alignment::Center);

        f.render_widget(paragraph, popup_area);
        ret
    }

    fn handle_input(&mut self, _key: KeyEvent) -> Option<Vec<Request>> {
        // always close
        Some(vec![Request::PopWindow])
    }
}
