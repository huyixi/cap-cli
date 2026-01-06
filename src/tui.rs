use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use rusqlite::Connection;
use std::io;

const HISTORY_LIMIT: usize = 10;
const TUI_POLL_MS: u64 = 200;

pub(crate) fn run_tui(conn: &Connection) -> Result<()> {
    let mut terminal = setup_terminal()?;
    let mut state = TuiState::new(crate::fetch_recent_memos(conn, HISTORY_LIMIT)?);

    let result = run_tui_loop(&mut terminal, conn, &mut state);
    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
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
        if let Event::Key(key) = event::read()? {
            if handle_tui_key(conn, state, key)? {
                break;
            }
        }
    }
    Ok(())
}

fn draw_tui(frame: &mut Frame<'_>, state: &TuiState) {
    let show_search = matches!(state.focus, Focus::Search) || !state.search.query.is_empty();
    let areas = if show_search {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
                Constraint::Length(1),
            ])
            .split(frame.area())
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(frame.area())
    };
    let (input_area, history_area, search_area) = if show_search {
        (areas[0], areas[1], Some(areas[2]))
    } else {
        (areas[0], areas[1], None)
    };

    if let Some(area) = search_area {
        let search_style = if matches!(state.focus, Focus::Search) {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        };
        let search_line = Line::from(format!("/{}", state.search.query));
        let search_widget = Paragraph::new(search_line)
            .style(search_style)
            .wrap(Wrap { trim: false });
        frame.render_widget(search_widget, area);
        if matches!(state.focus, Focus::Search) {
            frame.set_cursor_position(state.search.cursor_position_inline(area));
        }
    }

    let input_lines: Vec<Line> = state
        .input
        .lines
        .iter()
        .map(|line| Line::from(line.as_str()))
        .collect();
    let input_title = match state.input.status.as_deref() {
        Some(status) => format!(
            "Memo Input{} (Cmd/Ctrl+Enter submit, Tab switch, Esc/q exit) - {}",
            if matches!(state.focus, Focus::Input) {
                " [active]"
            } else {
                ""
            },
            status
        ),
        None => format!(
            "Memo Input{} (Cmd/Ctrl+Enter submit, Tab switch, Esc/q exit)",
            if matches!(state.focus, Focus::Input) {
                " [active]"
            } else {
                ""
            }
        ),
    };
    let input_border_style = if matches!(state.focus, Focus::Input) {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };
    let input_widget = Paragraph::new(Text::from(input_lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(input_title)
                .border_style(input_border_style),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(input_widget, input_area);
    if matches!(state.focus, Focus::Input) {
        frame.set_cursor_position(state.input.cursor_position(input_area));
    }

    let history_items: Vec<ListItem> = state
        .history
        .iter()
        .map(|(created_at, content)| ListItem::new(format!("{}  {}", created_at, content)))
        .collect();
    let history_title = if matches!(state.focus, Focus::History) {
        "Recent Memos [active] (Tab switch, / search)".to_string()
    } else {
        "Recent Memos (Tab switch)".to_string()
    };
    let history_border_style = if matches!(state.focus, Focus::History) {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };
    let history_highlight_style = if matches!(state.focus, Focus::History) {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };
    let history_highlight_symbol = "";
    let history_widget = List::new(history_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(history_title)
                .border_style(history_border_style),
        )
        .highlight_symbol(history_highlight_symbol)
        .highlight_style(history_highlight_style)
        .style(Style::default());
    let mut list_state = ListState::default();
    list_state.select(state.history_index);
    frame.render_stateful_widget(history_widget, history_area, &mut list_state);
}

fn poll_event() -> Result<bool> {
    Ok(event::poll(std::time::Duration::from_millis(TUI_POLL_MS))?)
}

fn handle_tui_key(conn: &Connection, state: &mut TuiState, key: KeyEvent) -> Result<bool> {
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
                state.all_history = crate::fetch_recent_memos(conn, HISTORY_LIMIT)?;
                state.apply_search();
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

#[derive(Copy, Clone)]
enum Focus {
    Search,
    Input,
    History,
}

struct TuiState {
    search: SearchState,
    input: InputState,
    history: Vec<(String, String)>,
    all_history: Vec<(String, String)>,
    focus: Focus,
    history_index: Option<usize>,
}

impl TuiState {
    fn new(history: Vec<(String, String)>) -> Self {
        let mut state = Self {
            search: SearchState::new(),
            input: InputState::new(),
            history: Vec::new(),
            all_history: history,
            focus: Focus::Input,
            history_index: None,
        };
        state.apply_search();
        state
    }

    fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Search => Focus::History,
            Focus::History => Focus::Input,
            Focus::Input => Focus::History,
        };
    }

    fn activate_search(&mut self) {
        self.focus = Focus::Search;
        self.search.clear();
        self.apply_search();
    }

    fn first_history_index(&self) -> Option<usize> {
        if self.history.is_empty() {
            None
        } else {
            Some(0)
        }
    }

    fn apply_search(&mut self) {
        if self.search.query.is_empty() {
            self.history = self.all_history.clone();
        } else {
            let needle = self.search.query.to_lowercase();
            self.history = self
                .all_history
                .iter()
                .filter(|(created_at, content)| {
                    content.to_lowercase().contains(&needle)
                        || created_at.to_lowercase().contains(&needle)
                })
                .cloned()
                .collect();
        }
        self.history_index = self.first_history_index();
    }

    fn move_history_selection_up(&mut self) {
        let Some(current) = self.history_index else {
            self.history_index = self.first_history_index();
            return;
        };
        if current > 0 {
            self.history_index = Some(current - 1);
        }
    }

    fn move_history_selection_down(&mut self) {
        let Some(current) = self.history_index else {
            self.history_index = self.first_history_index();
            return;
        };
        let max_index = self.history.len().saturating_sub(1);
        if current < max_index {
            self.history_index = Some(current + 1);
        }
    }
}

struct SearchState {
    query: String,
}

impl SearchState {
    fn new() -> Self {
        Self {
            query: String::new(),
        }
    }

    fn insert_char(&mut self, ch: char) {
        self.query.push(ch);
    }

    fn backspace(&mut self) {
        self.query.pop();
    }

    fn clear(&mut self) {
        self.query.clear();
    }

    fn cursor_position_inline(&self, area: Rect) -> (u16, u16) {
        let col = self.query.chars().count() as u16;
        (area.x + col + 1, area.y)
    }
}

struct InputState {
    lines: Vec<String>,
    status: Option<String>,
}

impl InputState {
    fn new() -> Self {
        Self {
            lines: vec![String::new()],
            status: None,
        }
    }

    fn insert_char(&mut self, ch: char) {
        if let Some(line) = self.lines.last_mut() {
            line.push(ch);
        }
        self.status = None;
    }

    fn backspace(&mut self) {
        if let Some(line) = self.lines.last_mut() {
            if line.pop().is_some() {
                self.status = None;
                return;
            }
        }
        if self.lines.len() > 1 {
            self.lines.pop();
            self.status = None;
        }
    }

    fn newline(&mut self) {
        self.lines.push(String::new());
        self.status = None;
    }

    fn clear(&mut self) {
        self.lines.clear();
        self.lines.push(String::new());
        self.status = None;
    }

    fn text(&self) -> String {
        self.lines.join("\n")
    }

    fn cursor_position(&self, area: Rect) -> (u16, u16) {
        let row = self.lines.len().saturating_sub(1) as u16;
        let col = self
            .lines
            .last()
            .map(|line| line.chars().count() as u16)
            .unwrap_or(0);
        (area.x + col + 1, area.y + row + 1)
    }

    fn is_empty(&self) -> bool {
        self.lines.len() == 1 && self.lines[0].is_empty()
    }
}
