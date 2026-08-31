use tauri::State;

use crate::AppState;

#[tauri::command]
pub fn enqueue_analysis(state: State<AppState>, task_id: String) -> Result<(), String> {
    state.agent_worker.enqueue(task_id);
    Ok(())
}

#[tauri::command]
pub fn get_task_analysis(
    state: State<AppState>,
    task_id: String,
) -> Result<serde_json::Value, String> {
    let task = state.db.get_task(&task_id).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "agentNotes": task.agent_notes,
        "analysisStatus": task.analysis_status,
    }))
}
