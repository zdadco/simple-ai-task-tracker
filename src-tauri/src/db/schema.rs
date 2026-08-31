use rusqlite::{Connection, Result as SqlResult};

pub fn run_migrations(conn: &Connection) -> SqlResult<()> {
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
            analysis_status TEXT NOT NULL DEFAULT 'none'
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_tasks_sort_order ON tasks(sort_order);
        CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks(priority);
        ",
    )?;
    Ok(())
}
