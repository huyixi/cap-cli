use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use super::state::{Focus, TuiState};

pub(crate) fn draw_tui(frame: &mut Frame<'_>, state: &TuiState) {
    let layout = split_layout(frame.area(), state.is_search_visible());

    draw_input(frame, state, layout.input_area);
    draw_history(frame, state, layout.history_area);
    if let Some(search_area) = layout.search_area {
        draw_search(frame, state, search_area);
    }
}

fn draw_input(frame: &mut Frame<'_>, state: &TuiState, area: Rect) {
    let input_lines: Vec<Line> = state
        .input
        .lines
        .iter()
        .map(|line| Line::from(line.as_str()))
        .collect();
    let input_title = format_input_title(state);
    let input_widget = Paragraph::new(Text::from(input_lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(input_title)
                .border_style(focus_style(state.focus, Focus::Input)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(input_widget, area);
    if matches!(state.focus, Focus::Input) {
        frame.set_cursor_position(state.input.cursor_position(area));
    }
}

fn draw_history(frame: &mut Frame<'_>, state: &TuiState, area: Rect) {
    let history_items: Vec<ListItem> = state
        .history
        .iter()
        .map(|(created_at, content)| ListItem::new(format!("{}  {}", created_at, content)))
        .collect();
    let history_widget = List::new(history_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(history_title(state))
                .border_style(focus_style(state.focus, Focus::History)),
        )
        .highlight_symbol("")
        .highlight_style(focus_style(state.focus, Focus::History))
        .style(Style::default());
    let mut list_state = ListState::default();
    list_state.select(state.history_index);
    frame.render_stateful_widget(history_widget, area, &mut list_state);
}

fn draw_search(frame: &mut Frame<'_>, state: &TuiState, area: Rect) {
    let search_style = focus_style(state.focus, Focus::Search);
    let search_line = Line::from(format!("/{}", state.search.query));
    let search_widget = Paragraph::new(search_line)
        .style(search_style)
        .wrap(Wrap { trim: false });
    frame.render_widget(search_widget, area);
    if matches!(state.focus, Focus::Search) {
        frame.set_cursor_position(state.search.cursor_position_inline(area));
    }
}

fn format_input_title(state: &TuiState) -> String {
    let active_label = if matches!(state.focus, Focus::Input) {
        " [active]"
    } else {
        ""
    };
    match state.input.status.as_deref() {
        Some(status) => format!(
            "Memo Input{} (Cmd/Ctrl+Enter submit, Tab switch, Esc exit) - {}",
            active_label, status
        ),
        None => format!(
            "Memo Input{} (Cmd/Ctrl+Enter submit, Tab switch, Esc exit)",
            active_label
        ),
    }
}

fn history_title(state: &TuiState) -> String {
    if matches!(state.focus, Focus::History) {
        "Recent Memos [active] (Tab switch, / search, q quit)".to_string()
    } else {
        "Recent Memos (Tab switch)".to_string()
    }
}

fn focus_style(current: Focus, target: Focus) -> Style {
    if current == target {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    }
}

struct LayoutAreas {
    input_area: Rect,
    history_area: Rect,
    search_area: Option<Rect>,
}

fn split_layout(area: Rect, show_search: bool) -> LayoutAreas {
    // Search is a single-line prompt shown beneath the history list (vim-style).
    if show_search {
        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
                Constraint::Length(1),
            ])
            .split(area);
        LayoutAreas {
            input_area: areas[0],
            history_area: areas[1],
            search_area: Some(areas[2]),
        }
    } else {
        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        LayoutAreas {
            input_area: areas[0],
            history_area: areas[1],
            search_area: None,
        }
    }
}
