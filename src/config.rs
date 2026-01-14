use anyhow::Result;
use std::{env, fs, path::PathBuf};

pub(crate) fn db_path() -> Result<PathBuf> {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dir = PathBuf::from(home).join(".capmind");
    fs::create_dir_all(&dir)?;
    Ok(dir.join("capmind.db"))
}
