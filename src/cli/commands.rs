use anyhow::Result;
use crossterm::terminal;

use crate::{
    app::AppContext,
    auth,
    cli::args::{Cli, Command},
    db,
    domain::memo::NewMemo,
    format, tui,
};

pub(crate) fn dispatch(app: &AppContext, cli: Cli) -> Result<()> {
    match cli.command {
        Some(Command::List) => list_memos(app),
        Some(Command::Login { email, password }) => auth::login(app.db(), &email, &password),
        Some(Command::Version) => {
            println!("cap {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some(Command::Add { content }) => add_memo(app, &content),
        None if cli.content.is_some() => add_memo(app, cli.content.as_deref().unwrap_or_default()),
        None => tui::run_tui(app.db()),
    }
}

fn add_memo(app: &AppContext, content: &str) -> Result<()> {
    let new_memo = NewMemo::new(content);
    db::add_memo(app.db(), &new_memo)?;
    Ok(())
}

fn list_memos(app: &AppContext) -> Result<()> {
    let memos = db::fetch_memos(app.db(), None)?;
    let terminal_width = terminal::size()
        .map(|(width, _)| width as usize)
        .unwrap_or(80);
    for memo in memos {
        let display_time = format::format_display_time(&memo.created_at);
        let line = format::format_memo_line(&display_time, &memo.content, terminal_width);
        println!("{}", line);
    }

    Ok(())
}
