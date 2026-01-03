use anyhow::Result;
use chrono::Local;
use clap::Parser;
use rusqlite::{Connection, params};

#[derive(Parser)]
struct Args {
    content: String,
}

fn init_db(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

fn add_note(conn: &Connection, content: &str) -> Result<()> {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO notes (content, created_at, updated_at) VALUES (?1, ?2, ?3)",
        params![content, now, now],
    )?;
    println!("笔记已保存！");
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let conn = Connection::open("notes.db")?;
    init_db(&conn)?;
    add_note(&conn, &args.content)?;
    Ok(())
}
