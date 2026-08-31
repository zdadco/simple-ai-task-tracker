use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App,
};

use crate::commands::windows::{show_and_focus, show_quick_capture};

pub fn setup_tray(app: &App) -> tauri::Result<()> {
    let new_task = MenuItem::with_id(app, "new_task", "Новая задача", true, None::<&str>)?;
    let open_list = MenuItem::with_id(app, "open_list", "Открыть список", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Настройки", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Выход", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&new_task, &open_list, &settings, &separator, &quit],
    )?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("Micro Task Tracker")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "new_task" => {
                let _ = show_quick_capture(app.clone());
            }
            "open_list" => {
                let _ = show_and_focus(&app, "main");
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
                let _ = show_and_focus(&app, "main");
            }
        })
        .build(app)?;

    Ok(())
}
