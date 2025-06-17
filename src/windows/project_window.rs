// src/windows/project_window.rs
use crate::utils::centered_rect::centered_rect;
use crate::utils::input_dialogue::InputDialogue; // Removed ExplorerInput
use crate::window::Window;
use crate::windows::main_window::MainWindow;
use crate::{app::Request, utils::input_dialogue::InputDialogueResult};
use crossterm::event::KeyEvent; // Keep KeyEvent for our main input matching
                                // Removed: use ratatui::crossterm::event::{Event as RatatuiCrosstermEvent, KeyEvent as RatatuiKeyEvent};
                                // We will use fully qualified paths for ratatui::crossterm::event::Event to avoid import warnings if not used elsewhere.
use ratatui::{prelude::*, widgets::*};
use std::fs;
use std::path::Path; // PathBuf is unused for now, Path is sufficient for helpers
                     // Removed PathBuf from use std::path::{Path, PathBuf};
use ratatui_explorer::{FileExplorer, Theme}; // Removed ExplorerInput

// Helper functions (create_project_structure, validate_project_structure) remain unchanged
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

enum ProjectWindowState {
    SelectingAction,
    BrowsingForCreate(Option<InputDialogue>),
    BrowsingForOpen,
}

pub struct ProjectWindow {
    selected_index: usize,
    options: Vec<String>,
    state: ProjectWindowState,
    explorer: FileExplorer,
}

impl ProjectWindow {
    pub fn new() -> Self {
        let theme = Theme::default().add_default_title();
        Self {
            selected_index: 0,
            options: vec![
                "Create New Project".to_string(),
                "Open Existing Project".to_string(),
            ],
            state: ProjectWindowState::SelectingAction,
            explorer: FileExplorer::with_theme(theme).unwrap(),
        }
    }
}

impl Window for ProjectWindow {
    fn name(&self) -> &str {
        match self.state {
            ProjectWindowState::SelectingAction => "Project Setup",
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
        match self.state {
            ProjectWindowState::SelectingAction => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(30),
                        Constraint::Min(5),
                        Constraint::Percentage(30),
                    ])
                    .split(area);
                let project_options_area = chunks[1];

                let items: Vec<ListItem> = self
                    .options
                    .iter()
                    .enumerate()
                    .map(|(i, opt)| {
                        let style = if i == self.selected_index {
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::REVERSED)
                        } else {
                            Style::default()
                        };
                        ListItem::new(Line::from(opt.as_str()).alignment(Alignment::Center))
                            .style(style)
                    })
                    .collect();

                let list = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Select Option")
                            .title_alignment(Alignment::Center),
                    )
                    .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                    .highlight_symbol("> ");
                f.render_widget(list, project_options_area);
            }
            ProjectWindowState::BrowsingForOpen => {
                // Pass explorer.widget() by reference as suggested by compiler
                f.render_widget(&self.explorer.widget(), area);
            }
            ProjectWindowState::BrowsingForCreate(ref mut v) => {
                f.render_widget(&self.explorer.widget(), area);
                if v.is_some() {
                    let centered_area = centered_rect(30, 20, area);
                    v.as_mut().unwrap().render(f, centered_area);
                }
            }
        }
        None
    }

    fn handle_input(&mut self, key: KeyEvent) -> Option<Vec<Request>> {
        // key is crossterm::event::KeyEvent
        match self.state {
            ProjectWindowState::SelectingAction => match key.code {
                crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('k') => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                }
                crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('j') => {
                    if self.selected_index < self.options.len() - 1 {
                        self.selected_index += 1;
                    }
                }
                crossterm::event::KeyCode::Enter => {
                    let selected_option = &self.options[self.selected_index];
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
                            let selected_file_info =
                                &self.explorer.files()[self.explorer.selected_idx()];
                            let project_path_buf = selected_file_info.path().to_path_buf();
                            let new_project_path = project_path_buf.join(str);
                            match create_project_structure(&new_project_path) {
                                Ok(_) => {
                                    match std::env::set_current_dir(&new_project_path) {
                                        Ok(_) => {
                                            return Some(vec![
                                                Request::PopWindow,
                                                Request::PushWindow(Box::new(MainWindow::new())),
                                                Request::Popup(crate::popup::Popup::new(
                                                    crate::popup::PopupType::Success,
                                                    format!(
                                                        "Project created successfully at: {}",
                                                        new_project_path.display()
                                                    ),
                                                )),
                                            ]);
                                        }
                                        Err(e) => {
                                            return Some(vec![Request::Popup(
                                                crate::popup::Popup::new(
                                                    crate::popup::PopupType::Warning,
                                                    format!(
                                                        "Failed to set current directory: {}",
                                                        e
                                                    ),
                                                ),
                                            )]);
                                        }
                                    };
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
                    let r_event = ratatui::crossterm::event::Event::Key(key);
                    if key.code == crossterm::event::KeyCode::Enter {
                        if self.explorer.selected_idx() < self.explorer.files().len() {
                            let selected_file_info =
                                &self.explorer.files()[self.explorer.selected_idx()];
                            // For "Create New Project", we expect to select a directory where the project will be created.
                            // The selected path itself becomes the project root.
                            if selected_file_info.is_dir() {
                                // Ensure selection is a directory for these actions
                                self.state = ProjectWindowState::BrowsingForCreate(Some(
                                    InputDialogue::new(
                                        "Create Project".to_string(),
                                        "Enter project name:".to_string(),
                                    ),
                                ));
                            } else {
                                // User pressed Enter on a file, not a directory. Show a popup.
                                return Some(vec![Request::Popup(crate::popup::Popup::new(
                                    crate::popup::PopupType::Info,
                                    "Please select a directory.".to_string(),
                                ))]);
                            }
                        }
                    } else {
                        match self.explorer.handle(&r_event) {
                            Ok(()) => {}
                            Err(_e) => {
                                // std::io::Error
                                self.state = ProjectWindowState::SelectingAction;
                                return Some(vec![Request::Popup(crate::popup::Popup::new(
                                    crate::popup::PopupType::Warning,
                                    format!("File browser error: {}", _e), // Include error
                                ))]);
                            }
                        }
                    }
                }
            }
            ProjectWindowState::BrowsingForOpen => {
                // key is crossterm::event::KeyEvent.
                // ratatui_explorer expects ratatui::crossterm::event::Event.
                // ratatui::crossterm::event::KeyEvent is an alias for ::crossterm::event::KeyEvent if versions align.
                // So, key can be used directly if types are compatible.
                // The error E0277 indicates the From<&crossterm::event::Event> is not implemented for Input.
                // It expects From<&ratatui::crossterm::event::Event>.
                // So we must construct a ratatui::crossterm::event::Event.
                let r_event = ratatui::crossterm::event::Event::Key(key);
                if key.code == crossterm::event::KeyCode::Enter {
                    if self.explorer.selected_idx() < self.explorer.files().len() {
                        let selected_file_info =
                            &self.explorer.files()[self.explorer.selected_idx()];
                        // Use the method .path() as suggested by compiler (E0616)
                        let project_path_buf = selected_file_info.path().to_path_buf();
                        // For "Open Existing Project", we expect to select the project root directory.
                        if selected_file_info.is_dir() {
                            // Ensure selection is a directory for these actions
                            self.state = ProjectWindowState::SelectingAction;
                            match validate_project_structure(&project_path_buf) {
                                Ok(_) => {
                                    return Some(vec![
                                        Request::PushWindow(Box::new(MainWindow::new())),
                                        Request::Popup(crate::popup::Popup::new(
                                            crate::popup::PopupType::Success,
                                            format!(
                                                "Opened project: {}",
                                                project_path_buf.display()
                                            ),
                                        )),
                                    ])
                                }
                                Err(e) => {
                                    return Some(vec![Request::Popup(crate::popup::Popup::new(
                                        crate::popup::PopupType::Warning,
                                        format!("Invalid project: {}", e),
                                    ))])
                                }
                            }
                        } else {
                            // User pressed Enter on a file, not a directory. Show a popup.
                            return Some(vec![Request::Popup(crate::popup::Popup::new(
                                crate::popup::PopupType::Info,
                                "Please select a directory for project operations.".to_string(),
                            ))]);
                        }
                    }
                } else {
                    match self.explorer.handle(&r_event) {
                        // Pass reference to the event
                        Ok(()) => {
                            // handle returns Ok(()) on success
                            // Check for Enter key press specifically to trigger action
                        }
                        Err(_e) => {
                            // std::io::Error
                            self.state = ProjectWindowState::SelectingAction;
                            return Some(vec![Request::Popup(crate::popup::Popup::new(
                                crate::popup::PopupType::Warning,
                                format!("File browser error: {}", _e), // Include error
                            ))]);
                        }
                    }
                }

                if key.code == crossterm::event::KeyCode::Esc {
                    self.state = ProjectWindowState::SelectingAction;
                }
            }
        }
        None
    }
    fn capture_all_input(&self) -> bool {
        // This window captures all input events
        match self.state {
            ProjectWindowState::BrowsingForCreate(ref v) => v.is_some(),
            _ => false,
        }
    }
}

// Tests module remains unchanged
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
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
