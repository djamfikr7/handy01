use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::correction::CorrectionStyle;
use crate::hotkey::HotkeyConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub hotkey: HotkeyConfig,
    pub correction_style: CorrectionStyle,
    pub whisper_model: String,
    pub local_llm_model: String,
    pub cloud_provider: String,
    pub openai_api_key: String,
    pub anthropic_api_key: String,
    pub sidecar_port: u16,
    pub auto_start_sidecar: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey: HotkeyConfig::default(),
            correction_style: CorrectionStyle::default(),
            whisper_model: "large-v3".to_string(),
            local_llm_model: "TinyLlama/TinyLlama-1.1B-Chat-v1.0".to_string(),
            cloud_provider: "openai".to_string(),
            openai_api_key: String::new(),
            anthropic_api_key: String::new(),
            sidecar_port: 8765,
            auto_start_sidecar: true,
        }
    }
}

impl Settings {
    pub fn load() -> Result<Self, String> {
        let path = Self::settings_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read settings: {}", e))?;

        serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {}", e))
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::settings_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create settings dir: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        std::fs::write(&path, content).map_err(|e| format!("Failed to write settings: {}", e))
    }

    fn settings_path() -> Result<PathBuf, String> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| "Could not find config directory".to_string())?;
        Ok(config_dir.join("handy01").join("settings.json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.sidecar_port, 8765);
        assert_eq!(settings.whisper_model, "large-v3");
        assert!(settings.auto_start_sidecar);
    }

    #[test]
    fn test_settings_serialization() {
        let settings = Settings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let loaded: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings.sidecar_port, loaded.sidecar_port);
    }
}
