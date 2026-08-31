use chrono::Local;

use crate::agent::client::LlmClient;
use crate::db::digests::{Digest, DigestKind};
use crate::db::tasks::{Task, TaskPriority};
use crate::db::AppDatabase;
use crate::digest::period::{format_period_label, period_bounds};

pub const PREVIEW_MAX: usize = 160;

pub fn make_preview(content: &str) -> String {
    let trimmed = content.trim().replace('\n', " ");
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() <= PREVIEW_MAX {
        return trimmed;
    }
    chars[..PREVIEW_MAX.saturating_sub(1)]
        .iter()
        .collect::<String>()
        + "…"
}

fn format_tasks_for_prompt(tasks: &[Task]) -> String {
    if tasks.is_empty() {
        return "(нет задач)".into();
    }
    tasks
        .iter()
        .map(|t| format!("- [{}] {}", t.priority.as_str(), t.title))
        .collect::<Vec<_>>()
        .join("\n")
}

fn local_compose(tasks: &[Task]) -> String {
    if tasks.is_empty() {
        return "Нет незавершённых задач за период.".into();
    }

    let mut sorted = tasks.to_vec();
    sorted.sort_by(|a, b| {
        b.priority
            .rank()
            .cmp(&a.priority.rank())
            .then_with(|| a.sort_order.cmp(&b.sort_order))
    });

    let hottest = &sorted[0];
    let top: Vec<&str> = sorted.iter().take(3).map(|t| t.title.as_str()).collect();

    format!(
        "Незавершённых задач за период: {}.\n\nСамая срочная: [{}] {}\n\nТоп задач:\n{}",
        tasks.len(),
        hottest.priority.as_str(),
        hottest.title,
        top.iter()
            .map(|t| format!("- {t}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

pub async fn generate_digest(
    db: &AppDatabase,
    kind: DigestKind,
    force: bool,
) -> Result<Digest, String> {
    let now = Local::now();
    let bounds = period_bounds(kind, now);

    if !force {
        if let Ok(Some(existing)) = db.find_digest(kind, bounds.start) {
            if existing.status == "done" || existing.status == "running" {
                return Ok(existing);
            }
        }
    }

    let digest = db
        .upsert_digest_running(kind, bounds.start, bounds.end)
        .map_err(|e| e.to_string())?;

    let tasks = db
        .list_open_tasks_in_period(bounds.start, bounds.end)
        .map_err(|e| e.to_string())?;

    let settings = db.get_settings().map_err(|e| e.to_string())?;
    let period_label = format_period_label(bounds.start, bounds.end);
    let tasks_text = format_tasks_for_prompt(&tasks);
    let prompt = settings
        .digest_prompt(kind)
        .replace("{kind}", kind.as_str())
        .replace("{period_start}", &period_label)
        .replace("{period_end}", &period_label)
        .replace("{tasks}", &tasks_text);

    let client = LlmClient::new();
    let llm_result = client
        .chat_completion(
            &settings.llm_base_url,
            &settings.llm_api_key,
            &settings.llm_model,
            &prompt,
        )
        .await;

    let (content, source) = match llm_result {
        Ok(text) if !text.trim().is_empty() => (text, "llm"),
        Ok(_) | Err(_) => (local_compose(&tasks), "local"),
    };

    let preview = make_preview(&content);
    db.complete_digest(&digest.id, &content, &preview, source)
        .map_err(|e| e.to_string())
}

#[allow(dead_code)]
fn _priority_hint(_: TaskPriority) {}
