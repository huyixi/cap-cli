use anyhow::Result;
use chrono::Local;
use rusqlite::params;

use crate::{
    db::Db,
    domain::memo::{Memo, MemoId, NewMemo},
};

pub(crate) fn add_memo(db: &Db, new_memo: &NewMemo) -> Result<MemoId> {
    let now = Local::now().to_rfc3339();
    let memo_id = MemoId::new();
    db.conn().execute(
        "INSERT INTO memos (
            memo_id,
            content,
            created_at,
            updated_at,
            deleted,
            dirty,
            server_rev
        ) VALUES (?1, ?2, ?3, ?4, 0, 1, 0)",
        params![memo_id.as_str(), &new_memo.content, now, now],
    )?;
    Ok(memo_id)
}

pub(crate) fn fetch_memos(db: &Db, limit: Option<usize>) -> Result<Vec<Memo>> {
    let limit_value = limit.map(|value| value as i64).unwrap_or(-1);
    let mut stmt = db.conn().prepare(
        "SELECT memo_id, created_at, updated_at, content
         FROM memos
         WHERE deleted = 0
         ORDER BY created_at DESC
         LIMIT ?1",
    )?;

    let rows = stmt.query_map(params![limit_value], |row| {
        Ok(Memo {
            memo_id: row.get::<_, String>(0)?.into(),
            created_at: row.get(1)?,
            updated_at: row.get(2)?,
            content: row.get(3)?,
        })
    })?;

    let mut memos = Vec::new();
    for row in rows {
        memos.push(row?);
    }
    Ok(memos)
}
