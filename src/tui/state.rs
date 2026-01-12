use ratatui::layout::Rect;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Focus {
    Search,
    Input,
    History,
}

pub(crate) struct TuiState {
    pub(crate) search: SearchState,
    pub(crate) input: InputState,
    pub(crate) history: Vec<(String, String)>,
    all_history: Vec<(String, String)>,
    pub(crate) focus: Focus,
    pub(crate) history_index: Option<usize>,
}

impl TuiState {
    pub(crate) fn new(history: Vec<(String, String)>) -> Self {
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

    pub(crate) fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Search => Focus::History,
            Focus::History => Focus::Input,
            Focus::Input => Focus::History,
        };
    }

    pub(crate) fn activate_search(&mut self) {
        self.focus = Focus::Search;
        self.search.clear();
        self.apply_search();
    }

    pub(crate) fn set_history(&mut self, history: Vec<(String, String)>) {
        self.all_history = history;
        self.apply_search();
    }

    pub(crate) fn apply_search(&mut self) {
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

    pub(crate) fn move_history_selection_up(&mut self) {
        let Some(current) = self.history_index else {
            self.history_index = self.first_history_index();
            return;
        };
        if current > 0 {
            self.history_index = Some(current - 1);
        }
    }

    pub(crate) fn move_history_selection_down(&mut self) {
        let Some(current) = self.history_index else {
            self.history_index = self.first_history_index();
            return;
        };
        let max_index = self.history.len().saturating_sub(1);
        if current < max_index {
            self.history_index = Some(current + 1);
        }
    }

    pub(crate) fn is_search_visible(&self) -> bool {
        matches!(self.focus, Focus::Search) || !self.search.query.is_empty()
    }

    fn first_history_index(&self) -> Option<usize> {
        if self.history.is_empty() {
            None
        } else {
            Some(0)
        }
    }
}

pub(crate) struct SearchState {
    pub(crate) query: String,
}

impl SearchState {
    fn new() -> Self {
        Self {
            query: String::new(),
        }
    }

    pub(crate) fn insert_char(&mut self, ch: char) {
        self.query.push(ch);
    }

    pub(crate) fn backspace(&mut self) {
        self.query.pop();
    }

    pub(crate) fn clear(&mut self) {
        self.query.clear();
    }

    pub(crate) fn cursor_position_inline(&self, area: Rect) -> (u16, u16) {
        let col = UnicodeWidthStr::width(self.query.as_str()) as u16;
        (area.x + col + 1, area.y)
    }
}

pub(crate) struct InputState {
    pub(crate) lines: Vec<String>,
    pub(crate) status: Option<String>,
    cursor: InputCursor,
}

impl InputState {
    fn new() -> Self {
        Self {
            lines: vec![String::new()],
            status: None,
            cursor: InputCursor::new(),
        }
    }

    pub(crate) fn insert_char(&mut self, ch: char) {
        self.ensure_invariants();
        let line = &mut self.lines[self.cursor.line];
        let byte_index = byte_index_at_char(line, self.cursor.col);
        line.insert(byte_index, ch);
        self.cursor.col = self.cursor.col.saturating_add(1);
        self.reset_edit_state();
    }

    pub(crate) fn backspace(&mut self) {
        self.ensure_invariants();
        if self.cursor.col > 0 {
            let line = &mut self.lines[self.cursor.line];
            let remove_at = byte_index_at_char(line, self.cursor.col.saturating_sub(1));
            if let Some((byte_len, _)) = line[remove_at..]
                .chars()
                .next()
                .map(|ch| (ch.len_utf8(), ch))
            {
                line.replace_range(remove_at..remove_at + byte_len, "");
            }
            self.cursor.col = self.cursor.col.saturating_sub(1);
            self.reset_edit_state();
            return;
        }
        if self.cursor.line > 0 {
            let current_line = self.lines.remove(self.cursor.line);
            self.cursor.line = self.cursor.line.saturating_sub(1);
            let line = &mut self.lines[self.cursor.line];
            let prev_len = line.chars().count();
            line.push_str(&current_line);
            self.cursor.col = prev_len;
            self.reset_edit_state();
        }
    }

    pub(crate) fn delete_char(&mut self) {
        self.ensure_invariants();
        let line_len = self.current_line_len();
        if self.cursor.col < line_len {
            let line = &mut self.lines[self.cursor.line];
            let remove_at = byte_index_at_char(line, self.cursor.col);
            if let Some((byte_len, _)) = line[remove_at..]
                .chars()
                .next()
                .map(|ch| (ch.len_utf8(), ch))
            {
                line.replace_range(remove_at..remove_at + byte_len, "");
            }
            self.reset_edit_state();
            return;
        }
        if self.cursor.line + 1 < self.lines.len() {
            let next_line = self.lines.remove(self.cursor.line + 1);
            self.lines[self.cursor.line].push_str(&next_line);
            self.reset_edit_state();
        }
    }

    pub(crate) fn newline(&mut self) {
        self.ensure_invariants();
        let line = &mut self.lines[self.cursor.line];
        let split_at = byte_index_at_char(line, self.cursor.col);
        let tail = line[split_at..].to_string();
        line.truncate(split_at);
        let insert_at = self.cursor.line + 1;
        self.lines.insert(insert_at, tail);
        self.cursor.line = insert_at;
        self.cursor.col = 0;
        self.reset_edit_state();
    }

    pub(crate) fn clear(&mut self) {
        self.lines.clear();
        self.lines.push(String::new());
        self.cursor = InputCursor::new();
        self.status = None;
    }

    pub(crate) fn text(&self) -> String {
        self.lines.join("\n")
    }

    pub(crate) fn cursor_position(&self, area: Rect) -> (u16, u16) {
        let content_width = area.width.saturating_sub(2).max(1) as usize;
        let (row, col) = wrapped_cursor_position(&self.lines, &self.cursor, content_width);
        (area.x + col as u16 + 1, area.y + row as u16 + 1)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.lines.len() == 1 && self.lines[0].is_empty()
    }

    pub(crate) fn move_left(&mut self) {
        self.ensure_invariants();
        if self.cursor.col > 0 {
            self.cursor.col = self.cursor.col.saturating_sub(1);
        } else if self.cursor.line > 0 {
            self.cursor.line = self.cursor.line.saturating_sub(1);
            self.cursor.col = self.current_line_len();
        }
        self.cursor.preferred_col = None;
    }

    pub(crate) fn move_right(&mut self) {
        self.ensure_invariants();
        let line_len = self.current_line_len();
        if self.cursor.col < line_len {
            self.cursor.col = self.cursor.col.saturating_add(1);
        } else if self.cursor.line + 1 < self.lines.len() {
            self.cursor.line = self.cursor.line.saturating_add(1);
            self.cursor.col = 0;
        }
        self.cursor.preferred_col = None;
    }

    pub(crate) fn move_up(&mut self) {
        self.ensure_invariants();
        if self.cursor.line == 0 {
            return;
        }
        let target_col = self.cursor.preferred_col.unwrap_or(self.cursor.col);
        self.cursor.line = self.cursor.line.saturating_sub(1);
        self.cursor.col = target_col.min(self.current_line_len());
        self.cursor.preferred_col = Some(target_col);
    }

    pub(crate) fn move_down(&mut self) {
        self.ensure_invariants();
        if self.cursor.line + 1 >= self.lines.len() {
            return;
        }
        let target_col = self.cursor.preferred_col.unwrap_or(self.cursor.col);
        self.cursor.line = self.cursor.line.saturating_add(1);
        self.cursor.col = target_col.min(self.current_line_len());
        self.cursor.preferred_col = Some(target_col);
    }

    fn ensure_invariants(&mut self) {
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        if self.cursor.line >= self.lines.len() {
            self.cursor.line = self.lines.len().saturating_sub(1);
        }
        let line_len = self.current_line_len();
        if self.cursor.col > line_len {
            self.cursor.col = line_len;
        }
    }

    fn current_line_len(&self) -> usize {
        self.lines
            .get(self.cursor.line)
            .map(|line| line.chars().count())
            .unwrap_or(0)
    }

    fn reset_edit_state(&mut self) {
        self.cursor.preferred_col = None;
        self.status = None;
    }
}

struct InputCursor {
    line: usize,
    col: usize,
    preferred_col: Option<usize>,
}

impl InputCursor {
    fn new() -> Self {
        Self {
            line: 0,
            col: 0,
            preferred_col: None,
        }
    }
}

fn byte_index_at_char(value: &str, char_index: usize) -> usize {
    if char_index == 0 {
        return 0;
    }
    value
        .char_indices()
        .nth(char_index)
        .map(|(idx, _)| idx)
        .unwrap_or_else(|| value.len())
}

fn width_up_to_char(value: &str, char_index: usize) -> usize {
    value
        .chars()
        .take(char_index)
        .map(|ch| UnicodeWidthChar::width(ch).unwrap_or(0))
        .sum()
}

fn wrapped_cursor_position(
    lines: &[String],
    cursor: &InputCursor,
    content_width: usize,
) -> (usize, usize) {
    let mut rows_before = 0usize;
    let cursor_line = cursor.line.min(lines.len().saturating_sub(1));
    for line in lines.iter().take(cursor_line) {
        let line_width = UnicodeWidthStr::width(line.as_str());
        let wrapped_rows = if line_width == 0 {
            0
        } else {
            (line_width - 1) / content_width
        };
        rows_before += wrapped_rows + 1;
    }

    let line = lines
        .get(cursor_line)
        .map(String::as_str)
        .unwrap_or("");
    let cursor_col = cursor.col.min(line.chars().count());
    let prefix_width = width_up_to_char(line, cursor_col);
    let row_in_line = prefix_width / content_width;
    let col_in_line = prefix_width % content_width;
    let row = rows_before.saturating_add(row_in_line);
    let col = col_in_line;

    (row, col)
}
