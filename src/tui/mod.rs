mod events;
mod ui;

use anyhow::Result;
use crossterm::terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::{Terminal, TerminalOptions, Viewport};

use crate::storage::Store;
use crate::todo::Todo;

const MAX_HEIGHT: u16 = 20;

/// RAII guard to ensure raw mode is disabled on exit.
struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

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
    pub fn new(store: Store) -> Result<Self> {
        let mut app = App {
            store,
            todos: Vec::new(),
            cursor: 0,
            mode: Mode::Normal,
            show_all: false,
        };
        app.reload()?;
        Ok(app)
    }

    pub fn reload(&mut self) -> Result<()> {
        self.todos = if self.show_all {
            self.store.list_all()?
        } else {
            self.store.list_open()?
        };
        if self.cursor >= self.todos.len() && !self.todos.is_empty() {
            self.cursor = self.todos.len() - 1;
        }
        Ok(())
    }

    pub fn selected(&self) -> Option<&Todo> {
        self.todos.get(self.cursor)
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
        let content_lines = self.todos.len() as u16;
        // 2 for top/bottom border, 1 for help line
        (content_lines + 3).min(MAX_HEIGHT)
    }
}

pub fn run_tui(store: Store) -> Result<()> {
    terminal::enable_raw_mode()?;
    let guard = RawModeGuard;

    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);

    let mut app = App::new(store)?;
    let height = app.viewport_height();

    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(height),
        },
    )?;

    let result = events::run_event_loop(&mut terminal, &mut app);

    // Disable raw mode (guard ensures this even on early returns above)
    drop(guard);

    // Move cursor below the inline viewport so the last frame stays in scrollback
    let viewport = terminal.get_frame().area();
    let _ = crossterm::execute!(
        std::io::stdout(),
        crossterm::cursor::MoveTo(0, viewport.y + viewport.height)
    );
    println!();

    result
}
