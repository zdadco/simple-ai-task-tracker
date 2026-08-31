use std::sync::Arc;

use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

use crate::agent::client::{LlmClient, LlmError};
use crate::db::AppDatabase;

#[derive(Clone)]
pub struct AgentWorker {
    db: Arc<AppDatabase>,
    sender: mpsc::UnboundedSender<String>,
}

impl AgentWorker {
    pub fn new(db: Arc<AppDatabase>, app: AppHandle) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        let db_clone = db.clone();
        let app_clone = app.clone();
        tauri::async_runtime::spawn(async move {
            Self::process_queue(db_clone, app_clone, receiver).await;
        });

        Self { db, sender }
    }

    pub fn enqueue(&self, task_id: String) {
        if let Err(e) = self.db.set_analysis_status(&task_id, "pending") {
            log::error!("Failed to set pending status: {e}");
            return;
        }
        let _ = self.sender.send(task_id);
    }

    async fn process_queue(
        db: Arc<AppDatabase>,
        app: AppHandle,
        mut receiver: mpsc::UnboundedReceiver<String>,
    ) {
        let client = LlmClient::new();

        while let Some(task_id) = receiver.recv().await {
            let status = match Self::analyze_task(&db, &client, &task_id).await {
                Ok(()) => "done",
                Err(e) => {
                    log::error!("Analysis failed for {task_id}: {e}");
                    let _ = db.set_analysis_status(&task_id, "failed");
                    "failed"
                }
            };
            let _ = app.emit("analysis-updated", serde_json::json!({
                "taskId": task_id,
                "status": status,
            }));
        }
    }

    async fn analyze_task(
        db: &AppDatabase,
        client: &LlmClient,
        task_id: &str,
    ) -> Result<(), LlmError> {
        db.set_analysis_status(task_id, "running").map_err(|e| {
            LlmError::Api(format!("db error: {e}"))
        })?;

        let task = db.get_task(task_id).map_err(|e| LlmError::Api(format!("{e}")))?;
        let settings = db
            .get_settings()
            .map_err(|e| LlmError::Api(format!("{e}")))?;

        let created_at = chrono::DateTime::from_timestamp(task.created_at, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| task.created_at.to_string());

        let prompt = settings
            .agent_prompt_template
            .replace("{title}", &task.title)
            .replace("{priority}", task.priority.as_str())
            .replace("{created_at}", &created_at);

        let notes = client
            .chat_completion(
                &settings.llm_base_url,
                &settings.llm_api_key,
                &settings.llm_model,
                &prompt,
            )
            .await?;

        db.set_agent_notes(task_id, &notes, "done")
            .map_err(|e| LlmError::Api(format!("{e}")))?;

        Ok(())
    }
}
