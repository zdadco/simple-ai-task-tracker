use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::db::AppDatabase;
use crate::windows;
use crate::AppState;

#[tauri::command]
pub fn show_quick_capture(app: AppHandle) -> Result<(), String> {
    show_and_focus(&app, "quick-capture")
}

#[tauri::command]
pub fn show_main_window(app: AppHandle) -> Result<(), String> {
    show_and_focus(&app, "main")
}

#[tauri::command]
pub fn show_settings_window(app: AppHandle) -> Result<(), String> {
    show_and_focus(&app, "settings")
}

#[tauri::command]
pub fn hide_window(app: AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.hide().map_err(|e| e.to_string())?;
        windows::keep_tray_only(&app);
    }
    Ok(())
}

#[tauri::command]
pub fn register_hotkey(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    let settings = state.db.get_settings().map_err(|e| e.to_string())?;
    register_hotkey_internal(&app, &settings.global_hotkey)
}

pub fn register_hotkey_from_settings(app: &AppHandle) -> Result<(), String> {
    let db = AppDatabase::new().map_err(|e| e.to_string())?;
    let settings = db.get_settings().map_err(|e| e.to_string())?;
    register_hotkey_internal(app, &settings.global_hotkey)
}

fn register_hotkey_internal(app: &AppHandle, hotkey_str: &str) -> Result<(), String> {
    let shortcut = parse_hotkey(hotkey_str)?;

    app.global_shortcut()
        .unregister_all()
        .map_err(|e| e.to_string())?;

    let app_clone = app.clone();
    app.global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                let _ = show_and_focus(&app_clone, "quick-capture");
            }
        })
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn show_and_focus(app: &AppHandle, label: &str) -> Result<(), String> {
    windows::prepare_window_show(app);

    let window = app.get_webview_window(label).ok_or_else(|| {
        let labels: Vec<String> = app
            .webview_windows()
            .keys()
            .cloned()
            .collect();
        log::error!("Window '{label}' not found. Available: {labels:?}");
        format!("Window '{label}' not found")
    })?;

    window.show().map_err(|e| e.to_string())?;
    window.unminimize().ok();
    window.set_focus().map_err(|e| e.to_string())?;

    if label == "quick-capture" {
        let _ = window.emit("quick-capture-focus", ());
    }

    Ok(())
}

fn parse_hotkey(s: &str) -> Result<Shortcut, String> {
    let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
    if parts.is_empty() {
        return Err("Invalid hotkey".into());
    }

    let mut modifiers = Modifiers::empty();
    let mut key_part = "";

    for part in &parts {
        let lower = part.to_lowercase();
        match lower.as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "shift" => modifiers |= Modifiers::SHIFT,
            "alt" => modifiers |= Modifiers::ALT,
            "meta" | "cmd" | "command" | "super" | "win" => modifiers |= Modifiers::SUPER,
            _ => key_part = *part,
        }
    }

    if key_part.is_empty() {
        return Err("Hotkey must include a key".into());
    }

    let code = parse_key_code(key_part)?;
    Ok(Shortcut::new(Some(modifiers), code))
}

fn parse_key_code(key: &str) -> Result<Code, String> {
    let key = key.to_uppercase();
    let single = key.chars().next().ok_or("Empty key")?;

    if key.len() == 1 && single.is_ascii_alphabetic() {
        return match single {
            'A' => Ok(Code::KeyA),
            'B' => Ok(Code::KeyB),
            'C' => Ok(Code::KeyC),
            'D' => Ok(Code::KeyD),
            'E' => Ok(Code::KeyE),
            'F' => Ok(Code::KeyF),
            'G' => Ok(Code::KeyG),
            'H' => Ok(Code::KeyH),
            'I' => Ok(Code::KeyI),
            'J' => Ok(Code::KeyJ),
            'K' => Ok(Code::KeyK),
            'L' => Ok(Code::KeyL),
            'M' => Ok(Code::KeyM),
            'N' => Ok(Code::KeyN),
            'O' => Ok(Code::KeyO),
            'P' => Ok(Code::KeyP),
            'Q' => Ok(Code::KeyQ),
            'R' => Ok(Code::KeyR),
            'S' => Ok(Code::KeyS),
            'T' => Ok(Code::KeyT),
            'U' => Ok(Code::KeyU),
            'V' => Ok(Code::KeyV),
            'W' => Ok(Code::KeyW),
            'X' => Ok(Code::KeyX),
            'Y' => Ok(Code::KeyY),
            'Z' => Ok(Code::KeyZ),
            _ => Err(format!("Unsupported key: {key}")),
        };
    }

    match key.as_str() {
        "SPACE" => Ok(Code::Space),
        "ENTER" | "RETURN" => Ok(Code::Enter),
        "ESC" | "ESCAPE" => Ok(Code::Escape),
        "TAB" => Ok(Code::Tab),
        "F1" => Ok(Code::F1),
        "F2" => Ok(Code::F2),
        "F3" => Ok(Code::F3),
        "F4" => Ok(Code::F4),
        "F5" => Ok(Code::F5),
        "F6" => Ok(Code::F6),
        "F7" => Ok(Code::F7),
        "F8" => Ok(Code::F8),
        "F9" => Ok(Code::F9),
        "F10" => Ok(Code::F10),
        "F11" => Ok(Code::F11),
        "F12" => Ok(Code::F12),
        _ => Err(format!("Unsupported key: {key}")),
    }
}
