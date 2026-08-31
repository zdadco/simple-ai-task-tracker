use chrono::Utc;
use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl TaskPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskPriority::Low => "low",
            TaskPriority::Medium => "medium",
            TaskPriority::High => "high",
            TaskPriority::Critical => "critical",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "low" => TaskPriority::Low,
            "high" => TaskPriority::High,
            "critical" => TaskPriority::Critical,
            _ => TaskPriority::Medium,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub sort_order: i32,
    pub priority: TaskPriority,
    pub agent_notes: Option<String>,
    pub analysis_status: String,
}

fn row_to_task(row: &rusqlite::Row) -> rusqlite::Result<Task> {
    let priority_str: String = row.get(4)?;
    Ok(Task {
        id: row.get(0)?,
        title: row.get(1)?,
        created_at: row.get(2)?,
        updated_at: row.get(3)?,
        sort_order: row.get(5)?,
        priority: TaskPriority::from_str(&priority_str),
        agent_notes: row.get(6)?,
        analysis_status: row.get(7)?,
    })
}

const TASK_SELECT: &str =
    "SELECT id, title, created_at, updated_at, priority, sort_order, agent_notes, analysis_status FROM tasks";

pub fn create_task(conn: &Connection, title: &str) -> SqlResult<Task> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();
    let min_order: i32 = conn
        .query_row("SELECT COALESCE(MIN(sort_order), 0) FROM tasks", [], |r| {
            r.get(0)
        })
        .unwrap_or(0);
    let sort_order = min_order - 1;

    conn.execute(
        "INSERT INTO tasks (id, title, created_at, updated_at, sort_order, priority, analysis_status)
         VALUES (?1, ?2, ?3, ?4, ?5, 'medium', 'none')",
        params![id, title.trim(), now, now, sort_order],
    )?;

    get_task(conn, &id)
}

pub fn update_task(
    conn: &Connection,
    id: &str,
    title: Option<&str>,
    priority: Option<TaskPriority>,
) -> SqlResult<Task> {
    let now = Utc::now().timestamp();

    if let Some(t) = title {
        conn.execute(
            "UPDATE tasks SET title = ?1, updated_at = ?2 WHERE id = ?3",
            params![t.trim(), now, id],
        )?;
    }

    if let Some(p) = priority {
        conn.execute(
            "UPDATE tasks SET priority = ?1, updated_at = ?2 WHERE id = ?3",
            params![p.as_str(), now, id],
        )?;
    }

    get_task(conn, id)
}

pub fn delete_task(conn: &Connection, id: &str) -> SqlResult<()> {
    conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn list_tasks(conn: &Connection, priority_filter: Option<&str>) -> SqlResult<Vec<Task>> {
    let mut tasks = Vec::new();

    if let Some(priority) = priority_filter {
        if !priority.is_empty() && priority != "all" {
            let sql = format!("{TASK_SELECT} WHERE priority = ?1 ORDER BY sort_order ASC");
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params![priority], row_to_task)?;
            for task in rows {
                tasks.push(task?);
            }
            return Ok(tasks);
        }
    }

    let sql = format!("{TASK_SELECT} ORDER BY sort_order ASC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], row_to_task)?;
    for task in rows {
        tasks.push(task?);
    }

    Ok(tasks)
}

pub fn reorder_tasks(conn: &Connection, ordered_ids: &[String]) -> SqlResult<()> {
    let now = Utc::now().timestamp();
    for (index, id) in ordered_ids.iter().enumerate() {
        conn.execute(
            "UPDATE tasks SET sort_order = ?1, updated_at = ?2 WHERE id = ?3",
            params![index as i32, now, id],
        )?;
    }
    Ok(())
}

pub fn get_task(conn: &Connection, id: &str) -> SqlResult<Task> {
    let sql = format!("{TASK_SELECT} WHERE id = ?1");
    conn.query_row(&sql, params![id], row_to_task)
}

pub fn set_analysis_status(conn: &Connection, id: &str, status: &str) -> SqlResult<()> {
    let now = Utc::now().timestamp();
    conn.execute(
        "UPDATE tasks SET analysis_status = ?1, updated_at = ?2 WHERE id = ?3",
        params![status, now, id],
    )?;
    Ok(())
}

pub fn set_agent_notes(conn: &Connection, id: &str, notes: &str, status: &str) -> SqlResult<()> {
    let now = Utc::now().timestamp();
    conn.execute(
        "UPDATE tasks SET agent_notes = ?1, analysis_status = ?2, updated_at = ?3 WHERE id = ?4",
        params![notes, status, now, id],
    )?;
    Ok(())
}
