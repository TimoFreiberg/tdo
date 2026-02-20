use std::io::Stdout;
use std::ops::ControlFlow;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use ratatui::{Terminal, TerminalOptions, Viewport};

use super::{App, Mode};
use crate::ops;

pub fn run_event_loop(
    mut terminal: Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
) -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut current_height = app.viewport_height();
    loop {
        terminal.draw(|f| super::ui::draw(f, &mut *app))?;
        if let Event::Key(key) = event::read()? {
            if handle_key(&mut terminal, app, key)? == ControlFlow::Break(()) {
                return Ok(terminal);
            }
            app.reload();
            let new_height = app.viewport_height();
            if new_height > current_height {
                terminal = grow_viewport(terminal, new_height)?;
                current_height = new_height;
            }
        }
    }
}

fn grow_viewport(
    mut terminal: Terminal<CrosstermBackend<Stdout>>,
    new_height: u16,
) -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let area = terminal.get_frame().area();
    crossterm::execute!(
        std::io::stdout(),
        crossterm::cursor::MoveTo(0, area.y + area.height - 1),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine),
        crossterm::cursor::MoveTo(0, area.y),
    )?;
    drop(terminal);
    let backend = CrosstermBackend::new(std::io::stdout());
    let new_terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(new_height),
        },
    )?;
    Ok(new_terminal)
}

fn handle_key(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    key: KeyEvent,
) -> Result<ControlFlow<()>> {
    match &app.mode {
        Mode::Normal => handle_normal(terminal, app, key),
        Mode::ConfirmDelete { .. } => handle_confirm_delete(app, key),
    }
}

fn handle_normal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    key: KeyEvent,
) -> Result<ControlFlow<()>> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    // Ctrl+key shortcuts (work regardless of cursor position)
    if ctrl {
        match key.code {
            KeyCode::Char('q') => return Ok(ControlFlow::Break(())),
            KeyCode::Char('d') => {
                if let Some(todo) = app.selected_todo() {
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
                if let Some(todo) = app.selected_todo() {
                    let id = todo.id.clone();
                    let title = todo.title().to_string();
                    app.mode = Mode::ConfirmDelete { id, title };
                }
            }
            KeyCode::Char('a') => {
                app.show_all = !app.show_all;
                app.reload();
            }
            _ => {}
        }
        return Ok(ControlFlow::Continue(()));
    }

    // Non-ctrl keys: behavior depends on cursor position
    if app.on_input() {
        // Cursor is on the input field (index 0)
        match key.code {
            KeyCode::Enter => {
                if !app.input.is_empty() {
                    ops::create_todo(&mut app.store, &app.input.clone())?;
                    app.input.clear();
                    // Move cursor to the newly created todo (index 1)
                    app.list_state.select(Some(1));
                }
            }
            KeyCode::Esc => {
                if !app.input.is_empty() {
                    app.input.clear();
                } else {
                    return Ok(ControlFlow::Break(()));
                }
            }
            KeyCode::Char(c) => app.input.push(c),
            KeyCode::Backspace => {
                app.input.pop();
            }
            KeyCode::Down => app.cursor_down(),
            KeyCode::Up => app.cursor_up(),
            _ => {}
        }
    } else {
        // Cursor is on a todo item (index > 0)
        match key.code {
            KeyCode::Esc => return Ok(ControlFlow::Break(())),
            KeyCode::Down => app.cursor_down(),
            KeyCode::Up => app.cursor_up(),
            KeyCode::Enter => {
                if let Some(todo) = app.selected_todo() {
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

                    // Resume TUI. Check edit_result before resume errors so
                    // a more actionable editor error isn't swallowed.
                    let resume = crossterm::terminal::enable_raw_mode();
                    if resume.is_ok() {
                        let _ = terminal.clear();
                    }
                    edit_result?;
                    resume?;
                }
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
        KeyCode::Char('y') | KeyCode::Enter => {
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
