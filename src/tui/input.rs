use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// A tiny single-line editable buffer used by the TUI input boxes.
#[derive(Clone, Debug, Default)]
pub struct Input {
    pub buffer: String,
    pub cursor: usize,
}

impl Input {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(text: &str) -> Self {
        Self {
            buffer: text.to_string(),
            cursor: text.chars().count(),
        }
    }

    pub fn set(&mut self, text: &str) {
        self.buffer = text.to_string();
        self.cursor = self.buffer.chars().count();
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.trim().is_empty()
    }

    pub fn value(&self) -> String {
        self.buffer.trim().to_string()
    }

    pub fn handle(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                let byte = byte_index(&self.buffer, self.cursor);
                self.buffer.insert(byte, c);
                self.cursor += 1;
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    let byte = byte_index(&self.buffer, self.cursor);
                    self.buffer.remove(byte);
                }
            }
            KeyCode::Delete => {
                if self.cursor < self.buffer.chars().count() {
                    let byte = byte_index(&self.buffer, self.cursor);
                    self.buffer.remove(byte);
                }
            }
            KeyCode::Left => {
                self.cursor = self.cursor.saturating_sub(1);
            }
            KeyCode::Right => {
                if self.cursor < self.buffer.chars().count() {
                    self.cursor += 1;
                }
            }
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.buffer.chars().count(),
            _ => {}
        }
    }
}

fn byte_index(text: &str, char_idx: usize) -> usize {
    text.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(text.len())
}
