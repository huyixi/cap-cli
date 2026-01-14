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
use std::io;

mod handler;
mod state;
mod view;

use crate::db::Db;
use handler::handle_tui_key;
use state::TuiState;
use view::draw_tui;

const TUI_POLL_MS: u64 = 200;

pub(crate) fn run_tui(db: &Db) -> Result<()> {
    let mut guard = TerminalGuard::new()?;
    let mut state = TuiState::new(crate::db::fetch_memos(db, None)?);

    let result = run_tui_loop(guard.terminal_mut(), db, &mut state);
    let _ = drain_pending_events();
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
    let mut first_error: Option<anyhow::Error> = None;
    if let Err(err) = disable_raw_mode() {
        first_error = Some(err.into());
    }
    if keyboard_enhanced {
        if let Err(err) = execute!(terminal.backend_mut(), PopKeyboardEnhancementFlags) {
            if first_error.is_none() {
                first_error = Some(err.into());
            }
        }
    }
    if let Err(err) = execute!(
        terminal.backend_mut(),
        DisableMouseCapture,
        LeaveAlternateScreen
    ) {
        if first_error.is_none() {
            first_error = Some(err.into());
        }
    }
    if let Err(err) = terminal.show_cursor() {
        if first_error.is_none() {
            first_error = Some(err.into());
        }
    }
    if let Some(err) = first_error {
        return Err(err);
    }
    Ok(())
}

fn run_tui_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    db: &Db,
    state: &mut TuiState,
) -> Result<()> {
    loop {
        terminal.draw(|frame| draw_tui(frame, state))?;
        if !poll_event()? {
            continue;
        }
        match event::read()? {
            Event::Key(key) => {
                if handle_tui_key(db, state, key)? {
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

fn drain_pending_events() -> Result<()> {
    while event::poll(std::time::Duration::from_millis(0))? {
        let _ = event::read();
    }
    Ok(())
}
