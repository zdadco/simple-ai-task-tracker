use tauri::{AppHandle, Emitter, State};

use crate::db::digests::DigestKind;
use crate::digest::generator::generate_digest;
use crate::digest::scheduler::notify_digest;
use crate::AppState;

#[tauri::command]
pub fn list_digests(
    state: State<AppState>,
    kind_filter: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    let digests = state
        .db
        .list_digests(kind_filter.as_deref())
        .map_err(|e| e.to_string())?;
    digests
        .into_iter()
        .map(|d| serde_json::to_value(&d).map_err(|e| e.to_string()))
        .collect()
}

#[tauri::command]
pub fn get_digest(state: State<AppState>, id: String) -> Result<serde_json::Value, String> {
    let digest = state.db.get_digest(&id).map_err(|e| e.to_string())?;
    serde_json::to_value(&digest).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_digest_now(
    app: AppHandle,
    state: State<'_, AppState>,
    kind: String,
) -> Result<serde_json::Value, String> {
    let kind = DigestKind::from_str(&kind).ok_or_else(|| "Unknown digest kind".to_string())?;
    let digest = generate_digest(&state.db, kind, true).await?;
    notify_digest(&app, kind, &digest.preview, false);
    let _ = app.emit(
        "digest-updated",
        serde_json::json!({ "id": digest.id, "kind": kind.as_str() }),
    );
    serde_json::to_value(&digest).map_err(|e| e.to_string())
}
