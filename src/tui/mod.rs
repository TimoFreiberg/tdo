mod events;
mod ui;

use anyhow::Result;
use crossterm::terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::ListState;
use ratatui::{Terminal, TerminalOptions, Viewport};

use crate::storage::Store;
use crate::todo::Todo;

const MAX_HEIGHT: u16 = 20;

/// Returns the number of rows in the terminal, or `MAX_HEIGHT` if the query fails.
fn terminal_rows() -> u16 {
    terminal::size().map(|(_, rows)| rows).unwrap_or(MAX_HEIGHT)
}

pub struct App {
    pub store: Store,
    pub todos: Vec<Todo>,
    /// Indices into `todos` that match the current fuzzy query.
    pub filtered: Vec<usize>,
    /// Selection state for the items below the input field.
    pub list_state: ListState,
    pub mode: Mode,
    pub show_all: bool,
    pub input: String,
}

pub enum Mode {
    Normal,
    ConfirmDelete { id: String, title: String },
}

impl App {
    pub fn new(store: Store) -> Self {
        let mut todos: Vec<Todo> = store.list_open().into_iter().cloned().collect();
        todos.sort_by_key(|t| t.is_assigned());
        let filtered: Vec<usize> = (0..todos.len()).collect();
        let mut list_state = ListState::default();
        if !filtered.is_empty() {
            list_state.select(Some(0));
        }
        App {
            store,
            todos,
            filtered,
            list_state,
            mode: Mode::Normal,
            show_all: false,
            input: String::new(),
        }
    }

    /// Whether the "Create new" line is shown (input is non-empty).
    pub fn has_create_line(&self) -> bool {
        !self.input.is_empty()
    }

    /// The index in the selectable list where filtered todos start.
    fn todo_start_index(&self) -> usize {
        if self.has_create_line() { 1 } else { 0 }
    }

    /// Returns the selected todo, if the selection is on a filtered todo item.
    pub fn selected_todo(&self) -> Option<&Todo> {
        let sel = self.list_state.selected()?;
        let start = self.todo_start_index();
        if sel >= start {
            let filtered_idx = sel - start;
            self.filtered.get(filtered_idx).and_then(|&i| self.todos.get(i))
        } else {
            None
        }
    }

    /// Whether the selection is on the "Create new" line.
    pub fn is_on_create_new(&self) -> bool {
        self.has_create_line() && self.list_state.selected() == Some(0)
    }

    /// Number of selectable items (create_new + filtered todos).
    pub fn selectable_count(&self) -> usize {
        (if self.has_create_line() { 1 } else { 0 }) + self.filtered.len()
    }

    /// Recompute the filtered list based on current input and reset
    /// selection to the first match. Call this when the input text changes.
    pub fn refilter(&mut self) {
        self.compute_filtered();
        self.reset_selection();
    }

    /// Recompute `self.filtered` from current `todos` and `input`.
    fn compute_filtered(&mut self) {
        if self.input.is_empty() {
            self.filtered = (0..self.todos.len()).collect();
        } else {
            self.filtered = self
                .todos
                .iter()
                .enumerate()
                .filter(|(_, t)| {
                    let input_lower = self.input.to_lowercase();
                    t.id.to_lowercase().starts_with(&input_lower)
                        || fuzzy_match(&self.input, t.title())
                })
                .map(|(i, _)| i)
                .collect();
        }
    }

    /// Reset selection to the first match. Used when the filter changes.
    fn reset_selection(&mut self) {
        let start = self.todo_start_index();
        if !self.filtered.is_empty() {
            self.list_state.select(Some(start));
        } else if self.has_create_line() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
    }

    /// Clamp selection to valid bounds without resetting it. Used after
    /// reloading the store (e.g. after marking done or deleting) so that
    /// arrow-key movement is preserved.
    fn clamp_selection(&mut self) {
        let total = self.selectable_count();
        if total == 0 {
            self.list_state.select(None);
        } else if let Some(sel) = self.list_state.selected() {
            if sel >= total {
                self.list_state.select(Some(total - 1));
            }
        } else {
            // Nothing selected but items exist — select first.
            self.list_state.select(Some(0));
        }
    }

    /// Re-read the store and recompute filtered list, clamping (not
    /// resetting) the selection so arrow-key position is preserved.
    pub fn reload(&mut self) {
        self.todos = if self.show_all {
            self.store.list_all().to_vec()
        } else {
            self.store.list_open().into_iter().cloned().collect()
        };
        self.todos.sort_by_key(|t| t.is_assigned());
        self.compute_filtered();
        self.clamp_selection();
    }

    pub fn cursor_down(&mut self) {
        let total = self.selectable_count();
        if let Some(sel) = self.list_state.selected() {
            if sel + 1 < total {
                self.list_state.select(Some(sel + 1));
            }
        }
    }

    pub fn cursor_up(&mut self) {
        if let Some(sel) = self.list_state.selected() {
            if sel > 0 {
                self.list_state.select(Some(sel - 1));
            }
        }
    }

    /// Viewport height: (input + items) + 2 (border) + 1 (help line), capped at
    /// `MAX_HEIGHT` and the terminal height minus one row of margin.
    pub fn viewport_height(&self) -> u16 {
        let content_lines = (1 + self.selectable_count()).min(u16::MAX as usize) as u16;
        let cap = MAX_HEIGHT.min(terminal_rows().saturating_sub(1));
        content_lines.saturating_add(3).min(cap)
    }
}

/// Fuzzy subsequence match (case-insensitive).
fn fuzzy_match(query: &str, text: &str) -> bool {
    let mut text_chars = text.chars().flat_map(|c| c.to_lowercase());
    for qc in query.chars().flat_map(|c| c.to_lowercase()) {
        loop {
            match text_chars.next() {
                Some(tc) if tc == qc => break,
                Some(_) => continue,
                None => return false,
            }
        }
    }
    true
}

/// RAII guard that disables raw mode on drop.
struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

pub fn run_tui(store: Store) -> Result<()> {
    terminal::enable_raw_mode()?;
    let _raw_guard = RawModeGuard;

    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);

    let app = App::new(store);
    let height = app.viewport_height();

    let terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(height),
        },
    )?;

    let mut app = app;
    let result = events::run_event_loop(terminal, &mut app);

    // Disable explicitly so cursor positioning works in cooked mode.
    // The guard will no-op on drop since raw mode is already off.
    terminal::disable_raw_mode()?;
    match result {
        Ok(mut terminal) => {
            let viewport = terminal.get_frame().area();
            crossterm::execute!(
                std::io::stdout(),
                crossterm::cursor::MoveTo(0, viewport.y + viewport.height)
            )?;
            println!();
            Ok(())
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_match_basic() {
        assert!(fuzzy_match("fb", "foobar"));
        assert!(fuzzy_match("FB", "foobar"));
        assert!(fuzzy_match("fb", "FooBar"));
    }

    #[test]
    fn fuzzy_match_exact() {
        assert!(fuzzy_match("foo", "foo"));
    }

    #[test]
    fn fuzzy_match_empty_query() {
        assert!(fuzzy_match("", "anything"));
    }

    #[test]
    fn fuzzy_match_no_match() {
        assert!(!fuzzy_match("xyz", "foobar"));
        assert!(!fuzzy_match("ba", "abc"));
    }
}
