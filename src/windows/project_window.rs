use crate::utils::centered_rect::{self, centered_rect};
use crate::utils::file_dialogue::{FileDialogue, FileDialogueResult};
use crate::utils::input_dialogue::InputDialogue; // Removed ExplorerInput
use crate::window::Window;
use crate::windows::main_window::MainWindow;
use crate::{app::Request, utils::input_dialogue::InputDialogueResult};

use crossterm::event::KeyEvent;

use ratatui::{prelude::*, widgets::*};

use std::any::Any;
use std::fs;
use std::path::Path;

#[cfg(feature = "for_fuzzer")]
use flashfuzzemu::opts::EmuOpts;
#[cfg(feature = "for_fuzzer")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "for_fuzzer")]
use serde_json;

enum ProjectWindowState<'a> {
    SelectingAction(ListState),
    BrowsingForCreate(Option<InputDialogue<'a>>),
    BrowsingForOpen,
}

pub struct ProjectWindow<'a> {
    options: Vec<String>,
    state: ProjectWindowState<'a>,
    explorer: FileDialogue,
}

impl<'a> ProjectWindow<'a> {
    pub fn new() -> Self {
        Self {
            options: vec![
                "Create New Project".to_string(),
                "Open Existing Project".to_string(),
            ],
            state: ProjectWindowState::SelectingAction(ListState::default().with_selected(Some(0))),
            explorer: FileDialogue::new(),
        }
    }

    fn set_current_dir(&mut self, path: &Path) -> Option<Vec<Request>> {
        match std::env::set_current_dir(&path) {
            Ok(_) => {
                return Some(vec![
                    Request::PushProperty(
                        "project_name".into(),
                        Box::new(String::from(path.file_name().unwrap().to_str().unwrap()))
                            as Box<dyn Any>,
                    ),
                    Request::PopWindow,
                    Request::PushWindow(Box::new(MainWindow::new())),
                    Request::Popup(crate::popup::Popup::new(
                        crate::popup::PopupType::Success,
                        format!("Project opened successfully at: {}", path.display()),
                    )),
                ]);
            }
            Err(e) => {
                return Some(vec![Request::Popup(crate::popup::Popup::new(
                    crate::popup::PopupType::Warning,
                    format!("Failed to set current directory: {}", e),
                ))]);
            }
        };
    }

    fn render_main(&mut self, f: &mut Frame, area: Rect) -> Option<Vec<Request>> {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(9),
                Constraint::Fill(1),
                Constraint::Min(3),
            ])
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
        f.render_widget(title, chunks[1]);

        let project_options_area = chunks[3];
        let selected_index = match self.state {
            ProjectWindowState::SelectingAction(ref l) => l.selected().unwrap_or(0),
            _ => 0,
        };
        let items: Vec<ListItem> = self
            .options
            .iter()
            .enumerate()
            .map(|(i, opt)| {
                let style = if i == selected_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(opt.as_str()).alignment(Alignment::Center)).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Select Option")
                    .title_alignment(Alignment::Center),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");
        let liststate = match self.state {
            ProjectWindowState::SelectingAction(ref mut i) => i,
            _ => &mut ListState::default(),
        };
        f.render_stateful_widget(list, project_options_area, liststate);
        None
    }

    fn render_browser(&mut self, f: &mut Frame, area: Rect) -> Option<Vec<Request>> {
        let cr = centered_rect::centered_rect(60, 60, area);
        self.explorer.render(f, cr);
        None
    }
}

impl<'a> Window for ProjectWindow<'a> {
    fn name(&self) -> &str {
        match self.state {
            ProjectWindowState::SelectingAction(_) => "Project Setup",
            ProjectWindowState::BrowsingForCreate(ref v) => {
                if v.is_some() {
                    "Create Project - Enter project name"
                } else {
                    "Create Project - Select Directory"
                }
            }
            ProjectWindowState::BrowsingForOpen => "Open Project - Select Directory",
        }
    }

    fn render(&mut self, f: &mut Frame, area: Rect) -> Option<Vec<Request>> {
        self.render_main(f, area);
        match self.state {
            ProjectWindowState::SelectingAction(_) => {
                return None;
            }
            _ => {}
        }
        self.render_browser(f, area);
        match self.state {
            ProjectWindowState::BrowsingForCreate(ref mut v) => {
                if v.is_some() {
                    let centered_area = centered_rect(30, 20, area);
                    v.as_mut().unwrap().render(f, centered_area);
                }
            }
            _ => {}
        }
        None
    }

    fn handle_input(&mut self, key: KeyEvent) -> Option<Vec<Request>> {
        // key is crossterm::event::KeyEvent
        match self.state {
            ProjectWindowState::SelectingAction(ref mut l) => match key.code {
                crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('k') => {
                    l.select_previous();
                }
                crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('j') => {
                    l.select_next();
                }
                crossterm::event::KeyCode::Enter => {
                    let selected_option = &self.options[l.selected().unwrap_or(0)];
                    match selected_option.as_str() {
                        "Create New Project" => {
                            self.state = ProjectWindowState::BrowsingForCreate(None)
                        }
                        "Open Existing Project" => self.state = ProjectWindowState::BrowsingForOpen,
                        _ => {}
                    }
                }
                _ => {}
            },
            ProjectWindowState::BrowsingForCreate(ref mut v) => {
                if v.is_some() {
                    let ret = v.as_mut().unwrap().handle_input(key);
                    match ret {
                        InputDialogueResult::Continue => {}
                        InputDialogueResult::Cancel => {
                            self.state = ProjectWindowState::BrowsingForCreate(None)
                        }
                        InputDialogueResult::Submit(str) => {
                            // create a directory called str in the current directory of the
                            // ratatui_explorer and setup project inside of it
                            let project_path_buf = self.explorer.get_selected();
                            let new_project_path = project_path_buf.join(str);
                            match create_project_structure(&new_project_path) {
                                Ok(_) => {
                                    return self.set_current_dir(&new_project_path);
                                }
                                Err(e) => {
                                    return Some(vec![Request::Popup(crate::popup::Popup::new(
                                        crate::popup::PopupType::Warning,
                                        format!("Failed to create project: {}", e),
                                    ))]);
                                }
                            };
                        }
                    }
                } else {
                    match self.explorer.handle_input(key) {
                        FileDialogueResult::Continue => {}
                        FileDialogueResult::Select(p) => {
                            // check if p is directory
                            if p.is_file() {
                                return Some(vec![Request::Popup(crate::popup::Popup::new(
                                    crate::popup::PopupType::Info,
                                    "Please select a directory for project operations.".to_string(),
                                ))]);
                            }
                            if p.is_dir() {
                                self.state =
                                    ProjectWindowState::BrowsingForCreate(Some(InputDialogue::new(
                                        "Create New Project".to_string(),
                                        "Enter project name:".to_string(),
                                    )))
                            }
                        }
                        FileDialogueResult::Cancel => {
                            self.state = ProjectWindowState::SelectingAction(
                                ListState::default().with_selected(Some(0)),
                            );
                        }
                    }
                }
            }
            ProjectWindowState::BrowsingForOpen => {
                // key is crossterm::event::KeyEvent.
                // ratatui_explorer expects ratatui::crossterm::event::Event.
                match self.explorer.handle_input(key) {
                    FileDialogueResult::Cancel => {
                        self.state = ProjectWindowState::SelectingAction(
                            ListState::default().with_selected(Some(0)),
                        );
                        return None;
                    }
                    FileDialogueResult::Continue => {}
                    FileDialogueResult::Select(p) => {
                        // check if p is directory
                        if p.is_file() {
                            return Some(vec![Request::Popup(crate::popup::Popup::new(
                                crate::popup::PopupType::Info,
                                "Please select a directory for project operations.".to_string(),
                            ))]);
                        }
                        if p.is_dir() {
                            // Ensure selection is a directory for these actions
                            match validate_project_structure(&p) {
                                Ok(_) => return self.set_current_dir(&p),
                                Err(e) => {
                                    return Some(vec![Request::Popup(crate::popup::Popup::new(
                                        crate::popup::PopupType::Warning,
                                        format!("Invalid project: {}", e),
                                    ))])
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }
    fn capture_all_input(&self) -> bool {
        // This window captures all input events
        match self.state {
            ProjectWindowState::SelectingAction(_) => false,
            _ => true,
        }
    }
}

pub fn create_project_structure(project_path: &Path) -> Result<(), String> {
    let corpus_path = project_path.join("corpus");
    let crashes_path = project_path.join("crashes");
    let config_file = project_path.join("config.json");
    let grammar_file = project_path.join("grammar.json");

    fs::create_dir_all(&corpus_path)
        .map_err(|e| format!("Failed to create corpus directory: {}", e))?;
    fs::create_dir_all(&crashes_path)
        .map_err(|e| format!("Failed to create crashes directory: {}", e))?;
    fs::File::create(&config_file).map_err(|e| format!("Failed to create config.json: {}", e))?;
    fs::File::create(&grammar_file).map_err(|e| format!("Failed to create grammar.json: {}", e))?;
    #[cfg(feature = "for_fuzzer")]
    {
        // create a dummy config.json using emuopts defaults
        let write_str =
            serde_json::to_string_pretty(&EmuOpts::default()).map_err(|e| e.to_string())?;
        fs::write(&config_file, write_str).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn validate_project_structure(project_path: &Path) -> Result<(), String> {
    let corpus_path = project_path.join("corpus");
    let crashes_path = project_path.join("crashes");
    let config_file = project_path.join("config.json");
    let grammar_file = project_path.join("grammar.json");

    if !corpus_path.is_dir() {
        return Err("Corpus directory not found.".to_string());
    }
    if !crashes_path.is_dir() {
        return Err("Crashes directory not found.".to_string());
    }
    if !config_file.is_file() {
        return Err("config.json not found.".to_string());
    }
    if !grammar_file.is_file() {
        return Err("grammar.json not found.".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::tempdir;

    #[test]
    fn test_create_project_structure_success() {
        let dir = tempdir().unwrap();
        let project_path = dir.path().join("test_project");
        fs::create_dir(&project_path).unwrap();

        let result = create_project_structure(&project_path);
        assert!(result.is_ok(), "Should succeed: {:?}", result.err());
        assert!(project_path.join("corpus").is_dir());
        assert!(project_path.join("crashes").is_dir());
        assert!(project_path.join("config.json").is_file());
        assert!(project_path.join("grammar.json").is_file());
    }

    #[test]
    fn test_validate_project_structure_success() {
        let dir = tempdir().unwrap();
        let project_path = dir.path().join("valid_project");
        fs::create_dir_all(&project_path.join("corpus")).unwrap();
        fs::create_dir_all(&project_path.join("crashes")).unwrap();
        File::create(&project_path.join("config.json")).unwrap();
        File::create(&project_path.join("grammar.json")).unwrap();

        let result = validate_project_structure(&project_path);
        assert!(result.is_ok(), "Should be valid: {:?}", result.err());
    }

    #[test]
    fn test_validate_project_structure_missing_corpus() {
        let dir = tempdir().unwrap();
        let project_path = dir.path().join("invalid_project");
        fs::create_dir_all(&project_path).unwrap();
        fs::create_dir_all(&project_path.join("crashes")).unwrap();
        File::create(&project_path.join("config.json")).unwrap();
        File::create(&project_path.join("grammar.json")).unwrap();

        let result = validate_project_structure(&project_path);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Corpus directory not found.");
    }

    #[test]
    fn test_validate_project_structure_missing_config_file() {
        let dir = tempdir().unwrap();
        let project_path = dir.path().join("invalid_project_config");
        fs::create_dir_all(&project_path.join("corpus")).unwrap();
        fs::create_dir_all(&project_path.join("crashes")).unwrap();
        File::create(&project_path.join("grammar.json")).unwrap();

        let result = validate_project_structure(&project_path);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "config.json not found.");
    }
}
