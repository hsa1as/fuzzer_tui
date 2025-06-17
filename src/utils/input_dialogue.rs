use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct InputDialogue {
    pub title: String,
    pub prompt: String,
    pub input: String,
    pub current_pos: usize,
}

pub enum InputDialogueResult {
    Continue,
    Submit(String),
    Cancel,
}

impl InputDialogue {
    pub fn new(title: String, prompt: String) -> Self {
        InputDialogue {
            title,
            prompt,
            input: String::new(),
            current_pos: 0,
        }
    }
    pub fn handle_input(&mut self, key: crossterm::event::KeyEvent) -> InputDialogueResult {
        match key.code {
            crossterm::event::KeyCode::Char(c) => {
                self.input.insert(self.current_pos, c);
                self.current_pos += 1;
            }
            crossterm::event::KeyCode::Backspace => {
                if self.current_pos > 0 {
                    self.input.remove(self.current_pos - 1);
                    self.current_pos -= 1;
                }
            }
            crossterm::event::KeyCode::Left => {
                if self.current_pos > 0 {
                    self.current_pos -= 1;
                }
            }
            crossterm::event::KeyCode::Right => {
                if self.current_pos < self.input.len() {
                    self.current_pos += 1;
                }
            }
            crossterm::event::KeyCode::Enter => {
                return InputDialogueResult::Submit(self.input.clone());
            }
            crossterm::event::KeyCode::Esc => {
                return InputDialogueResult::Cancel;
            }
            _ => {}
        };

        return InputDialogueResult::Continue;
    }

    pub fn render(&mut self, f: &mut Frame, popup_area: Rect) {
        // Further divide the middle section (actual popup) vertically
        let content_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20), // Prompt
                Constraint::Percentage(80), // Input
            ])
            .margin(1)
            .split(popup_area);
        let mut input_with_cursor = self.input.clone();
        input_with_cursor.insert(self.current_pos.min(input_with_cursor.len()), '|');

        // Prompt paragraph
        let prompt_para = Paragraph::new(self.prompt.as_str())
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Left);

        // Input paragraph with cursor
        let input_para = Paragraph::new(input_with_cursor.as_str())
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left)
            .block(Block::default().borders(Borders::ALL));

        // Wrap everything in a Block
        let block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black).fg(Color::White));

        // Render the block
        f.render_widget(block, popup_area);

        // Render the prompt and input
        f.render_widget(prompt_para, content_layout[0]);
        f.render_widget(input_para, content_layout[1]);
    }
}
