use std::collections::VecDeque;

/// Manages overlapping audio chunks for streaming transcription
pub struct Chunker {
    buffer: Vec<f32>,
    chunk_size: usize,
    overlap: usize,
    ready: VecDeque<Vec<f32>>,
}

impl Chunker {
    pub fn new(sample_rate: u32, chunk_ms: u32, overlap_ms: u32) -> Self {
        let chunk_size = (sample_rate as usize * chunk_ms as usize) / 1000;
        let overlap = (sample_rate as usize * overlap_ms as usize) / 1000;
        Self {
            buffer: Vec::with_capacity(chunk_size * 2),
            chunk_size,
            overlap,
            ready: VecDeque::new(),
        }
    }

    pub fn push(&mut self, samples: &[f32]) {
        self.buffer.extend_from_slice(samples);
        while self.buffer.len() >= self.chunk_size {
            let chunk: Vec<f32> = self.buffer[..self.chunk_size].to_vec();
            self.ready.push_back(chunk);
            let _keep = self.buffer.len() - self.chunk_size + self.overlap;
            self.buffer.drain(..self.chunk_size - self.overlap);
        }
    }

    pub fn next_chunk(&mut self) -> Option<Vec<f32>> {
        self.ready.pop_front()
    }

    pub fn flush(&mut self) -> Option<Vec<f32>> {
        if self.buffer.is_empty() {
            return None;
        }
        let mut chunk = self.buffer.clone();
        chunk.resize(self.chunk_size, 0.0);
        self.buffer.clear();
        Some(chunk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunker_creates_chunks() {
        let mut chunker = Chunker::new(16000, 500, 250);
        let samples = vec![0.5f32; 16000];
        chunker.push(&samples);
        assert!(chunker.next_chunk().is_some());
        assert!(chunker.next_chunk().is_some());
        assert!(chunker.next_chunk().is_some());
        assert!(chunker.next_chunk().is_none());
    }

    #[test]
    fn test_chunker_overlap() {
        let mut chunker = Chunker::new(16000, 500, 250);
        let samples = vec![0.5f32; 8000];
        chunker.push(&samples);
        let chunk1 = chunker.next_chunk().unwrap();
        assert_eq!(chunk1.len(), 8000);
    }

    #[test]
    fn test_chunker_flush() {
        let mut chunker = Chunker::new(16000, 500, 250);
        let samples = vec![0.5f32; 4000];
        chunker.push(&samples);
        assert!(chunker.next_chunk().is_none());
        let flushed = chunker.flush().unwrap();
        assert_eq!(flushed.len(), 8000);
    }

    #[test]
    fn test_chunker_empty_flush() {
        let mut chunker = Chunker::new(16000, 500, 250);
        assert!(chunker.flush().is_none());
    }
}
