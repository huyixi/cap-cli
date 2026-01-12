use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
        supports_keyboard_enhancement,
    },
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
    let mut guard = TerminalGuard::new()?;
    let mut state = TuiState::new(crate::fetch_memos(conn, None)?);

    let result = run_tui_loop(guard.terminal_mut(), conn, &mut state);
    let restore_result = guard.restore();
    result.and(restore_result)
}

fn setup_terminal() -> Result<(Terminal<CrosstermBackend<io::Stdout>>, bool)> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    let keyboard_enhanced = matches!(supports_keyboard_enhancement(), Ok(true));
    if keyboard_enhanced {
        execute!(
            stdout,
            PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                    | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
                    | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
                    | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
            )
        )?;
    }
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture,)?;
    let backend = CrosstermBackend::new(stdout);
    Ok((Terminal::new(backend)?, keyboard_enhanced))
}

struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    keyboard_enhanced: bool,
    restored: bool,
}

impl TerminalGuard {
    fn new() -> Result<Self> {
        let (terminal, keyboard_enhanced) = setup_terminal()?;
        Ok(Self {
            terminal,
            keyboard_enhanced,
            restored: false,
        })
    }

    fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<io::Stdout>> {
        &mut self.terminal
    }

    fn restore(&mut self) -> Result<()> {
        if self.restored {
            return Ok(());
        }
        self.restored = true;
        restore_terminal(&mut self.terminal, self.keyboard_enhanced)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if self.restored {
            return;
        }
        let _ = restore_terminal(&mut self.terminal, self.keyboard_enhanced);
        self.restored = true;
    }
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    keyboard_enhanced: bool,
) -> Result<()> {
    disable_raw_mode()?;
    if keyboard_enhanced {
        execute!(terminal.backend_mut(), PopKeyboardEnhancementFlags)?;
    }
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
