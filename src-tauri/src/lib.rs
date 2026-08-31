use std::sync::Arc;

use agent::worker::AgentWorker;
use db::AppDatabase;
use tauri::{Manager, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;

mod agent;
mod commands;
mod db;
mod tray;
mod windows;

pub struct AppState {
    pub db: AppDatabase,
    pub agent_worker: AgentWorker,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let db = AppDatabase::new().expect("failed to initialize database");
    let db_arc = Arc::new(db.clone());

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .invoke_handler(tauri::generate_handler![
            commands::tasks::create_task,
            commands::tasks::update_task,
            commands::tasks::delete_task,
            commands::tasks::list_tasks,
            commands::tasks::reorder_tasks,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::test_llm_connection,
            commands::analysis::enqueue_analysis,
            commands::analysis::get_task_analysis,
            commands::windows::show_quick_capture,
            commands::windows::show_main_window,
            commands::windows::show_settings_window,
            commands::windows::hide_window,
            commands::windows::register_hotkey,
        ])
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if let Err(e) = window.hide() {
                    log::warn!("Failed to hide window on close: {e}");
                }
                windows::keep_tray_only(window.app_handle());
            }
        })
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            windows::keep_tray_only(app.handle());

            let agent_worker = AgentWorker::new(db_arc.clone(), app.handle().clone());

            app.manage(AppState {
                db: db.clone(),
                agent_worker,
            });

            tray::setup_tray(app)?;

            if let Err(e) = commands::windows::register_hotkey_from_settings(app.handle()) {
                log::warn!("Failed to register hotkey on startup: {e}");
            }

            // Apply autostart setting
            if let Ok(settings) = db.get_settings() {
                if let Err(e) = commands::settings::apply_autostart(app.handle(), settings.autostart_enabled) {
                    log::warn!("Failed to apply autostart: {e}");
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
