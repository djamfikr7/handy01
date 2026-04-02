use tokio::sync::Mutex;

use crate::correction::SlidingWindow;
use crate::hotkey::HotkeyManager;
use crate::inject::TextInjector;
use crate::sidecar::SidecarClient;
use crate::sidecar::SidecarProcess;

use super::settings::Settings;

pub struct AppState {
    pub settings: Mutex<Settings>,
    pub hotkey_manager: HotkeyManager,
    pub sidecar_process: Mutex<SidecarProcess>,
    pub sidecar_client: SidecarClient,
    pub text_injector: Mutex<TextInjector>,
    pub sliding_window: Mutex<SlidingWindow>,
    pub is_recording: Mutex<bool>,
    pub last_transcript: Mutex<String>,
}

impl AppState {
    pub fn new(settings: Settings) -> Self {
        let port = settings.sidecar_port;
        Self {
            settings: Mutex::new(settings),
            hotkey_manager: HotkeyManager::new(),
            sidecar_process: Mutex::new(SidecarProcess::new(port)),
            sidecar_client: SidecarClient::new(None),
            text_injector: Mutex::new(TextInjector::new()),
            sliding_window: Mutex::new(SlidingWindow::new(5000)),
            is_recording: Mutex::new(false),
            last_transcript: Mutex::new(String::new()),
        }
    }
}
