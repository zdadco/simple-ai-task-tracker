use rusqlite::{Connection, Result as SqlResult};

pub fn run_migrations(conn: &Connection) -> SqlResult<()> {
    // Base tables (CREATE IF NOT EXISTS does not add new columns to existing tables)
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY NOT NULL,
            title TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            sort_order INTEGER NOT NULL,
            priority TEXT NOT NULL DEFAULT 'medium',
            agent_notes TEXT,
            analysis_status TEXT NOT NULL DEFAULT 'none',
            status TEXT NOT NULL DEFAULT 'open'
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS digests (
            id TEXT PRIMARY KEY NOT NULL,
            kind TEXT NOT NULL,
            period_start INTEGER NOT NULL,
            period_end INTEGER NOT NULL,
            content TEXT NOT NULL DEFAULT '',
            preview TEXT NOT NULL DEFAULT '',
            source TEXT NOT NULL DEFAULT 'local',
            status TEXT NOT NULL DEFAULT 'pending',
            error TEXT,
            created_at INTEGER NOT NULL,
            UNIQUE(kind, period_start)
        );

        CREATE INDEX IF NOT EXISTS idx_tasks_sort_order ON tasks(sort_order);
        CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks(priority);
        CREATE INDEX IF NOT EXISTS idx_digests_kind_period ON digests(kind, period_start);
        ",
    )?;

    // Upgrade existing installs before indexes that depend on new columns
    ensure_column(conn, "tasks", "status", "TEXT NOT NULL DEFAULT 'open'")?;

    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);",
    )?;

    Ok(())
}

fn ensure_column(conn: &Connection, table: &str, column: &str, decl: &str) -> SqlResult<()> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let names: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .collect();
    if !names.iter().any(|n| n == column) {
        conn.execute(
            &format!("ALTER TABLE {table} ADD COLUMN {column} {decl}"),
            [],
        )?;
    }
    Ok(())
}
