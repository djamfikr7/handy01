use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub modifiers: Vec<String>,
    pub key: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            modifiers: vec!["Control".to_string(), "Shift".to_string()],
            key: "Space".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct HotkeyState {
    pub is_recording: bool,
    pub config: HotkeyConfig,
}

pub struct HotkeyManager {
    state: Arc<Mutex<HotkeyState>>,
}

impl HotkeyManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(HotkeyState {
                is_recording: false,
                config: HotkeyConfig::default(),
            })),
        }
    }

    pub async fn get_state(&self) -> HotkeyState {
        self.state.lock().await.clone()
    }

    pub async fn toggle_recording(&self) -> bool {
        let mut state = self.state.lock().await;
        state.is_recording = !state.is_recording;
        state.is_recording
    }

    pub async fn is_recording(&self) -> bool {
        self.state.lock().await.is_recording
    }

    pub async fn set_config(&self, config: HotkeyConfig) {
        let mut state = self.state.lock().await;
        state.config = config;
    }

    pub async fn get_hotkey_string(&self) -> String {
        let state = self.state.lock().await;
        let mods = state.config.modifiers.join("+");
        format!("{}+{}", mods, state.config.key)
    }
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hotkey_manager_creation() {
        let manager = HotkeyManager::new();
        let state = manager.get_state().await;
        assert!(!state.is_recording);
        assert_eq!(state.config.key, "Space");
    }

    #[tokio::test]
    async fn test_toggle_recording() {
        let manager = HotkeyManager::new();
        let recording = manager.toggle_recording().await;
        assert!(recording);
        let recording = manager.toggle_recording().await;
        assert!(!recording);
    }

    #[tokio::test]
    async fn test_is_recording() {
        let manager = HotkeyManager::new();
        assert!(!manager.is_recording().await);
        manager.toggle_recording().await;
        assert!(manager.is_recording().await);
    }

    #[tokio::test]
    async fn test_set_config() {
        let manager = HotkeyManager::new();
        manager
            .set_config(HotkeyConfig {
                modifiers: vec!["Alt".to_string()],
                key: "D".to_string(),
            })
            .await;
        let hotkey = manager.get_hotkey_string().await;
        assert_eq!(hotkey, "Alt+D");
    }

    #[tokio::test]
    async fn test_default_hotkey_string() {
        let manager = HotkeyManager::new();
        let hotkey = manager.get_hotkey_string().await;
        assert_eq!(hotkey, "Control+Shift+Space");
    }
}
