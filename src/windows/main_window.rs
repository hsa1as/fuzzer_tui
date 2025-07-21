// windows/main_window.rs
use crate::app::Request;
use crate::window::Window; // Removed WindowTransition
use crate::windows::config::ConfigWindow;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};

pub struct MainWindow {
    options: Vec<String>,
    list_state: ListState,
}

impl MainWindow {
    pub fn new() -> Self {
        Self {
            options: vec![
                "Static analysis".into(),
                "Fuzz !".into(),
                "Config".into(),
                "Quit".into(),
            ],
            list_state: ListState::default().with_selected(Some(0)),
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
                    .fg(Color::Indexed(33))
                    .add_modifier(Modifier::BOLD) ,
            )
            .alignment(Alignment::Center);
        f.render_widget(title, vertical_chunks[0]);
        let selected = self.list_state.selected().unwrap_or(0);
        let items: Vec<ListItem> = self
            .options
            .iter()
            .enumerate()
            .map(|(i, opt)| {
                let style = if i == selected {
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
                .border_type(BorderType::Rounded)
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
            KeyCode::Up | KeyCode::Char('k') => {
                self.list_state.select_previous();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.list_state.select_next();
            }
            KeyCode::Enter => {
                let selected = self.list_state.selected().unwrap_or(0);
                if &self.options[selected] == "Config" {
                    // Config
                    ret = Some(vec![Request::PushWindow(Box::new(ConfigWindow::new()))]);
                }
                if &self.options[selected] == "Fuzz !" {
                    //ret = Some(vec![Request::PushWindow(Box::new(FuzzingWindow::new()))]);
                }
            }
            _ => {}
        }
        return ret;
    }
}
