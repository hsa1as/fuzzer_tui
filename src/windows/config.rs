#![allow(dead_code)]
#![allow(unused)]
use std::fs::read_to_string;
use std::path::PathBuf;

use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph};

use tui_textarea::TextArea;

use crate::app::Request;
use crate::popup::{Popup, PopupType};
use crate::utils::centered_rect;
use crate::utils::file_dialogue::{FileDialogue, FileDialogueResult};
use crate::utils::input_dialogue::{self, InputDialogue, InputDialogueResult};
use crate::window::Window;

#[cfg(feature = "for_fuzzer")]
use flashfuzzemu::opts::EmuOpts;

pub enum ConfigWindowState<'a> {
    Main(ListState),
    ManualConfig,
    FromScript(FileDialogue),
    SelectBinary(FileDialogue),
    SetPort(InputDialogue<'a>),
}

pub struct ConfigWindow<'a> {
    state: ConfigWindowState<'a>,
    config_file: Option<PathBuf>,
    config_tx: TextArea<'static>,
    options: Vec<String>,
}

impl<'a> ConfigWindow<'a> {
    pub fn new() -> Self {
        let mut config_tx = TextArea::default();
        let tx_block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Configuration JSON")
            .title_style(Style::default().fg(Color::Red))
            .border_style(Style::default().fg(Color::Red))
            .style(Style::default().fg(Color::White).bg(Color::Black));

        config_tx.set_block(tx_block);
        config_tx.insert_str("Configuration not read yet!");
        let mut ret = Self {
            state: ConfigWindowState::Main(ListState::default().with_selected(Some(0))),
            config_file: None,
            config_tx,
            options: vec![
                "From script".into(),
                "Manual configuration".into(),
                "Select Binary".into(),
                "Set Port".into(),
                "Done".into(),
            ],
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
        let config_json = match read_to_string(self.config_file.as_ref().unwrap()) {
            Ok(s) => s,
            Err(e) => format!("Error reading config file: {}", e),
        };
        let mut config_tx = TextArea::default();
        config_tx.insert_str(&config_json);
        self.config_tx = config_tx;
        self.update_config_block();
    }

    pub fn update_config_block(&mut self) {
        let json_str = self.config_tx.lines().join("\n");
        let block = get_tx_block(&json_str);
        self.config_tx.set_block(block);
    }
    fn render_main(
        &mut self,
        f: &mut ratatui::prelude::Frame,
        area: ratatui::prelude::Rect,
    ) -> Option<Vec<Request>> {
        f.render_widget(Clear, area);
        let selected_idx = match self.state {
            ConfigWindowState::Main(ref l) => l.selected().unwrap_or(0),
            _ => 0,
        };
        let items: Vec<ListItem> = self
            .options
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let style = if i == selected_idx {
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
            .horizontal_margin(1)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        f.render_widget(list, horizontal_chunks[0]);
        f.render_widget(&self.config_tx, horizontal_chunks[1]);
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

    fn render_select_binary(
        &mut self,
        f: &mut ratatui::prelude::Frame,
        area: ratatui::prelude::Rect,
    ) -> Option<Vec<Request>> {
        let mut ret = self.render_main(f, area);
        let centered = centered_rect::centered_rect(60, 60, area);
        if let ConfigWindowState::SelectBinary(ref mut file_dialogue) = self.state {
            f.render_widget(Clear, centered);
            file_dialogue.render(f, centered);
        }
        // display file dialogue
        return ret;
    }

    fn render_set_port(
        &mut self,
        f: &mut ratatui::prelude::Frame,
        area: ratatui::prelude::Rect,
    ) -> Option<Vec<Request>> {
        let mut ret = self.render_main(f, area);
        let centered = centered_rect::centered_rect(60, 60, area);
        if let ConfigWindowState::SetPort(ref mut input_dialogue) = self.state {
            f.render_widget(Clear, centered);
            input_dialogue.render(f, centered);
        }
        // display input dialogue
        return ret;
    }

    fn handle_input_manual(&mut self, key: crossterm::event::KeyEvent) -> Option<Vec<Request>> {
        match key.code {
            crossterm::event::KeyCode::Esc => {
                self.state = ConfigWindowState::Main(ListState::default().with_selected(Some(0)))
            }
            _ => {
                self.config_tx.input(key);
            }
        }
        self.update_config_block();

        return None;
    }

    fn save_config(&mut self) -> Result<String, String> {
        let json_str = self.config_tx.lines().join("\n");
        match validate_json(&json_str) {
            Ok(_) => {
                if let Some(ref config_file) = self.config_file {
                    if let Err(e) = std::fs::write(config_file, json_str) {
                        return Err(format!("Error writing config file: {}", e));
                    }
                    return Ok(format!("Configuration saved to {}", config_file.display()));
                } else {
                    return Err(format!("Config file not set:"));
                }
            }
            Err(e) => {
                return Err(format!("Invalid JSON configuration: {}", e));
            }
        }
    }
}

impl<'a> Window for ConfigWindow<'a> {
    fn capture_all_input(&self) -> bool {
        match self.state {
            ConfigWindowState::Main(_) => false,
            ConfigWindowState::ManualConfig => true,
            ConfigWindowState::FromScript(_) => true,
            ConfigWindowState::SelectBinary(_) => true,
            ConfigWindowState::SetPort(_) => true,
        }
    }
    fn name(&self) -> &str {
        match self.state {
            ConfigWindowState::Main(_) => "Configuration Menu",
            ConfigWindowState::ManualConfig => "Configuration Menu: Manual config",
            ConfigWindowState::FromScript(_) => "Configuration Menu: From Script",
            ConfigWindowState::SelectBinary(_) => "Configuration Menu: Selecting Binary",
            ConfigWindowState::SetPort(_) => "Configuration Menu: Selecting port",
        }
    }
    fn render(
        &mut self,
        f: &mut ratatui::prelude::Frame,
        area: ratatui::prelude::Rect,
    ) -> Option<Vec<Request>> {
        match self.state {
            ConfigWindowState::Main(_) | ConfigWindowState::ManualConfig => {
                self.render_main(f, area)
            }
            ConfigWindowState::FromScript(_) => self.render_from_script(f, area),
            ConfigWindowState::SelectBinary(_) => self.render_select_binary(f, area),
            ConfigWindowState::SetPort(_) => self.render_set_port(f, area),
        }
    }

    fn handle_input(&mut self, key: crossterm::event::KeyEvent) -> Option<Vec<Request>> {
        let mut ret = None;
        match self.state {
            ConfigWindowState::Main(ref mut l) => match key.code {
                crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('j') => {
                    l.select_previous();
                }
                crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('k') => {
                    l.select_next();
                }
                crossterm::event::KeyCode::Enter => {
                    let selected_idx = l.selected().unwrap_or(0);
                    if &self.options[selected_idx] == "Manual configuration" {
                        self.state = ConfigWindowState::ManualConfig;
                        return None;
                    }
                    if &self.options[selected_idx] == "From script" {
                        self.state = ConfigWindowState::FromScript(FileDialogue::new());
                        return None;
                    }
                    if &self.options[selected_idx] == "Select Binary" {
                        self.state = ConfigWindowState::SelectBinary(FileDialogue::new());
                        return None;
                    }
                    if &self.options[selected_idx] == "Set Port" {
                        self.state = ConfigWindowState::SetPort(InputDialogue::new(
                            "Set Port".to_string(),
                            "Enter the port number:".to_string(),
                        ));
                        return None;
                    }
                    if &self.options[selected_idx] == "Done" {
                        let mut ret: Option<Vec<Request>> = None;
                        let mut ret_vec = vec![];
                        match self.save_config() {
                            Ok(msg) => {
                                ret_vec.push(Request::Popup(Popup::new(PopupType::Info, msg)));
                            }
                            Err(msg) => {
                                ret =
                                    Some(vec![Request::Popup(Popup::new(PopupType::Warning, msg))]);
                            }
                        }
                        #[cfg(feature = "for_fuzzer")]
                        {
                            println!("{}", self.config_tx.lines().join("\n"));
                            let opts = EmuOpts::from_json(self.config_tx.lines().join("\n"));
                            if let Ok(opts) = opts {
                                ret_vec.push(Request::PushProperty(
                                    "emu_opts".to_string(),
                                    Box::new(opts) as Box<dyn std::any::Any>,
                                ));
                            } else {
                                ret = Some(vec![Request::Popup(Popup::new(
                                    PopupType::Warning,
                                    "Invalid EmuOpts configuration".to_string(),
                                ))]);
                            }
                        }
                        if ret.is_some() {
                            return ret;
                        } else {
                            ret_vec.push(Request::PopWindow);
                            return Some(ret_vec);
                        }
                    }
                }
                _ => {}
            },
            ConfigWindowState::ManualConfig => {
                self.handle_input_manual(key);
            }

            ConfigWindowState::FromScript(ref mut file_dialogue) => {
                match file_dialogue.handle_input(key) {
                    FileDialogueResult::Continue => {}
                    FileDialogueResult::Select(input) => {
                        self.state =
                            ConfigWindowState::Main(ListState::default().with_selected(Some(0)));
                        // Go back to main after submission
                    }
                    FileDialogueResult::Cancel => {
                        self.state =
                            ConfigWindowState::Main(ListState::default().with_selected(Some(0)));
                        // Go back to main after cancel
                    }
                }
            }

            ConfigWindowState::SelectBinary(ref mut file_dialogue) => {
                match file_dialogue.handle_input(key) {
                    FileDialogueResult::Continue => {}
                    FileDialogueResult::Select(input) => {
                        self.state =
                            ConfigWindowState::Main(ListState::default().with_selected(Some(0)));
                        // Go back to main after submission
                        ret = Some(vec![Request::PushProperty(
                            "binary_path".to_string(),
                            Box::new(input) as Box<dyn std::any::Any>,
                        )]);
                    }
                    FileDialogueResult::Cancel => {
                        self.state =
                            ConfigWindowState::Main(ListState::default().with_selected(Some(0)));
                        // Go back to main after cancel
                    }
                }
            }
            ConfigWindowState::SetPort(ref mut input_dialogue) => {
                match input_dialogue.handle_input(key) {
                    InputDialogueResult::Continue => {}
                    InputDialogueResult::Submit(input) => {
                        // Handle port submission
                        if let Ok(port) = input.parse::<u16>() {
                            ret = Some(vec![Request::PushProperty(
                                "port".to_string(),
                                Box::new(port) as Box<dyn std::any::Any>,
                            )]);
                        } else {
                            ret = Some(vec![Request::Popup(Popup::new(
                                PopupType::Warning,
                                "Invalid port number".to_string(),
                            ))]);
                        }
                        self.state =
                            ConfigWindowState::Main(ListState::default().with_selected(Some(0)));
                    }
                    InputDialogueResult::Cancel => {
                        self.state =
                            ConfigWindowState::Main(ListState::default().with_selected(Some(0)));
                    }
                }
            }
        }
        return ret;
    }
}

pub fn validate_json(json_str: &str) -> Result<serde_json::Value, serde_json::Error> {
    // Simple validation for JSON format
    serde_json::from_str::<serde_json::Value>(json_str)
}

pub fn get_tx_block<'a, 'b>(json_str: &'b str) -> Block<'a> {
    // Return block based on whether json is valid
    let valid_json = validate_json(json_str);

    if (valid_json.is_ok()) {
        Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Configuration JSON")
            .title_style(Style::default().fg(Color::Indexed(2)))
            .border_style(Style::default().fg(Color::Indexed(2)))
            .style(Style::default().fg(Color::White).bg(Color::Black))
    } else {
        let err: String = valid_json.err().unwrap().to_string();
        Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(format!("Configuration JSON - {}", err))
            .title_style(Style::default().fg(Color::Indexed(88)))
            .border_style(Style::default().fg(Color::Indexed(88)))
            .style(Style::default().fg(Color::White).bg(Color::Black))
    }
}
