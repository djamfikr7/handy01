/// Deduplication and merge logic for overlapping transcription chunks
pub struct Merger;

impl Merger {
    pub fn merge(existing: &str, new: &str) -> String {
        if existing.is_empty() {
            return new.to_string();
        }
        if new.is_empty() {
            return existing.to_string();
        }

        let existing_words: Vec<&str> = existing.split_whitespace().collect();
        let new_words: Vec<&str> = new.split_whitespace().collect();

        let max_overlap = existing_words.len().min(new_words.len());

        for overlap in (1..=max_overlap).rev() {
            let existing_suffix = &existing_words[existing_words.len() - overlap..];
            let new_prefix = &new_words[..overlap];

            if Self::words_match(existing_suffix, new_prefix) {
                let mut result = existing.to_string();
                for word in &new_words[overlap..] {
                    result.push(' ');
                    result.push_str(word);
                }
                return result.trim().to_string();
            }
        }

        format!("{} {}", existing.trim_end(), new)
    }

    fn words_match(a: &[&str], b: &[&str]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        a.iter()
            .zip(b.iter())
            .all(|(x, y)| x.to_lowercase() == y.to_lowercase())
    }

    pub fn deduplicate(text: &str) -> String {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() < 4 {
            return text.to_string();
        }

        for seg_len in (2..=4).rev() {
            if words.len() < seg_len * 2 {
                continue;
            }

            for i in 0..=(words.len() - seg_len * 2) {
                let left = &words[i..i + seg_len];
                let right = &words[i + seg_len..i + seg_len * 2];

                if Self::words_match(left, right) {
                    let mut result = words[..i + seg_len].join(" ");
                    if i + seg_len * 2 < words.len() {
                        result.push(' ');
                        result.push_str(&words[i + seg_len * 2..].join(" "));
                    }
                    return result;
                }
            }
        }

        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_no_overlap() {
        let result = Merger::merge("Hello", "world");
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_merge_with_overlap() {
        let result = Merger::merge("Hello world how", "world how are you");
        assert_eq!(result, "Hello world how are you");
    }

    #[test]
    fn test_merge_empty_existing() {
        let result = Merger::merge("", "Hello world");
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_merge_empty_new() {
        let result = Merger::merge("Hello world", "");
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_merge_exact_match() {
        let result = Merger::merge("Hello world", "Hello world");
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_deduplicate_removes_repetition() {
        let text = "Hello world Hello world how are you";
        let result = Merger::deduplicate(text);
        assert!(!result.contains("Hello world Hello world"));
    }

    #[test]
    fn test_deduplicate_no_repetition() {
        let text = "Hello world how are you";
        let result = Merger::deduplicate(text);
        assert_eq!(result, text);
    }

    #[test]
    fn test_deduplicate_short_text() {
        let text = "Hi";
        let result = Merger::deduplicate(text);
        assert_eq!(result, text);
    }
}
