use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use rusqlite::Connection;
use std::io;

mod handler;
mod state;
mod view;

use handler::handle_tui_key;
use state::TuiState;
use view::draw_tui;

const TUI_POLL_MS: u64 = 200;

pub(crate) fn run_tui(conn: &Connection) -> Result<()> {
    let mut terminal = setup_terminal()?;
    let mut state = TuiState::new(crate::fetch_memos(conn, None)?);

    let result = run_tui_loop(&mut terminal, conn, &mut state);
    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
    )?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        DisableMouseCapture,
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_tui_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    conn: &Connection,
    state: &mut TuiState,
) -> Result<()> {
    loop {
        terminal.draw(|frame| draw_tui(frame, state))?;
        if !poll_event()? {
            continue;
        }
        match event::read()? {
            Event::Key(key) => {
                if handle_tui_key(conn, state, key)? {
                    break;
                }
            }
            Event::Mouse(_) => {}
            _ => {}
        }
    }
    Ok(())
}

fn poll_event() -> Result<bool> {
    Ok(event::poll(std::time::Duration::from_millis(TUI_POLL_MS))?)
}
