use std::io::Stdout;
use std::ops::ControlFlow;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use super::{App, Mode};
use crate::ops;

pub fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| super::ui::draw(f, app))?;
        if let Event::Key(key) = event::read()? {
            if handle_key(terminal, app, key)? == ControlFlow::Break(()) {
                return Ok(());
            }
            app.reload();
        }
    }
}

fn handle_key(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    key: KeyEvent,
) -> Result<ControlFlow<()>> {
    match &app.mode {
        Mode::Normal => handle_normal(terminal, app, key),
        Mode::NewTodo { .. } => handle_new_todo(app, key),
        Mode::ConfirmDelete { .. } => handle_confirm_delete(app, key),
    }
}

fn handle_normal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    key: KeyEvent,
) -> Result<ControlFlow<()>> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => return Ok(ControlFlow::Break(())),
        KeyCode::Char('j') | KeyCode::Down => app.cursor_down(),
        KeyCode::Char('k') | KeyCode::Up => app.cursor_up(),
        KeyCode::Char('d') => {
            if let Some(todo) = app.todos.get(app.cursor) {
                let id = todo.id.clone();
                let is_open = todo.is_open();
                if is_open {
                    ops::mark_done(&mut app.store, &id)?;
                } else {
                    ops::reopen_todo(&mut app.store, &id)?;
                }
            }
        }
        KeyCode::Char('x') => {
            if let Some(todo) = app.todos.get(app.cursor) {
                let id = todo.id.clone();
                let title = todo.title().to_string();
                app.mode = Mode::ConfirmDelete { id, title };
            }
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            if let Some(todo) = app.todos.get(app.cursor) {
                let id = todo.id.clone();
                // Suspend TUI for editor
                crossterm::terminal::disable_raw_mode()?;
                let viewport = terminal.get_frame().area();
                crossterm::execute!(
                    std::io::stdout(),
                    crossterm::cursor::MoveTo(0, viewport.y + viewport.height),
                    crossterm::cursor::Show,
                )?;

                let edit_result = ops::edit_todo(&mut app.store, &id, None, None, true);

                // Resume TUI regardless of editor result
                crossterm::terminal::enable_raw_mode()?;
                terminal.clear()?;

                edit_result?;
            }
        }
        KeyCode::Char('n') => {
            app.mode = Mode::NewTodo {
                input: String::new(),
            };
        }
        KeyCode::Char('a') => {
            app.show_all = !app.show_all;
            app.reload();
        }
        _ => {}
    }
    Ok(ControlFlow::Continue(()))
}

fn handle_new_todo(app: &mut App, key: KeyEvent) -> Result<ControlFlow<()>> {
    if let Mode::NewTodo { ref mut input } = app.mode {
        match key.code {
            KeyCode::Enter => {
                if !input.is_empty() {
                    ops::create_todo(&mut app.store, &input.clone())?;
                }
                app.mode = Mode::Normal;
            }
            KeyCode::Esc => {
                app.mode = Mode::Normal;
            }
            KeyCode::Char(c) => input.push(c),
            KeyCode::Backspace => {
                input.pop();
            }
            _ => {}
        }
    }
    Ok(ControlFlow::Continue(()))
}

fn handle_confirm_delete(app: &mut App, key: KeyEvent) -> Result<ControlFlow<()>> {
    let id = if let Mode::ConfirmDelete { ref id, .. } = app.mode {
        id.clone()
    } else {
        return Ok(ControlFlow::Continue(()));
    };

    match key.code {
        KeyCode::Char('y') => {
            app.store.delete(&id)?;
            app.mode = Mode::Normal;
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            app.mode = Mode::Normal;
        }
        _ => {}
    }
    Ok(ControlFlow::Continue(()))
}
