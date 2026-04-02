pub mod audio;
pub mod correction;
pub mod hotkey;
pub mod inject;
pub mod sidecar;
pub mod state;

use state::AppState;
use state::Settings;

#[tauri::command]
async fn toggle_recording(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let is_recording = state.hotkey_manager.toggle_recording().await;
    *state.is_recording.lock().await = is_recording;

    if !is_recording {
        state.sliding_window.lock().await.lock();
    }

    Ok(is_recording)
}

#[tauri::command]
async fn get_recording_state(state: tauri::State<'_, AppState>) -> bool {
    *state.is_recording.lock().await
}

#[tauri::command]
async fn get_settings(state: tauri::State<'_, AppState>) -> Result<Settings, String> {
    let settings = state.settings.lock().await;
    Ok(settings.clone())
}

#[tauri::command]
async fn update_settings(
    state: tauri::State<'_, AppState>,
    settings: Settings,
) -> Result<(), String> {
    settings.save()?;
    *state.settings.lock().await = settings;
    Ok(())
}

#[tauri::command]
async fn get_current_text(state: tauri::State<'_, AppState>) -> String {
    state.sliding_window.lock().await.get_full()
}

#[tauri::command]
async fn check_sidecar_health(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    match state.sidecar_client.health_check().await {
        Ok(health) => Ok(health.whisper_loaded && health.correction_loaded),
        Err(_) => Ok(false),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings = Settings::load().unwrap_or_default();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState::new(settings))
        .invoke_handler(tauri::generate_handler![
            toggle_recording,
            get_recording_state,
            get_settings,
            update_settings,
            get_current_text,
            check_sidecar_health,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
