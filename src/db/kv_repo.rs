use anyhow::Result;
use rusqlite::params;

use crate::db::Db;

pub(crate) fn set_kv(db: &Db, key: &str, value: &str) -> Result<()> {
    db.conn().execute(
        "INSERT INTO kv (key, value)
         VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

#[allow(dead_code)]
pub(crate) fn get_kv(db: &Db, key: &str) -> Result<Option<String>> {
    let mut stmt = db.conn().prepare("SELECT value FROM kv WHERE key = ?1")?;
    let mut rows = stmt.query(params![key])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

#[allow(dead_code)]
pub(crate) fn get_auth_token(db: &Db) -> Result<Option<String>> {
    get_kv(db, "auth_access_token")
}
