use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear},
    Frame,
};

use tui_textarea::TextArea;

pub struct InputDialogue<'a> {
    pub textarea: TextArea<'a>,
}

pub enum InputDialogueResult {
    Continue,
    Submit(String),
    Cancel,
}

impl<'a> InputDialogue<'a> {
    pub fn new(title: String, prompt: String) -> Self {
        // Wrap everything in a Block
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .style(Style::default().bg(Color::Black).fg(Color::White));
        let mut textarea = TextArea::default();
        textarea.set_block(block);
        textarea.set_placeholder_text(prompt);
        InputDialogue { textarea }
    }
    pub fn handle_input(&mut self, key: crossterm::event::KeyEvent) -> InputDialogueResult {
        match key.code {
            crossterm::event::KeyCode::Enter => {
                let input = self.textarea.lines().join("");
                return InputDialogueResult::Submit(input);
            }
            crossterm::event::KeyCode::Esc => {
                return InputDialogueResult::Cancel;
            }
            _ => {
                self.textarea.input(key);
            }
        };

        return InputDialogueResult::Continue;
    }

    pub fn render(&mut self, f: &mut Frame, popup_area: Rect) {
        f.render_widget(Clear, popup_area);
        f.render_widget(&self.textarea, popup_area);
    }
}
