use anyhow::Result;
use chrono::Local;
use clap::{ArgAction, Parser, Subcommand};
use rusqlite::{Connection, params};
use std::{env, fs, path::PathBuf};

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
    conn.execute(
        "CREATE TABLE IF NOT EXISTS memos (
            id INTEGER PRIMARY KEY,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

pub(crate) fn add_memo(conn: &Connection, content: &str) -> Result<()> {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO memos (content, created_at, updated_at) VALUES (?1, ?2, ?3)",
        params![content, now, now],
    )?;
    Ok(())
}

fn list_memos(conn: &Connection) -> Result<()> {
    let memos = fetch_recent_memos(conn, 10)?;
    for (created_at, content) in memos {
        println!("{}  {}", created_at, content);
    }

    Ok(())
}

pub(crate) fn fetch_recent_memos(conn: &Connection, limit: usize) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT created_at, content
         FROM memos
         ORDER BY created_at DESC
         LIMIT ?1",
    )?;

    let rows = stmt.query_map(params![limit as i64], |row| {
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
    Ok(dir.join("memos.db"))
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
