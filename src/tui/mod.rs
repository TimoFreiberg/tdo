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

pub struct App {
    pub store: Store,
    pub todos: Vec<Todo>,
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
        let todos: Vec<Todo> = store.list_open().into_iter().cloned().collect();
        let mut list_state = ListState::default();
        // Index 0 is always the input field
        list_state.select(Some(0));
        App {
            store,
            todos,
            list_state,
            mode: Mode::Normal,
            show_all: false,
            input: String::new(),
        }
    }

    /// Returns the selected todo, if the cursor is on a todo (index > 0).
    /// Index 0 is the input field and returns None.
    pub fn selected_todo(&self) -> Option<&Todo> {
        self.list_state
            .selected()
            .filter(|&i| i > 0)
            .and_then(|i| self.todos.get(i - 1))
    }

    /// Whether the cursor is on the input field (index 0).
    pub fn on_input(&self) -> bool {
        self.list_state.selected() == Some(0)
    }

    pub fn reload(&mut self) {
        self.todos = if self.show_all {
            self.store.list_all().to_vec()
        } else {
            self.store.list_open().into_iter().cloned().collect()
        };
        // Total items = input field + todos
        let total = self.todos.len() + 1;
        match total {
            0 => unreachable!("always at least the input field"),
            n => {
                let clamped = self
                    .list_state
                    .selected()
                    .map(|i| i.min(n - 1))
                    .unwrap_or(0);
                self.list_state.select(Some(clamped));
            }
        }
    }

    pub fn cursor_down(&mut self) {
        // Total items = input field + todos
        let total = self.todos.len() + 1;
        if total > 0 {
            self.list_state.select_next();
        }
    }

    pub fn cursor_up(&mut self) {
        if self.list_state.selected().is_some() {
            self.list_state.select_previous();
        }
    }

    /// Viewport height: (input field + todo items) + 2 (border) + 1 (help line), capped at MAX_HEIGHT.
    pub fn viewport_height(&self) -> u16 {
        // +1 for the input field row
        let content_lines = (self.todos.len() + 1).min(u16::MAX as usize) as u16;
        content_lines.saturating_add(3).min(MAX_HEIGHT)
    }
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
