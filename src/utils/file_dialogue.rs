use std::path::PathBuf;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{palette::tailwind::SLATE, Color, Modifier, Style, Stylize},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, ListState, Padding, Paragraph,
        StatefulWidget, Widget,
    },
    Frame,
};

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

pub struct FileDialogue {
    current_list: Vec<FileType>,
    cwd: PathBuf,
    state: ListState,
}

pub enum FileType {
    File(String),
    Directory(String),
}

impl Into<String> for &FileType {
    fn into(self) -> String {
        match self {
            FileType::File(name) => format!("{}", name),
            FileType::Directory(name) => format!("{}", name),
        }
    }
}

pub enum FileDialogueResult {
    Continue,
    Select(PathBuf),
    Cancel,
}

impl FileDialogue {
    pub fn new() -> Self {
        FileDialogue::with_cwd(std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")))
    }

    pub fn with_cwd(cwd: PathBuf) -> Self {
        let mut ret = Self {
            current_list: Vec::new(),
            cwd,
            state: ListState::default().with_selected(Some(1)),
        };
        ret.get_list();
        ret
    }

    fn get_list(&mut self) {
        let mut list = Vec::new();
        list.push(FileType::Directory("../".to_string())); // Add parent directory option
        if let Ok(entries) = std::fs::read_dir(&self.cwd) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    match entry.file_type() {
                        Ok(file_type) if file_type.is_dir() => {
                            list.push(FileType::Directory(name.to_string()));
                        }
                        Ok(_) => {
                            list.push(FileType::File(name.to_string()));
                        }
                        Err(_) => continue, // Skip entries that cannot be read
                    }
                }
            }
        }
        self.current_list = list;
    }

    pub fn get_selected(&mut self) -> PathBuf {
        let selected_idx = self.state.selected().unwrap_or(0);
        if let Some(selected) = self.current_list.get(selected_idx) {
            let mut file = self.cwd.clone();
            file.push(Into::<String>::into(selected));
            return file;
        }
        self.cwd.clone() // Fallback to current directory if nothing is selected
    }

    fn select_current_dir(&mut self) {
        let selected_idx = self.state.selected().unwrap_or(0);
        if let Some(selected) = self.current_list.get(selected_idx) {
            if let FileType::Directory(name) = selected {
                if name == "../" {
                    // Go up to parent directory
                    self.cwd.pop();
                } else {
                    // Go into the selected directory
                    self.cwd.push(name);
                }
                self.get_list();
            }
        }
    }

    pub fn handle_input(&mut self, key: crossterm::event::KeyEvent) -> FileDialogueResult {
        match key.code {
            crossterm::event::KeyCode::Char(c) => {
                // vim keybindings
                match c {
                    'j' => {
                        self.state.select_next();
                    }
                    'k' => {
                        self.state.select_previous();
                    }
                    'h' => {
                        // go to parent directory
                        if !self.cwd.pop() {
                            self.cwd = PathBuf::from("/");
                        }
                        self.get_list();
                    }
                    'l' => {
                        // go into currently selected directory
                        self.select_current_dir();
                    }
                    'q' => {
                        // return
                        return FileDialogueResult::Cancel;
                    }
                    'g' => {
                        // go to top
                        self.state.select_first();
                    }
                    'G' => {
                        // go to bottom
                        self.state.select_last();
                    }
                    _ => {}
                }
            }

            crossterm::event::KeyCode::Left => {
                // go to parent directory
                if !self.cwd.pop() {
                    self.cwd = PathBuf::from("/");
                }
                self.get_list();
            }
            crossterm::event::KeyCode::Right => {
                // go into currently selected directory
                self.select_current_dir();
            }
            crossterm::event::KeyCode::Down => {
                self.state.select_next();
            }
            crossterm::event::KeyCode::Up => {
                self.state.select_previous();
            }
            crossterm::event::KeyCode::Enter => {
                let selected_idx = self.state.selected().unwrap_or(0);
                if let Some(selected) = self.current_list.get(selected_idx) {
                    let mut file = self.cwd.clone();
                    file.push(Into::<String>::into(selected));
                    return FileDialogueResult::Select(file);
                }
            }
            crossterm::event::KeyCode::Esc => {}
            _ => {}
        };
        return FileDialogueResult::Continue;
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let selected_idx = self.state.selected().unwrap_or(0);
        f.render_widget(Clear, area);
        let title = format!("{}", self.cwd.display());
        let block = Block::default()
            .title(title)
            .title_style(Style::new().blue())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center)
            .padding(Padding::new(1, 1, 1, 1));
        let items = self
            .current_list
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let mut style = match item {
                    FileType::Directory(_) => Style::default().fg(Color::Blue),
                    FileType::File(_) => Style::default().fg(Color::White),
                };
                if i == selected_idx {
                    style = style.add_modifier(Modifier::REVERSED);
                }
                ListItem::from(Into::<String>::into(item)).style(style)
            })
            .collect::<Vec<_>>();
        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE);
        f.render_stateful_widget(list, area, &mut self.state);
    }
}
