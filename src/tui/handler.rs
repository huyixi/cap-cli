use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rusqlite::Connection;

use super::state::{Focus, TuiState};

pub(crate) fn handle_tui_key(
    conn: &Connection,
    state: &mut TuiState,
    key: KeyEvent,
) -> Result<bool> {
    match (key.code, key.modifiers) {
        (KeyCode::Char('c'), KeyModifiers::CONTROL) | (KeyCode::Esc, _) => Ok(true),
        (KeyCode::Char('q'), _) | (KeyCode::Char('Q'), _)
            if matches!(state.focus, Focus::History) =>
        {
            Ok(true)
        }
        (KeyCode::Tab, _) => {
            state.toggle_focus();
            Ok(false)
        }
        (KeyCode::Char('/'), _) if matches!(state.focus, Focus::History) => {
            state.activate_search();
            Ok(false)
        }
        (KeyCode::Enter, modifiers)
            if modifiers.contains(KeyModifiers::SUPER)
                || modifiers.contains(KeyModifiers::CONTROL)
                || modifiers.contains(KeyModifiers::ALT) =>
        {
            if matches!(state.focus, Focus::Input) && !state.input.is_empty() {
                crate::add_memo(conn, &state.input.text())?;
                refresh_history(conn, state)?;
                state.input.clear();
            }
            Ok(false)
        }
        (KeyCode::Up, _) if matches!(state.focus, Focus::History) => {
            state.move_history_selection_up();
            Ok(false)
        }
        (KeyCode::Down, _) if matches!(state.focus, Focus::History) => {
            state.move_history_selection_down();
            Ok(false)
        }
        (KeyCode::Char('k'), _) if matches!(state.focus, Focus::History) => {
            state.move_history_selection_up();
            Ok(false)
        }
        (KeyCode::Char('j'), _) if matches!(state.focus, Focus::History) => {
            state.move_history_selection_down();
            Ok(false)
        }
        (KeyCode::Enter, _) => {
            if matches!(state.focus, Focus::Input) {
                state.input.newline();
            }
            Ok(false)
        }
        (KeyCode::Backspace, _) => {
            match state.focus {
                Focus::Input => state.input.backspace(),
                Focus::Search => {
                    state.search.backspace();
                    state.apply_search();
                }
                Focus::History => {}
            }
            Ok(false)
        }
        (KeyCode::Char(ch), _) => {
            match state.focus {
                Focus::Input => state.input.insert_char(ch),
                Focus::Search => {
                    state.search.insert_char(ch);
                    state.apply_search();
                }
                Focus::History => {}
            }
            Ok(false)
        }
        _ => Ok(false),
    }
}

fn refresh_history(conn: &Connection, state: &mut TuiState) -> Result<()> {
    let history = crate::fetch_memos(conn, None)?;
    state.set_history(history);
    Ok(())
}
