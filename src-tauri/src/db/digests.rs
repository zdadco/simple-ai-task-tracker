use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DigestKind {
    Daily,
    Weekly,
    Monthly,
}

impl DigestKind {
    pub fn as_str(self) -> &'static str {
        match self {
            DigestKind::Daily => "daily",
            DigestKind::Weekly => "weekly",
            DigestKind::Monthly => "monthly",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "daily" => Some(DigestKind::Daily),
            "weekly" => Some(DigestKind::Weekly),
            "monthly" => Some(DigestKind::Monthly),
            _ => None,
        }
    }

    pub fn title_ru(self) -> &'static str {
        match self {
            DigestKind::Daily => "Дайджест дня",
            DigestKind::Weekly => "Дайджест недели",
            DigestKind::Monthly => "Дайджест месяца",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Digest {
    pub id: String,
    pub kind: String,
    pub period_start: i64,
    pub period_end: i64,
    pub content: String,
    pub preview: String,
    pub source: String,
    pub status: String,
    pub error: Option<String>,
    pub created_at: i64,
}

fn row_to_digest(row: &rusqlite::Row) -> rusqlite::Result<Digest> {
    Ok(Digest {
        id: row.get(0)?,
        kind: row.get(1)?,
        period_start: row.get(2)?,
        period_end: row.get(3)?,
        content: row.get(4)?,
        preview: row.get(5)?,
        source: row.get(6)?,
        status: row.get(7)?,
        error: row.get(8)?,
        created_at: row.get(9)?,
    })
}

const DIGEST_SELECT: &str = "SELECT id, kind, period_start, period_end, content, preview, source, status, error, created_at FROM digests";

pub fn find_by_kind_period(
    conn: &Connection,
    kind: DigestKind,
    period_start: i64,
) -> SqlResult<Option<Digest>> {
    let sql = format!("{DIGEST_SELECT} WHERE kind = ?1 AND period_start = ?2");
    conn.query_row(sql.as_str(), params![kind.as_str(), period_start], row_to_digest)
        .optional()
}

pub fn create_pending(
    conn: &Connection,
    kind: DigestKind,
    period_start: i64,
    period_end: i64,
) -> SqlResult<Digest> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();
    conn.execute(
        "INSERT INTO digests (id, kind, period_start, period_end, content, preview, source, status, created_at)
         VALUES (?1, ?2, ?3, ?4, '', '', 'local', 'pending', ?5)",
        params![id, kind.as_str(), period_start, period_end, now],
    )?;
    get_digest(conn, &id)
}

pub fn upsert_running(
    conn: &Connection,
    kind: DigestKind,
    period_start: i64,
    period_end: i64,
) -> SqlResult<Digest> {
    if let Some(existing) = find_by_kind_period(conn, kind, period_start)? {
        conn.execute(
            "UPDATE digests SET status = 'running', error = NULL, period_end = ?1 WHERE id = ?2",
            params![period_end, existing.id],
        )?;
        return get_digest(conn, &existing.id);
    }
    let digest = create_pending(conn, kind, period_start, period_end)?;
    conn.execute(
        "UPDATE digests SET status = 'running' WHERE id = ?1",
        params![digest.id],
    )?;
    get_digest(conn, &digest.id)
}

pub fn complete_digest(
    conn: &Connection,
    id: &str,
    content: &str,
    preview: &str,
    source: &str,
) -> SqlResult<Digest> {
    conn.execute(
        "UPDATE digests SET content = ?1, preview = ?2, source = ?3, status = 'done', error = NULL WHERE id = ?4",
        params![content, preview, source, id],
    )?;
    get_digest(conn, id)
}

pub fn fail_digest(conn: &Connection, id: &str, error: &str) -> SqlResult<Digest> {
    conn.execute(
        "UPDATE digests SET status = 'failed', error = ?1 WHERE id = ?2",
        params![error, id],
    )?;
    get_digest(conn, id)
}

pub fn get_digest(conn: &Connection, id: &str) -> SqlResult<Digest> {
    let sql = format!("{DIGEST_SELECT} WHERE id = ?1");
    conn.query_row(sql.as_str(), params![id], row_to_digest)
}

pub fn list_digests(conn: &Connection, kind_filter: Option<&str>) -> SqlResult<Vec<Digest>> {
    let (sql, use_kind) = match kind_filter {
        Some(k) if !k.is_empty() && k != "all" => (
            format!("{DIGEST_SELECT} WHERE kind = ?1 ORDER BY period_start DESC, created_at DESC"),
            Some(k),
        ),
        _ => (
            format!("{DIGEST_SELECT} ORDER BY period_start DESC, created_at DESC"),
            None,
        ),
    };

    let mut stmt = conn.prepare(&sql)?;
    let rows = if let Some(k) = use_kind {
        stmt.query_map(params![k], row_to_digest)?
    } else {
        stmt.query_map([], row_to_digest)?
    };

    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}
