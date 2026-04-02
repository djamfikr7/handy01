pub mod audio;
pub mod correction;
pub mod hotkey;
pub mod inject;
pub mod sidecar;
pub mod state;

use state::AppState;
use state::Settings;
use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

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
async fn get_recording_state(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.is_recording.lock().await)
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
async fn get_current_text(state: tauri::State<'_, AppState>) -> Result<String, String> {
    Ok(state.sliding_window.lock().await.get_full())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings = Settings::load().unwrap_or_default();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, _event| {
                    let handle: tauri::AppHandle = app.clone();

                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        rt.block_on(async {
                            let state = handle.state::<AppState>();
                            let is_recording = state.hotkey_manager.toggle_recording().await;
                            *state.is_recording.lock().await = is_recording;

                            if !is_recording {
                                state.sliding_window.lock().await.lock();
                            }

                            let _ = handle.emit("recording-toggled", is_recording);
                        });
                    });
                })
                .build(),
        )
        .setup(|app| {
            let shortcut = Shortcut::new(
                Some(Modifiers::CONTROL | Modifiers::SHIFT),
                Code::Space,
            );
            app.global_shortcut().register(shortcut)?;
            Ok(())
        })
        .manage(AppState::new(settings))
        .invoke_handler(tauri::generate_handler![
            toggle_recording,
            get_recording_state,
            get_settings,
            update_settings,
            get_current_text,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
