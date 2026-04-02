use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::thread;
use std::time::Duration;

pub struct TextInjector {
    enigo: Enigo,
    fallback_to_clipboard: bool,
}

impl TextInjector {
    pub fn new() -> Self {
        Self {
            enigo: Enigo::new(&Settings::default()).expect("Failed to initialize Enigo"),
            fallback_to_clipboard: true,
        }
    }

    pub fn inject(&mut self, text: &str) -> Result<(), String> {
        if text.len() > 50 {
            self.inject_via_clipboard(text)
        } else {
            self.inject_via_keystrokes(text)
        }
    }

    fn inject_via_keystrokes(&mut self, text: &str) -> Result<(), String> {
        for ch in text.chars() {
            match ch {
                ' ' => self
                    .enigo
                    .key(Key::Space, Direction::Click)
                    .map_err(|e| e.to_string())?,
                '\n' | '\r' => self
                    .enigo
                    .key(Key::Return, Direction::Click)
                    .map_err(|e| e.to_string())?,
                '\t' => self
                    .enigo
                    .key(Key::Tab, Direction::Click)
                    .map_err(|e| e.to_string())?,
                c => {
                    self.enigo
                        .text(&c.to_string())
                        .map_err(|e| format!("Failed to type '{}': {}", c, e))?;
                }
            }
            thread::sleep(Duration::from_millis(5));
        }
        Ok(())
    }

    fn inject_via_clipboard(&mut self, text: &str) -> Result<(), String> {
        arboard::Clipboard::new()
            .map_err(|e| format!("Failed to access clipboard: {}", e))?
            .set_text(text)
            .map_err(|e| format!("Failed to set clipboard text: {}", e))?;

        thread::sleep(Duration::from_millis(50));

        #[cfg(target_os = "macos")]
        {
            self.enigo
                .key(Key::Meta, Direction::Click)
                .map_err(|e| e.to_string())?;
            self.enigo
                .key(Key::Unicode('v'), Direction::Click)
                .map_err(|e| e.to_string())?;
            self.enigo
                .key(Key::Meta, Direction::Click)
                .map_err(|e| e.to_string())?;
        }

        #[cfg(not(target_os = "macos"))]
        {
            self.enigo
                .key(Key::Control, Direction::Click)
                .map_err(|e| e.to_string())?;
            self.enigo
                .key(Key::Unicode('v'), Direction::Click)
                .map_err(|e| e.to_string())?;
            self.enigo
                .key(Key::Control, Direction::Click)
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    pub fn backspace(&mut self, count: usize) -> Result<(), String> {
        for _ in 0..count {
            self.enigo
                .key(Key::Backspace, Direction::Click)
                .map_err(|e| e.to_string())?;
            thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    }
}

impl Default for TextInjector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_injector_creation() {
        let injector = TextInjector::new();
        assert!(injector.fallback_to_clipboard);
    }

    #[test]
    fn test_injector_default() {
        let injector = TextInjector::default();
        assert!(injector.fallback_to_clipboard);
    }
}
