// windows/main_window.rs
use crate::app::Request;
use crate::popup::{Popup, PopupType};
use crate::window::Window; // Removed WindowTransition
use crate::windows::config::ConfigWindow;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};

pub struct MainWindow {
    selected: usize,
    options: Vec<String>,
}

impl MainWindow {
    pub fn new() -> Self {
        Self {
            selected: 0,
            options: vec![
                "Static analysis".into(),
                "Fuzz !".into(),
                "Config".into(),
                "Quit".into(),
            ],
        }
    }
}

impl Window for MainWindow {
    fn name(&self) -> &str {
        "Main Menu"
    }

    fn render(&mut self, f: &mut Frame, area: Rect) -> Option<Vec<Request>> {
        let ret = None;
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(40), Constraint::Min(5)])
            .split(area);

        let title = Paragraph::new("


░▒▓████████▓▒░▒▓█▓▒░       ░▒▓██████▓▒░ ░▒▓███████▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓████████▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓████████▓▒░▒▓████████▓▒░ 
░▒▓█▓▒░      ░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░      ░▒▓█▓▒░      ░▒▓█▓▒░ 
░▒▓█▓▒░      ░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░    ░▒▓██▓▒░     ░▒▓██▓▒░  
░▒▓██████▓▒░ ░▒▓█▓▒░      ░▒▓████████▓▒░░▒▓██████▓▒░░▒▓████████▓▒░▒▓██████▓▒░ ░▒▓█▓▒░░▒▓█▓▒░  ░▒▓██▓▒░     ░▒▓██▓▒░    
░▒▓█▓▒░      ░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░      ░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░░▒▓██▓▒░     ░▒▓██▓▒░      
░▒▓█▓▒░      ░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░      ░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░        
░▒▓█▓▒░      ░▒▓████████▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓███████▓▒░░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░       ░▒▓██████▓▒░░▒▓████████▓▒░▒▓████████▓▒░ 
")
            .style(
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD) ,
            )
            .alignment(Alignment::Center);
        f.render_widget(title, vertical_chunks[0]);

        let items: Vec<ListItem> = self
            .options
            .iter()
            .enumerate()
            .map(|(i, opt)| {
                let style = if i == self.selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };
                let mut txt: Text = opt.clone().into();
                txt = txt.alignment(Alignment::Center);
                ListItem::new(txt).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Options")
                .title_alignment(Alignment::Center)
                .title_style(Style::new().bold()),
        );
        f.render_widget(list, vertical_chunks[1]);
        return ret;
    }

    fn handle_input(&mut self, key: KeyEvent) -> Option<Vec<Request>> {
        let mut ret = None;
        match key.code {
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected < self.options.len() - 1 {
                    self.selected += 1;
                }
            }
            KeyCode::Enter => {
                // Future: return different transitions based on selected option
                ret = Some(vec![Request::Popup(Popup::new(
                    PopupType::Info,
                    "You selected: ".to_string() + &self.options[self.selected],
                ))]);

                if &self.options[self.selected] == "Config" {
                    // Config
                    ret = Some(vec![Request::PushWindow(Box::new(ConfigWindow::new()))]);
                }
            }
            _ => {}
        }
        return ret;
    }
}
