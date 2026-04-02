use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorrectionStyle {
    Inline,
    Highlighted,
    DraftFinal,
}

impl Default for CorrectionStyle {
    fn default() -> Self {
        Self::Inline
    }
}

impl CorrectionStyle {
    pub fn format(&self, original: &str, corrected: &str) -> String {
        match self {
            CorrectionStyle::Inline => corrected.to_string(),
            CorrectionStyle::Highlighted => {
                if original == corrected {
                    corrected.to_string()
                } else {
                    format!("[[{}]]", corrected)
                }
            }
            CorrectionStyle::DraftFinal => {
                format!("~~{}~~ → {}", original, corrected)
            }
        }
    }

    pub fn shows_original(&self) -> bool {
        matches!(self, Self::DraftFinal | Self::Highlighted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_style() {
        let style = CorrectionStyle::Inline;
        assert_eq!(style.format("hello", "Hello"), "Hello");
    }

    #[test]
    fn test_highlighted_style_changed() {
        let style = CorrectionStyle::Highlighted;
        assert_eq!(style.format("hello", "Hello"), "[[Hello]]");
    }

    #[test]
    fn test_highlighted_style_unchanged() {
        let style = CorrectionStyle::Highlighted;
        assert_eq!(style.format("Hello", "Hello"), "Hello");
    }

    #[test]
    fn test_draft_final_style() {
        let style = CorrectionStyle::DraftFinal;
        assert_eq!(style.format("hello", "Hello"), "~~hello~~ → Hello");
    }

    #[test]
    fn test_shows_original() {
        assert!(CorrectionStyle::DraftFinal.shows_original());
        assert!(CorrectionStyle::Highlighted.shows_original());
        assert!(!CorrectionStyle::Inline.shows_original());
    }

    #[test]
    fn test_default_style() {
        assert_eq!(CorrectionStyle::default(), CorrectionStyle::Inline);
    }

    #[test]
    fn test_serialization() {
        let style = CorrectionStyle::Highlighted;
        let json = serde_json::to_string(&style).unwrap();
        assert!(json.contains("Highlighted"));
    }
}
