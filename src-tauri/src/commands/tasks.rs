use tauri::State;

use crate::db::tasks::TaskPriority;
use crate::AppState;

#[tauri::command]
pub fn create_task(state: State<AppState>, title: String) -> Result<serde_json::Value, String> {
    if title.trim().is_empty() {
        return Err("Название задачи не может быть пустым".into());
    }

    let task = state.db.create_task(&title).map_err(|e| e.to_string())?;

    if let Ok(settings) = state.db.get_settings() {
        if settings.analyze_on_create {
            state.agent_worker.enqueue(task.id.clone());
        }
    }

    serde_json::to_value(&task).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_task(
    state: State<AppState>,
    id: String,
    title: Option<String>,
    priority: Option<String>,
) -> Result<serde_json::Value, String> {
    let priority = priority.map(|p| TaskPriority::from_str(&p));
    let task = state
        .db
        .update_task(&id, title.as_deref(), priority)
        .map_err(|e| e.to_string())?;
    serde_json::to_value(&task).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_task(state: State<AppState>, id: String) -> Result<(), String> {
    state.db.delete_task(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_tasks(
    state: State<AppState>,
    priority_filter: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    let filter = priority_filter.as_deref();
    let tasks = state
        .db
        .list_tasks(filter)
        .map_err(|e| e.to_string())?;
    tasks
        .into_iter()
        .map(|t| serde_json::to_value(&t).map_err(|e| e.to_string()))
        .collect()
}

#[tauri::command]
pub fn reorder_tasks(state: State<AppState>, ordered_ids: Vec<String>) -> Result<(), String> {
    state
        .db
        .reorder_tasks(&ordered_ids)
        .map_err(|e| e.to_string())
}
