pub mod digests;
pub mod schema;
pub mod tasks;

use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::{Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};

use crate::db::digests::{Digest, DigestKind};
use crate::db::schema::run_migrations;
use crate::db::tasks::{Task, TaskPriority, TaskStatus};

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
        status: Option<TaskStatus>,
    ) -> SqlResult<Task> {
        self.with_conn(|conn| tasks::update_task(conn, id, title, priority, status))
    }

    pub fn delete_task(&self, id: &str) -> SqlResult<()> {
        self.with_conn(|conn| tasks::delete_task(conn, id))
    }

    pub fn list_tasks(
        &self,
        priority_filter: Option<&str>,
        status_filter: Option<&str>,
    ) -> SqlResult<Vec<Task>> {
        self.with_conn(|conn| tasks::list_tasks(conn, priority_filter, status_filter))
    }

    pub fn list_open_tasks_in_period(
        &self,
        period_start: i64,
        period_end: i64,
    ) -> SqlResult<Vec<Task>> {
        self.with_conn(|conn| tasks::list_open_tasks_in_period(conn, period_start, period_end))
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

    pub fn find_digest(
        &self,
        kind: DigestKind,
        period_start: i64,
    ) -> SqlResult<Option<Digest>> {
        self.with_conn(|conn| digests::find_by_kind_period(conn, kind, period_start))
    }

    pub fn upsert_digest_running(
        &self,
        kind: DigestKind,
        period_start: i64,
        period_end: i64,
    ) -> SqlResult<Digest> {
        self.with_conn(|conn| digests::upsert_running(conn, kind, period_start, period_end))
    }

    pub fn complete_digest(
        &self,
        id: &str,
        content: &str,
        preview: &str,
        source: &str,
    ) -> SqlResult<Digest> {
        self.with_conn(|conn| digests::complete_digest(conn, id, content, preview, source))
    }

    pub fn fail_digest(&self, id: &str, error: &str) -> SqlResult<Digest> {
        self.with_conn(|conn| digests::fail_digest(conn, id, error))
    }

    pub fn get_digest(&self, id: &str) -> SqlResult<Digest> {
        self.with_conn(|conn| digests::get_digest(conn, id))
    }

    pub fn list_digests(&self, kind_filter: Option<&str>) -> SqlResult<Vec<Digest>> {
        self.with_conn(|conn| digests::list_digests(conn, kind_filter))
    }
}

pub mod settings {
    use super::*;

    const DEFAULT_PROMPT: &str = "Проанализируй задачу и дай краткие заметки (3–5 пунктов):\nчто важно, возможные шаги, риски.\n\nЗадача: {title}\nПриоритет: {priority}\nСоздана: {created_at}\n\nОтветь на русском, в markdown.";

    const DEFAULT_DAILY_PROMPT: &str = "Составь план на день по незавершённым задачам, созданным за сегодня.\nПериод: {period_start} — {period_end}\nЗадачи:\n{tasks}\n\nОтветь на русском, кратко, markdown.";
    const DEFAULT_WEEKLY_PROMPT: &str = "Составь план на неделю по незавершённым задачам, созданным за эту неделю.\nПериод: {period_start} — {period_end}\nЗадачи:\n{tasks}\n\nОтветь на русском, markdown.";
    const DEFAULT_MONTHLY_PROMPT: &str = "Составь план на месяц по незавершённым задачам, созданным за этот месяц.\nПериод: {period_start} — {period_end}\nЗадачи:\n{tasks}\n\nОтветь на русском, markdown.";

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
        pub daily_enabled: bool,
        pub daily_time: String,
        pub daily_prompt_template: String,
        pub weekly_enabled: bool,
        pub weekly_time: String,
        pub weekly_prompt_template: String,
        pub monthly_enabled: bool,
        pub monthly_time: String,
        pub monthly_prompt_template: String,
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
                daily_enabled: true,
                daily_time: "09:00".into(),
                daily_prompt_template: DEFAULT_DAILY_PROMPT.into(),
                weekly_enabled: true,
                weekly_time: "09:00".into(),
                weekly_prompt_template: DEFAULT_WEEKLY_PROMPT.into(),
                monthly_enabled: true,
                monthly_time: "09:00".into(),
                monthly_prompt_template: DEFAULT_MONTHLY_PROMPT.into(),
            }
        }
    }

    impl AppSettings {
        pub fn digest_enabled(&self, kind: DigestKind) -> bool {
            match kind {
                DigestKind::Daily => self.daily_enabled,
                DigestKind::Weekly => self.weekly_enabled,
                DigestKind::Monthly => self.monthly_enabled,
            }
        }

        pub fn digest_time(&self, kind: DigestKind) -> &str {
            match kind {
                DigestKind::Daily => &self.daily_time,
                DigestKind::Weekly => &self.weekly_time,
                DigestKind::Monthly => &self.monthly_time,
            }
        }

        pub fn digest_prompt(&self, kind: DigestKind) -> &str {
            match kind {
                DigestKind::Daily => &self.daily_prompt_template,
                DigestKind::Weekly => &self.weekly_prompt_template,
                DigestKind::Monthly => &self.monthly_prompt_template,
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
                "daily_enabled" => settings.daily_enabled = value == "true",
                "daily_time" => settings.daily_time = value,
                "daily_prompt_template" => settings.daily_prompt_template = value,
                "weekly_enabled" => settings.weekly_enabled = value == "true",
                "weekly_time" => settings.weekly_time = value,
                "weekly_prompt_template" => settings.weekly_prompt_template = value,
                "monthly_enabled" => settings.monthly_enabled = value == "true",
                "monthly_time" => settings.monthly_time = value,
                "monthly_prompt_template" => settings.monthly_prompt_template = value,
                _ => {}
            }
        }

        Ok(settings)
    }

    pub fn save_settings(conn: &Connection, settings: &AppSettings) -> SqlResult<()> {
        let bool_str = |b: bool| if b { "true" } else { "false" };
        let pairs = [
            ("llm_base_url", settings.llm_base_url.as_str()),
            ("llm_api_key", settings.llm_api_key.as_str()),
            ("llm_model", settings.llm_model.as_str()),
            ("agent_prompt_template", settings.agent_prompt_template.as_str()),
            ("analyze_on_create", bool_str(settings.analyze_on_create)),
            ("global_hotkey", settings.global_hotkey.as_str()),
            ("autostart_enabled", bool_str(settings.autostart_enabled)),
            (
                "quick_capture_hint_shown",
                bool_str(settings.quick_capture_hint_shown),
            ),
            ("daily_enabled", bool_str(settings.daily_enabled)),
            ("daily_time", settings.daily_time.as_str()),
            ("daily_prompt_template", settings.daily_prompt_template.as_str()),
            ("weekly_enabled", bool_str(settings.weekly_enabled)),
            ("weekly_time", settings.weekly_time.as_str()),
            (
                "weekly_prompt_template",
                settings.weekly_prompt_template.as_str(),
            ),
            ("monthly_enabled", bool_str(settings.monthly_enabled)),
            ("monthly_time", settings.monthly_time.as_str()),
            (
                "monthly_prompt_template",
                settings.monthly_prompt_template.as_str(),
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
