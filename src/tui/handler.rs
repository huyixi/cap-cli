use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::state::{Focus, TuiState};
use crate::{
    db::{self, Db},
    domain::memo::NewMemo,
};

pub(crate) fn handle_tui_key(db: &Db, state: &mut TuiState, key: KeyEvent) -> Result<bool> {
    if key.kind == KeyEventKind::Release {
        return Ok(false);
    }
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
        (KeyCode::Enter, modifiers) if modifiers.contains(KeyModifiers::CONTROL) => {
            submit_input_if_ready(db, state)?;
            Ok(false)
        }
        (KeyCode::Char('\n' | '\r'), modifiers) if modifiers.contains(KeyModifiers::CONTROL) => {
            submit_input_if_ready(db, state)?;
            Ok(false)
        }
        (KeyCode::Char('m' | 'j'), modifiers) if modifiers.contains(KeyModifiers::CONTROL) => {
            submit_input_if_ready(db, state)?;
            Ok(false)
        }
        (KeyCode::Enter, _) => {
            insert_newline_if_input_focus(state);
            Ok(false)
        }
        (KeyCode::Char('\n' | '\r'), _) => {
            insert_newline_if_input_focus(state);
            Ok(false)
        }
        (KeyCode::Up, _) if matches!(state.focus, Focus::History) => {
            state.move_history_selection_up();
            Ok(false)
        }
        (KeyCode::Up, _) if matches!(state.focus, Focus::Input) => {
            state.input.move_up();
            Ok(false)
        }
        (KeyCode::Down, _) if matches!(state.focus, Focus::History) => {
            state.move_history_selection_down();
            Ok(false)
        }
        (KeyCode::Down, _) if matches!(state.focus, Focus::Input) => {
            state.input.move_down();
            Ok(false)
        }
        (KeyCode::Left, _) if matches!(state.focus, Focus::Input) => {
            state.input.move_left();
            Ok(false)
        }
        (KeyCode::Right, _) if matches!(state.focus, Focus::Input) => {
            state.input.move_right();
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
        (KeyCode::Delete, _) if matches!(state.focus, Focus::Input) => {
            state.input.delete_char();
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

fn refresh_history(db: &Db, state: &mut TuiState) -> Result<()> {
    let history = db::fetch_memos(db, None)?;
    state.set_history(history);
    Ok(())
}

fn insert_newline_if_input_focus(state: &mut TuiState) {
    if matches!(state.focus, Focus::Input) {
        state.input.newline();
    }
}

fn submit_input_if_ready(db: &Db, state: &mut TuiState) -> Result<()> {
    if !matches!(state.focus, Focus::Input) {
        return Ok(());
    }
    if state.input.is_empty() {
        return Ok(());
    }
    let new_memo = NewMemo::new(state.input.text());
    db::add_memo(db, &new_memo)?;
    refresh_history(db, state)?;
    state.input.clear();
    Ok(())
}
