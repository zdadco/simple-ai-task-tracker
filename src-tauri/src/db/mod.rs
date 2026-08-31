pub mod schema;
pub mod tasks;

use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::{Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};

use crate::db::schema::run_migrations;
use crate::db::tasks::{Task, TaskPriority};

#[derive(Clone)]
pub struct AppDatabase {
    conn: ArcConnection,
}

struct ArcConnection(Mutex<Connection>);

impl Clone for ArcConnection {
    fn clone(&self) -> Self {
        Self(Mutex::new(
            Connection::open(db_path()).expect("failed to reopen database"),
        ))
    }
}

impl AppDatabase {
    pub fn new() -> SqlResult<Self> {
        let path = db_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(&path)?;
        run_migrations(&conn)?;
        Ok(Self {
            conn: ArcConnection(Mutex::new(conn)),
        })
    }

    fn with_conn<F, T>(&self, f: F) -> SqlResult<T>
    where
        F: FnOnce(&Connection) -> SqlResult<T>,
    {
        let conn = self.conn.0.lock().map_err(|_| {
            rusqlite::Error::InvalidParameterName("database lock poisoned".into())
        })?;
        f(&conn)
    }

    pub fn create_task(&self, title: &str) -> SqlResult<Task> {
        self.with_conn(|conn| tasks::create_task(conn, title))
    }

    pub fn update_task(
        &self,
        id: &str,
        title: Option<&str>,
        priority: Option<TaskPriority>,
    ) -> SqlResult<Task> {
        self.with_conn(|conn| tasks::update_task(conn, id, title, priority))
    }

    pub fn delete_task(&self, id: &str) -> SqlResult<()> {
        self.with_conn(|conn| tasks::delete_task(conn, id))
    }

    pub fn list_tasks(&self, priority_filter: Option<&str>) -> SqlResult<Vec<Task>> {
        self.with_conn(|conn| tasks::list_tasks(conn, priority_filter))
    }

    pub fn reorder_tasks(&self, ordered_ids: &[String]) -> SqlResult<()> {
        self.with_conn(|conn| tasks::reorder_tasks(conn, ordered_ids))
    }

    pub fn get_task(&self, id: &str) -> SqlResult<Task> {
        self.with_conn(|conn| tasks::get_task(conn, id))
    }

    pub fn set_analysis_status(&self, id: &str, status: &str) -> SqlResult<()> {
        self.with_conn(|conn| tasks::set_analysis_status(conn, id, status))
    }

    pub fn set_agent_notes(&self, id: &str, notes: &str, status: &str) -> SqlResult<()> {
        self.with_conn(|conn| tasks::set_agent_notes(conn, id, notes, status))
    }

    pub fn get_settings(&self) -> SqlResult<AppSettings> {
        self.with_conn(|conn| settings::load_settings(conn))
    }

    pub fn save_settings(&self, settings: &AppSettings) -> SqlResult<()> {
        self.with_conn(|conn| settings::save_settings(conn, settings))
    }
}

pub mod settings {
    use super::*;

    const DEFAULT_PROMPT: &str = "Проанализируй задачу и дай краткие заметки (3–5 пунктов):\nчто важно, возможные шаги, риски.\n\nЗадача: {title}\nПриоритет: {priority}\nСоздана: {created_at}\n\nОтветь на русском, в markdown.";

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AppSettings {
        pub llm_base_url: String,
        pub llm_api_key: String,
        pub llm_model: String,
        pub agent_prompt_template: String,
        pub analyze_on_create: bool,
        pub global_hotkey: String,
        pub autostart_enabled: bool,
        pub quick_capture_hint_shown: bool,
    }

    impl Default for AppSettings {
        fn default() -> Self {
            Self {
                llm_base_url: "http://localhost:11434/v1".into(),
                llm_api_key: String::new(),
                llm_model: "llama3.2".into(),
                agent_prompt_template: DEFAULT_PROMPT.into(),
                analyze_on_create: false,
                global_hotkey: "Ctrl+Shift+T".into(),
                autostart_enabled: false,
                quick_capture_hint_shown: false,
            }
        }
    }

    pub fn load_settings(conn: &Connection) -> SqlResult<AppSettings> {
        let mut settings = AppSettings::default();
        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for row in rows {
            let (key, value) = row?;
            match key.as_str() {
                "llm_base_url" => settings.llm_base_url = value,
                "llm_api_key" => settings.llm_api_key = value,
                "llm_model" => settings.llm_model = value,
                "agent_prompt_template" => settings.agent_prompt_template = value,
                "analyze_on_create" => settings.analyze_on_create = value == "true",
                "global_hotkey" => settings.global_hotkey = value,
                "autostart_enabled" => settings.autostart_enabled = value == "true",
                "quick_capture_hint_shown" => settings.quick_capture_hint_shown = value == "true",
                _ => {}
            }
        }

        Ok(settings)
    }

    pub fn save_settings(conn: &Connection, settings: &AppSettings) -> SqlResult<()> {
        let pairs = [
            ("llm_base_url", settings.llm_base_url.as_str()),
            ("llm_api_key", settings.llm_api_key.as_str()),
            ("llm_model", settings.llm_model.as_str()),
            ("agent_prompt_template", settings.agent_prompt_template.as_str()),
            (
                "analyze_on_create",
                if settings.analyze_on_create {
                    "true"
                } else {
                    "false"
                },
            ),
            ("global_hotkey", settings.global_hotkey.as_str()),
            (
                "autostart_enabled",
                if settings.autostart_enabled {
                    "true"
                } else {
                    "false"
                },
            ),
            (
                "quick_capture_hint_shown",
                if settings.quick_capture_hint_shown {
                    "true"
                } else {
                    "false"
                },
            ),
        ];

        for (key, value) in pairs {
            conn.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                [key, value],
            )?;
        }

        Ok(())
    }
}

pub use settings::AppSettings;

fn db_path() -> PathBuf {
    let data_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    data_dir.join("simple-ai-task-tracker").join("tasks.db")
}
