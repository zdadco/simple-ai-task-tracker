use tauri::State;
use tauri_plugin_autostart::ManagerExt;

use crate::agent::client::LlmClient;
use crate::db::AppSettings;
use crate::AppState;

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<AppSettings, String> {
    state.db.get_settings().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_settings(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<(), String> {
    state
        .db
        .save_settings(&settings)
        .map_err(|e| e.to_string())?;

    if let Err(e) = apply_autostart(&app, settings.autostart_enabled) {
        log::warn!("Autostart apply failed: {e}");
        return Err(format!("Автозапуск: {e}"));
    }

    if let Err(e) = super::windows::register_hotkey_from_settings(&app) {
        log::warn!("Hotkey registration failed: {e}");
        return Err(format!("Горячая клавиша: {e}"));
    }

    Ok(())
}

#[tauri::command]
pub async fn test_llm_connection(
    base_url: String,
    api_key: String,
    model: String,
) -> Result<String, String> {
    let client = LlmClient::new();
    client
        .test_connection(&base_url, &api_key, &model)
        .await
        .map_err(|e| e.to_string())
}

pub fn apply_autostart(app: &tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let autostart = app.autolaunch();
    if enabled {
        return autostart.enable().map_err(|e| e.to_string());
    }

    // disable() deletes a registry Run value — fails with os error 2 if never enabled
    match autostart.is_enabled() {
        Ok(true) => autostart.disable().map_err(|e| e.to_string()),
        Ok(false) => Ok(()),
        Err(_) => match autostart.disable() {
            Ok(()) => Ok(()),
            Err(e) if is_missing_registry_entry(&e.to_string()) => Ok(()),
            Err(e) => Err(e.to_string()),
        },
    }
}

fn is_missing_registry_entry(message: &str) -> bool {
    let lower = message.to_lowercase();
    lower.contains("os error 2")
        || lower.contains("not found")
        || lower.contains("не удается найти")
}
