use anyhow::Result;
use chrono::Local;
use clap::{Parser, Subcommand};
use rusqlite::{Connection, params};
use std::{env, fs, path::PathBuf};

#[derive(Parser)]
#[command(name = "cap")]
#[command(about = "A tiny memo app")]
struct Cli {
    content: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Add { content: String },
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

fn add_memo(conn: &Connection, content: &str) -> Result<()> {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO memos (content, created_at, updated_at) VALUES (?1, ?2, ?3)",
        params![content, now, now],
    )?;
    println!("Saved!");
    Ok(())
}

fn list_memos(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT created_at, content
         FROM memos
         ORDER BY created_at DESC
         LIMIT 10",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    for row in rows {
        let (created_at, content) = row?;
        println!("{}  {}", created_at, content);
    }

    Ok(())
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
        Some(Command::Add { content }) => add_memo(&conn, &content)?,
        None if cli.content.is_some() => {
            add_memo(&conn, cli.content.as_deref().unwrap_or_default())?
        }
        _ => {
            eprintln!("Nothing to do. Try `cap add \"hello world\"` or `cap list`.");
        }
    }

    Ok(())
}
