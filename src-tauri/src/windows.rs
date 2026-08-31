use tauri::AppHandle;

/// macOS: Accessory policy = menu bar / tray only, no Dock icon.
pub fn keep_tray_only(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    {
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    }
    let _ = app;
}

pub fn prepare_window_show(app: &AppHandle) {
    keep_tray_only(app);
}
