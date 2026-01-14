use anyhow::Result;
use rusqlite::Connection;

pub(super) fn init(conn: &Connection) -> Result<()> {
    create_memos_table(conn)?;
    create_kv_table(conn)
}

fn create_memos_table(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS memos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            memo_id TEXT NOT NULL UNIQUE,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted INTEGER NOT NULL DEFAULT 0,
            dirty INTEGER NOT NULL DEFAULT 1,
            server_rev INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS memos_created_at_desc_idx
            ON memos (created_at DESC);
        CREATE INDEX IF NOT EXISTS memos_deleted_idx
            ON memos (deleted);
        CREATE INDEX IF NOT EXISTS memos_dirty_idx
            ON memos (dirty);",
    )?;
    Ok(())
}

fn create_kv_table(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS kv (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
    )?;
    Ok(())
}
