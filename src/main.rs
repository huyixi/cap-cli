use anyhow::Result;
use chrono::{DateTime, Local};
use clap::{ArgAction, Parser, Subcommand};
use rusqlite::{Connection, params};
use std::{env, fs, path::PathBuf};
use uuid::Uuid;

mod tui;

#[derive(Parser)]
#[command(name = "cap")]
#[command(about = "A tiny memo app", version)]
struct Cli {
    content: Option<String>,

    #[arg(short = 'v', long = "version", action = ArgAction::Version)]
    version: Option<bool>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Add {
        content: String,
    },
    Version,
    #[command(alias = "ls")]
    List,
}

fn init_db(conn: &Connection) -> Result<()> {
    create_memos_table(conn)
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

pub(crate) fn add_memo(conn: &Connection, content: &str) -> Result<()> {
    let now = Local::now().to_rfc3339();
    let memo_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO memos (
            memo_id,
            content,
            created_at,
            updated_at,
            deleted,
            dirty,
            server_rev
        ) VALUES (?1, ?2, ?3, ?4, 0, 1, 0)",
        params![memo_id, content, now, now],
    )?;
    Ok(())
}

fn list_memos(conn: &Connection) -> Result<()> {
    let memos = fetch_memos(conn, None)?;
    for (created_at, content) in memos {
        let display_time = format_display_time(&created_at);
        println!("{}  {}", display_time, content);
    }

    Ok(())
}

pub(crate) fn format_display_time(value: &str) -> String {
    match DateTime::parse_from_rfc3339(value) {
        Ok(timestamp) => timestamp
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
        Err(_) => value.to_string(),
    }
}

pub(crate) fn fetch_memos(conn: &Connection, limit: Option<usize>) -> Result<Vec<(String, String)>> {
    let limit_value = limit.map(|value| value as i64).unwrap_or(-1);
    let mut stmt = conn.prepare(
        "SELECT created_at, content
         FROM memos
         WHERE deleted = 0
         ORDER BY created_at DESC
         LIMIT ?1",
    )?;

    let rows = stmt.query_map(params![limit_value], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    let mut memos = Vec::new();
    for row in rows {
        memos.push(row?);
    }
    Ok(memos)
}

fn db_path() -> Result<PathBuf> {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dir = PathBuf::from(home).join(".capmind");
    fs::create_dir_all(&dir)?;
    Ok(dir.join("capmind.db"))
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let conn = Connection::open(db_path()?)?;
    init_db(&conn)?;

    match cli.command {
        Some(Command::List) => list_memos(&conn)?,
        Some(Command::Version) => {
            println!("cap {}", env!("CARGO_PKG_VERSION"));
        }
        Some(Command::Add { content }) => add_memo(&conn, &content)?,
        None if cli.content.is_some() => {
            add_memo(&conn, cli.content.as_deref().unwrap_or_default())?
        }
        None => tui::run_tui(&conn)?,
    }

    Ok(())
}
