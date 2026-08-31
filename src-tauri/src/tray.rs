use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App,
};

use crate::commands::windows::{show_and_focus, show_quick_capture};

fn tray_icon() -> tauri::Result<Image<'static>> {
    Image::from_bytes(include_bytes!("../icons/icon.png"))
}

pub fn setup_tray(app: &App) -> tauri::Result<()> {
    let new_task = MenuItem::with_id(app, "new_task", "Новая задача", true, None::<&str>)?;
    let open_list = MenuItem::with_id(app, "open_list", "Открыть список", true, None::<&str>)?;
    let digests = MenuItem::with_id(app, "digests", "Дайджесты", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Настройки", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Выход", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&new_task, &open_list, &digests, &settings, &separator, &quit],
    )?;

    // Tray id must NOT match any window label (e.g. "main") — that breaks get_webview_window on macOS.
    let _tray = TrayIconBuilder::with_id("app-tray")
        .icon(tray_icon()?)
        .icon_as_template(false)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("Micro Task Tracker")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "new_task" => {
                let _ = show_quick_capture(app.clone());
            }
            "open_list" => {
                let _ = show_and_focus(&app, "main");
            }
            "digests" => {
                let _ = show_and_focus(&app, "digests");
            }
            "settings" => {
                let _ = show_and_focus(&app, "settings");
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                log::info!("Tray left click — opening main window");
                if let Err(e) = show_and_focus(&app, "main") {
                    log::error!("Failed to open main window from tray: {e}");
                }
            }
        })
        .build(app)?;

    Ok(())
}
