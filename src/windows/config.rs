#![allow(dead_code)]
#![allow(unused)]
use std::fs::read_to_string;
use std::path::PathBuf;

use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph};

use crate::app::Request;
use crate::utils::centered_rect;
use crate::utils::file_dialogue::{FileDialogue, FileDialogueResult};
use crate::utils::input_dialogue::{self, InputDialogue, InputDialogueResult};
use crate::window::Window;

#[cfg(feature = "for_fuzzer")]
use flashfuzzemu::opts::EmuOpts;

pub enum ConfigWindowState {
    Main,
    ManualConfig,
    FromScript(FileDialogue),
}

pub struct ConfigWindow {
    state: ConfigWindowState,
    config_file: Option<PathBuf>,
    config_json: Option<String>,
    selected_idx: usize,
    options: Vec<String>,
    config_options: Vec<String>,
}

impl ConfigWindow {
    pub fn new() -> Self {
        let mut ret = Self {
            state: ConfigWindowState::Main,
            config_file: None,
            config_json: Some("Config string not read yet!".to_string()),
            selected_idx: 0,
            options: vec!["From script".into(), "Manual configuration".into()],
            config_options: vec![],
        };
        ret.load_config_file();
        ret.load_config_str();
        ret
    }

    pub fn load_config_file(&mut self) {
        let mut p = PathBuf::from("./");
        p.set_file_name("config.json");
        self.config_file = Some(p);
    }

    pub fn load_config_str(&mut self) {
        self.config_json = match read_to_string(self.config_file.as_ref().unwrap()) {
            Ok(s) => Some(s),
            Err(e) => Some(format!("Error reading config file: {}", e)),
        };
    }
    fn render_main(
        &mut self,
        f: &mut ratatui::prelude::Frame,
        area: ratatui::prelude::Rect,
    ) -> Option<Vec<Request>> {
        f.render_widget(Clear, area);

        let items: Vec<ListItem> = self
            .options
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let style = if i == self.selected_idx {
                    ratatui::style::Style::default()
                        .fg(ratatui::style::Color::Yellow)
                        .add_modifier(Modifier::REVERSED)
                } else {
                    ratatui::style::Style::default()
                };
                ratatui::widgets::ListItem::new(s.clone()).style(style)
            })
            .collect();
        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Configuration Options")
                .border_type(BorderType::Rounded),
        );
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        f.render_widget(list, horizontal_chunks[0]);
        let config_str = Paragraph::new(self.config_json.as_ref().unwrap().as_str())
            .style(Style::default())
            .alignment(Alignment::Left)
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            );
        f.render_widget(config_str, horizontal_chunks[1]);
        None
    }

    fn render_manual(
        &mut self,
        f: &mut ratatui::prelude::Frame,
        area: ratatui::prelude::Rect,
    ) -> Option<Vec<Request>> {
        // Placeholder for manual configuration rendering
        let chunk = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(100)])
            .split(area);
        f.render_widget(Clear, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Manual Configuration")
            .border_type(BorderType::Rounded)
            .title_style(ratatui::style::Style::default().fg(ratatui::style::Color::Green));
        f.render_widget(block, chunk[0]); // menu items
        None
    }
    fn render_from_script(
        &mut self,
        f: &mut ratatui::prelude::Frame,
        area: ratatui::prelude::Rect,
    ) -> Option<Vec<Request>> {
        let mut ret = self.render_main(f, area);
        let centered = centered_rect::centered_rect(60, 60, area);
        if let ConfigWindowState::FromScript(ref mut file_dialogue) = self.state {
            f.render_widget(Clear, centered);
            file_dialogue.render(f, centered);
        }
        // display file dialogue
        return ret;
    }
}

impl Window for ConfigWindow {
    fn capture_all_input(&self) -> bool {
        match self.state {
            ConfigWindowState::Main => false,
            ConfigWindowState::ManualConfig => true,
            ConfigWindowState::FromScript(_) => true,
        }
    }
    fn name(&self) -> &str {
        match self.state {
            ConfigWindowState::Main => "Configuration Menu",
            ConfigWindowState::ManualConfig => "Configuration Menu: Manual config",
            ConfigWindowState::FromScript(_) => "Configuration Menu: From Script",
        }
    }
    fn render(
        &mut self,
        f: &mut ratatui::prelude::Frame,
        area: ratatui::prelude::Rect,
    ) -> Option<Vec<Request>> {
        match self.state {
            ConfigWindowState::Main => self.render_main(f, area),
            ConfigWindowState::ManualConfig => self.render_manual(f, area),
            ConfigWindowState::FromScript(_) => self.render_from_script(f, area),
        }
    }

    fn handle_input(&mut self, key: crossterm::event::KeyEvent) -> Option<Vec<Request>> {
        let mut ret = None;
        match self.state {
            ConfigWindowState::Main => match key.code {
                crossterm::event::KeyCode::Up => {
                    if self.selected_idx > 0 {
                        self.selected_idx -= 1;
                    }
                }
                crossterm::event::KeyCode::Down => {
                    if self.selected_idx < self.options.len() - 1 {
                        self.selected_idx += 1;
                    }
                }
                crossterm::event::KeyCode::Enter => {
                    if self.selected_idx == 1 {
                        self.state = ConfigWindowState::ManualConfig;
                    }
                    if self.selected_idx == 0 {
                        self.state = ConfigWindowState::FromScript(FileDialogue::new());
                    }
                }
                _ => {}
            },
            ConfigWindowState::ManualConfig => match key.code {
                crossterm::event::KeyCode::Esc => self.state = ConfigWindowState::Main,
                _ => {}
            },
            ConfigWindowState::FromScript(ref mut file_dialogue) => {
                match file_dialogue.handle_input(key) {
                    FileDialogueResult::Continue => {}
                    FileDialogueResult::Select(input) => {
                        self.state = ConfigWindowState::Main; // Go back to main after submission
                    }
                    FileDialogueResult::Cancel => {
                        self.state = ConfigWindowState::Main; // Go back to main on cancel
                    }
                }
            }
        }
        return ret;
    }
}
