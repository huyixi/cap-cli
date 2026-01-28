use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::state::{Focus, TuiState};
use crate::{
    db::{self, Db},
    domain::memo::NewMemo,
};

#[derive(Clone, Copy, Debug)]
enum Action {
    Quit,
    ToggleFocus,
    ActivateSearch,
    SubmitInput,
    InsertNewline,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Backspace,
    Delete,
    InsertChar(char),
}

pub(crate) fn handle_tui_key(db: &Db, state: &mut TuiState, key: KeyEvent) -> Result<bool> {
    if key.kind == KeyEventKind::Release {
        return Ok(false);
    }
    match key_to_action(&key, state.focus) {
        Some(action) => apply_action(db, state, action),
        None => Ok(false),
    }
}

fn key_to_action(key: &KeyEvent, focus: Focus) -> Option<Action> {
    let code = key.code;
    let modifiers = key.modifiers;

    if matches!(
        (code, modifiers),
        (KeyCode::Char('c'), KeyModifiers::CONTROL) | (KeyCode::Esc, _)
    ) {
        return Some(Action::Quit);
    }

    if matches!(focus, Focus::History) && matches!(code, KeyCode::Char('q') | KeyCode::Char('Q')) {
        return Some(Action::Quit);
    }

    if matches!(code, KeyCode::Tab) {
        return Some(Action::ToggleFocus);
    }

    if matches!(focus, Focus::History) && matches!(code, KeyCode::Char('/')) {
        return Some(Action::ActivateSearch);
    }

    if is_submit_key(code, modifiers) {
        return Some(Action::SubmitInput);
    }

    if is_newline_key(code) {
        return Some(Action::InsertNewline);
    }

    match code {
        KeyCode::Up => Some(Action::MoveUp),
        KeyCode::Down => Some(Action::MoveDown),
        KeyCode::Left => Some(Action::MoveLeft),
        KeyCode::Right => Some(Action::MoveRight),
        KeyCode::Char('k') if matches!(focus, Focus::History) => Some(Action::MoveUp),
        KeyCode::Char('j') if matches!(focus, Focus::History) => Some(Action::MoveDown),
        KeyCode::Backspace => Some(Action::Backspace),
        KeyCode::Delete if matches!(focus, Focus::Input) => Some(Action::Delete),
        KeyCode::Char(ch) => match focus {
            Focus::History => None,
            Focus::Input | Focus::Search => Some(Action::InsertChar(ch)),
        },
        _ => None,
    }
}

fn apply_action(db: &Db, state: &mut TuiState, action: Action) -> Result<bool> {
    match action {
        Action::Quit => Ok(true),
        Action::ToggleFocus => {
            state.toggle_focus();
            Ok(false)
        }
        Action::ActivateSearch => {
            state.activate_search();
            Ok(false)
        }
        Action::SubmitInput => {
            submit_input_if_ready(db, state)?;
            Ok(false)
        }
        Action::InsertNewline => {
            insert_newline_if_input_focus(state);
            Ok(false)
        }
        Action::MoveUp => {
            match state.focus {
                Focus::History => state.move_history_selection_up(),
                Focus::Input => state.input.move_up(),
                Focus::Search => {}
            }
            Ok(false)
        }
        Action::MoveDown => {
            match state.focus {
                Focus::History => state.move_history_selection_down(),
                Focus::Input => state.input.move_down(),
                Focus::Search => {}
            }
            Ok(false)
        }
        Action::MoveLeft => {
            if matches!(state.focus, Focus::Input) {
                state.input.move_left();
            }
            Ok(false)
        }
        Action::MoveRight => {
            if matches!(state.focus, Focus::Input) {
                state.input.move_right();
            }
            Ok(false)
        }
        Action::Backspace => {
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
        Action::Delete => {
            if matches!(state.focus, Focus::Input) {
                state.input.delete_char();
            }
            Ok(false)
        }
        Action::InsertChar(ch) => {
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
    }
}

fn is_submit_key(code: KeyCode, modifiers: KeyModifiers) -> bool {
    if !modifiers.contains(KeyModifiers::CONTROL) {
        return false;
    }
    matches!(
        code,
        KeyCode::Enter
            | KeyCode::Char('\n')
            | KeyCode::Char('\r')
            | KeyCode::Char('m')
            | KeyCode::Char('j')
    )
}

fn is_newline_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::Enter | KeyCode::Char('\n') | KeyCode::Char('\r'))
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
