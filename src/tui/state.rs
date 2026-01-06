use ratatui::layout::Rect;

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
        let col = self.query.chars().count() as u16;
        (area.x + col + 1, area.y)
    }
}

pub(crate) struct InputState {
    pub(crate) lines: Vec<String>,
    pub(crate) status: Option<String>,
}

impl InputState {
    fn new() -> Self {
        Self {
            lines: vec![String::new()],
            status: None,
        }
    }

    pub(crate) fn insert_char(&mut self, ch: char) {
        if let Some(line) = self.lines.last_mut() {
            line.push(ch);
        }
        self.status = None;
    }

    pub(crate) fn backspace(&mut self) {
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

    pub(crate) fn newline(&mut self) {
        self.lines.push(String::new());
        self.status = None;
    }

    pub(crate) fn clear(&mut self) {
        self.lines.clear();
        self.lines.push(String::new());
        self.status = None;
    }

    pub(crate) fn text(&self) -> String {
        self.lines.join("\n")
    }

    pub(crate) fn cursor_position(&self, area: Rect) -> (u16, u16) {
        let row = self.lines.len().saturating_sub(1) as u16;
        let col = self
            .lines
            .last()
            .map(|line| line.chars().count() as u16)
            .unwrap_or(0);
        (area.x + col + 1, area.y + row + 1)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.lines.len() == 1 && self.lines[0].is_empty()
    }
}
