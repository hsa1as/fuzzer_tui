#![allow(dead_code)]
#![allow(unused)]
use std::path::PathBuf;

use ratatui::style::{Modifier, Stylize};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem};

use crate::app::Request;
use crate::window::Window;

#[cfg(feature = "for_fuzzer")]
use flashfuzzemu::opts::EmuOpts;

#[derive(PartialEq)]
pub enum ConfigWindowState {
    Main,
    ManualConfig,
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
        Self {
            state: ConfigWindowState::Main,
            config_file: None,
            config_json: None,
            selected_idx: 0,
            options: vec!["From script".into(), "Manual configuration".into()],
            config_options: vec![],
        }
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
                .title("Configuration Options"),
        );
        f.render_widget(list, area);
        None
    }

    fn render_manual(
        &mut self,
        f: &mut ratatui::prelude::Frame,
        area: ratatui::prelude::Rect,
    ) -> Option<Vec<Request>> {
        // Placeholder for manual configuration rendering
        f.render_widget(Clear, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Manual Configuration")
            .title_style(ratatui::style::Style::default().fg(ratatui::style::Color::Green));
        f.render_widget(block, area);
        None
    }
}

impl Window for ConfigWindow {
    fn capture_all_input(&self) -> bool {
        match self.state {
            ConfigWindowState::Main => false,
            ConfigWindowState::ManualConfig => true,
        }
    }
    fn name(&self) -> &str {
        match self.state {
            ConfigWindowState::Main => "Configuration Menu",
            ConfigWindowState::ManualConfig => "Configuration Menu: Manual config",
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
        }
    }

    fn handle_input(&mut self, key: crossterm::event::KeyEvent) -> Option<Vec<Request>> {
        let mut ret = None;
        if (self.state == ConfigWindowState::Main) {
            match key.code {
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
                }
                _ => {}
            }
        }
        if (self.state == ConfigWindowState::ManualConfig) {
            match key.code {
                crossterm::event::KeyCode::Esc => self.state = ConfigWindowState::Main,
                _ => {}
            }
        }
        return ret;
    }
}
