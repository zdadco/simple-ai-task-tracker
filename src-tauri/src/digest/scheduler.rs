use std::sync::Arc;
use std::time::Duration;

use chrono::Local;
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;

use crate::db::digests::DigestKind;
use crate::db::AppDatabase;
use crate::digest::generator::generate_digest;
use crate::digest::period::should_fire;

pub fn start_digest_scheduler(db: Arc<AppDatabase>, app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        // First tick soon after start for catch-up
        tokio::time::sleep(Duration::from_secs(2)).await;
        loop {
            if let Err(e) = tick(&db, &app).await {
                log::warn!("Digest scheduler tick failed: {e}");
            }
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    });
}

async fn tick(db: &AppDatabase, app: &AppHandle) -> Result<(), String> {
    let settings = db.get_settings().map_err(|e| e.to_string())?;
    let now = Local::now();

    for kind in [DigestKind::Daily, DigestKind::Weekly, DigestKind::Monthly] {
        if !settings.digest_enabled(kind) {
            continue;
        }
        let hhmm = settings.digest_time(kind);
        if !should_fire(kind, now, hhmm) {
            continue;
        }

        let bounds = crate::digest::period::period_bounds(kind, now);
        if db
            .find_digest(kind, bounds.start)
            .map_err(|e| e.to_string())?
            .is_some()
        {
            continue;
        }

        match generate_digest(db, kind, false).await {
            Ok(digest) => {
                notify_digest(app, kind, &digest.preview, false);
                let _ = app.emit(
                    "digest-updated",
                    serde_json::json!({ "id": digest.id, "kind": kind.as_str() }),
                );
            }
            Err(e) => {
                log::error!("Digest generation failed ({:?}): {e}", kind);
                notify_digest(app, kind, &e, true);
            }
        }
    }

    Ok(())
}

pub fn notify_digest(app: &AppHandle, kind: DigestKind, body: &str, is_error: bool) {
    let title = if is_error {
        "Ошибка дайджеста".to_string()
    } else {
        kind.title_ru().to_string()
    };

    if let Err(e) = app
        .notification()
        .builder()
        .title(&title)
        .body(body)
        .show()
    {
        log::warn!("Failed to show notification: {e}");
    }

    // Best-effort: focus digests window on success path is handled by UI listen;
    // keep tray accessory on macOS.
    let _ = app;
}
