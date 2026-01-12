use anyhow::Result;
use chrono::{DateTime, Local};
use clap::{ArgAction, Parser, Subcommand};
use crossterm::terminal;
use reqwest::blocking::Client;
use rusqlite::{Connection, params};
use serde::Deserialize;
use std::{env, fs, path::PathBuf};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use uuid::Uuid;

mod tui;

const DEFAULT_SUPABASE_URL: &str = "https://your-project.supabase.co";
const DEFAULT_SUPABASE_ANON_KEY: &str = "your_anon_key";

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
    Login {
        #[arg(long)]
        email: String,
        #[arg(long)]
        password: String,
    },
    Version,
    #[command(alias = "ls")]
    List,
}

fn init_db(conn: &Connection) -> Result<()> {
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
    let terminal_width = terminal::size()
        .map(|(width, _)| width as usize)
        .unwrap_or(80);
    for (created_at, content) in memos {
        let display_time = format_display_time(&created_at);
        let line = format_memo_line(&display_time, &content, terminal_width);
        println!("{}", line);
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

fn login(conn: &Connection, email: &str, password: &str) -> Result<()> {
    let supabase_url =
        env::var("SUPABASE_URL").unwrap_or_else(|_| DEFAULT_SUPABASE_URL.to_string());
    let supabase_anon_key =
        env::var("SUPABASE_ANON_KEY").unwrap_or_else(|_| DEFAULT_SUPABASE_ANON_KEY.to_string());
    let url = format!(
        "{}/auth/v1/token?grant_type=password",
        supabase_url.trim_end_matches('/')
    );

    let client = Client::new();
    let response = client
        .post(url)
        .header("apikey", supabase_anon_key)
        .json(&LoginRequest { email, password })
        .send()?
        .error_for_status()?;

    let login_response: LoginResponse = response.json()?;
    set_kv(conn, "auth_access_token", &login_response.access_token)?;
    set_kv(conn, "auth_refresh_token", &login_response.refresh_token)?;
    set_kv(
        conn,
        "auth_expires_in",
        &login_response.expires_in.to_string(),
    )?;
    set_kv(conn, "auth_user_id", &login_response.user.id)?;
    println!("Logged in as {}", login_response.user.id);
    Ok(())
}

#[derive(Deserialize)]
struct LoginResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
    user: LoginUser,
}

#[derive(Deserialize)]
struct LoginUser {
    id: String,
}

#[derive(serde::Serialize)]
struct LoginRequest<'a> {
    email: &'a str,
    password: &'a str,
}

fn set_kv(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO kv (key, value)
         VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

fn get_kv(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM kv WHERE key = ?1")?;
    let mut rows = stmt.query(params![key])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

pub(crate) fn get_auth_token(conn: &Connection) -> Result<Option<String>> {
    get_kv(conn, "auth_access_token")
}

pub(crate) fn format_memo_line(display_time: &str, content: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    let prefix = format!("{}  ", display_time);
    let prefix_width = UnicodeWidthStr::width(prefix.as_str());
    let clean_content = sanitize_content(content);
    if max_width <= prefix_width {
        return truncate_with_ellipsis(display_time, max_width);
    }

    let content_width = max_width.saturating_sub(prefix_width);
    let truncated = truncate_with_ellipsis(&clean_content, content_width);
    format!("{}{}", prefix, truncated)
}

fn sanitize_content(content: &str) -> String {
    content
        .replace(['\n', '\r', '\t'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn truncate_with_ellipsis(value: &str, max_width: usize) -> String {
    let value_width = UnicodeWidthStr::width(value);
    if value_width <= max_width {
        return value.to_string();
    }
    if max_width <= 3 {
        return ".".repeat(max_width);
    }

    let mut current_width = 0;
    let mut result = String::new();
    for ch in value.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(1);
        if current_width + ch_width > max_width - 3 {
            break;
        }
        result.push(ch);
        current_width += ch_width;
    }
    result.push_str("...");
    result
}

pub(crate) fn fetch_memos(
    conn: &Connection,
    limit: Option<usize>,
) -> Result<Vec<(String, String)>> {
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
        Some(Command::Login { email, password }) => login(&conn, &email, &password)?,
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
