mod events;
mod ui;

use anyhow::Result;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::storage::Store;
use crate::todo::Todo;

pub struct App {
    pub store: Store,
    pub todos: Vec<Todo>,
    pub cursor: usize,
    pub mode: Mode,
}

pub enum Mode {
    Normal,
    NewTodo { input: String },
}

impl App {
    pub fn new(store: Store) -> Result<Self> {
        let todos = store.list_open()?;
        Ok(App {
            store,
            todos,
            cursor: 0,
            mode: Mode::Normal,
        })
    }

    pub fn reload(&mut self) -> Result<()> {
        self.todos = self.store.list_open()?;
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
}

pub fn run_tui(store: Store) -> Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(store)?;
    let result = events::run_event_loop(&mut terminal, &mut app);

    terminal::disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    result
}
