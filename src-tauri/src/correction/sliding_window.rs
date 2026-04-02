/// Sliding window buffer for maintaining context during real-time correction
pub struct SlidingWindow {
    max_chars: usize,
    buffer: String,
    locked: String,
}

impl SlidingWindow {
    pub fn new(max_chars: usize) -> Self {
        Self {
            max_chars,
            buffer: String::new(),
            locked: String::new(),
        }
    }

    pub fn push(&mut self, text: &str) {
        self.buffer.push_str(text);
        if self.buffer.len() > self.max_chars {
            let excess = self.buffer.len() - self.max_chars;
            let to_lock = self.buffer.drain(..excess).collect::<String>();
            self.locked.push_str(&to_lock);
        }
    }

    pub fn lock(&mut self) {
        self.locked.push_str(&self.buffer);
        self.buffer.clear();
    }

    pub fn get_full(&self) -> String {
        format!("{}{}", self.locked, self.buffer)
    }

    pub fn get_buffer(&self) -> &str {
        &self.buffer
    }

    pub fn get_locked(&self) -> &str {
        &self.locked
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.locked.clear();
    }

    pub fn has_sentence_boundary(&self) -> bool {
        let trimmed = self.buffer.trim_end();
        trimmed.ends_with('.') || trimmed.ends_with('!') || trimmed.ends_with('?')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sliding_window_push() {
        let mut window = SlidingWindow::new(100);
        window.push("Hello world");
        assert_eq!(window.get_buffer(), "Hello world");
    }

    #[test]
    fn test_sliding_window_trims_to_locked() {
        let mut window = SlidingWindow::new(10);
        window.push("Hello world this is a test");
        assert!(window.get_buffer().len() <= 10);
        assert!(!window.get_locked().is_empty());
    }

    #[test]
    fn test_sliding_window_lock() {
        let mut window = SlidingWindow::new(100);
        window.push("Sentence one.");
        window.lock();
        assert_eq!(window.get_locked(), "Sentence one.");
        assert_eq!(window.get_buffer(), "");
    }

    #[test]
    fn test_sliding_window_sentence_boundary() {
        let mut window = SlidingWindow::new(100);
        window.push("Hello world.");
        assert!(window.has_sentence_boundary());

        window.push(" No punctuation");
        assert!(!window.has_sentence_boundary());
    }

    #[test]
    fn test_sliding_window_reset() {
        let mut window = SlidingWindow::new(100);
        window.push("Some text");
        window.lock();
        window.reset();
        assert_eq!(window.get_full(), "");
    }

    #[test]
    fn test_sliding_window_get_full() {
        let mut window = SlidingWindow::new(100);
        window.push("Locked.");
        window.lock();
        window.push("Active");
        assert_eq!(window.get_full(), "Locked.Active");
    }
}
