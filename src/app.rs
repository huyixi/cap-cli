use anyhow::Result;

use crate::{config, db::Db};

pub(crate) struct AppContext {
    db: Db,
}

impl AppContext {
    pub(crate) fn new() -> Result<Self> {
        let path = config::db_path()?;
        let db = Db::open(path)?;
        Ok(Self { db })
    }

    pub(crate) fn db(&self) -> &Db {
        &self.db
    }
}
