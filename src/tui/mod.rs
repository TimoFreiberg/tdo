mod events;
mod ui;

use anyhow::Result;
use crossterm::terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::{Terminal, TerminalOptions, Viewport};

use crate::storage::Store;
use crate::todo::Todo;

const MAX_HEIGHT: u16 = 20;

pub struct App {
    pub store: Store,
    pub todos: Vec<Todo>,
    pub cursor: usize,
    pub mode: Mode,
    pub show_all: bool,
}

pub enum Mode {
    Normal,
    NewTodo { input: String },
    ConfirmDelete { id: String, title: String },
}

impl App {
    pub fn new(store: Store) -> Self {
        let todos: Vec<Todo> = store.list_open().into_iter().cloned().collect();
        App {
            store,
            todos,
            cursor: 0,
            mode: Mode::Normal,
            show_all: false,
        }
    }

    pub fn reload(&mut self) {
        self.todos = if self.show_all {
            self.store.list_all().to_vec()
        } else {
            self.store.list_open().into_iter().cloned().collect()
        };
        if self.cursor >= self.todos.len() && !self.todos.is_empty() {
            self.cursor = self.todos.len() - 1;
        }
    }

    pub fn cursor_down(&mut self) {
        if !self.todos.is_empty() && self.cursor < self.todos.len() - 1 {
            self.cursor += 1;
        }
    }

    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Viewport height: number of todo items + 2 (border) + 1 (help line), capped at MAX_HEIGHT.
    pub fn viewport_height(&self) -> u16 {
        // 2 for top/bottom border, 1 for help line
        let content_lines = self.todos.len().min(u16::MAX as usize) as u16;
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

    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(height),
        },
    )?;

    let mut app = app;
    let result = events::run_event_loop(&mut terminal, &mut app);

    // Disable explicitly so cursor positioning works in cooked mode.
    // The guard will no-op on drop since raw mode is already off.
    terminal::disable_raw_mode()?;
    let viewport = terminal.get_frame().area();
    crossterm::execute!(
        std::io::stdout(),
        crossterm::cursor::MoveTo(0, viewport.y + viewport.height)
    )?;
    println!();
    result
}
