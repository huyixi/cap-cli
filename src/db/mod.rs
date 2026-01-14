use anyhow::Result;
use rusqlite::Connection;
use std::path::PathBuf;

mod kv_repo;
mod memo_repo;
mod schema;

pub(crate) use kv_repo::set_kv;
pub(crate) use memo_repo::{add_memo, fetch_memos};

pub(crate) struct Db {
    conn: Connection,
}

impl Db {
    pub(crate) fn open(path: PathBuf) -> Result<Self> {
        let conn = Connection::open(path)?;
        schema::init(&conn)?;
        Ok(Self { conn })
    }

    pub(crate) fn conn(&self) -> &Connection {
        &self.conn
    }
}
