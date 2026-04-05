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
        match event::read()? {
            Event::Key(key) => match handle_key(&mut terminal, app, key)? {
                ControlFlow::Break(()) => return Ok(terminal),
                ControlFlow::Continue(dirty) => {
                    if dirty {
                        app.reload();
                    }
                }
            },
            Event::Resize(_, _) => {}
            _ => continue,
        }
        let new_height = app.viewport_height();
        if new_height != current_height {
            terminal = resize_viewport(terminal, new_height)?;
            current_height = new_height;
        }
    }
}

/// Create a fresh inline terminal, clearing the viewport area so old
/// content doesn't bleed through.
fn create_inline_terminal(
    cursor_y: u16,
    height: u16,
) -> Result<Terminal<CrosstermBackend<Stdout>>> {
    crossterm::execute!(std::io::stdout(), crossterm::cursor::MoveTo(0, cursor_y),)?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(height),
        },
    )?;
    let area = terminal.get_frame().area();
    crossterm::execute!(
        std::io::stdout(),
        crossterm::cursor::MoveTo(area.x, area.y),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown),
    )?;
    Ok(terminal)
}

fn resize_viewport(
    mut terminal: Terminal<CrosstermBackend<Stdout>>,
    new_height: u16,
) -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let area = terminal.get_frame().area();
    drop(terminal);
    create_inline_terminal(area.y, new_height)
}

fn handle_key(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    key: KeyEvent,
) -> Result<ControlFlow<(), bool>> {
    match &app.mode {
        Mode::Normal => handle_normal(terminal, app, key),
        Mode::ConfirmDelete { .. } => handle_confirm_delete(app, key),
    }
}

fn handle_normal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    key: KeyEvent,
) -> Result<ControlFlow<(), bool>> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    // Ctrl+key shortcuts (always active)
    if ctrl {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('c') => return Ok(ControlFlow::Break(())),
            KeyCode::Char('a') => {
                app.show_all = !app.show_all;
                app.reload();
                return Ok(ControlFlow::Continue(false));
            }
            KeyCode::Char('d') => {
                if let Some(todo) = app.selected_todo() {
                    let id = todo.id.clone();
                    let is_open = todo.is_open();
                    if is_open {
                        ops::mark_done(&mut app.store, &id)?;
                    } else {
                        ops::reopen_todo(&mut app.store, &id)?;
                    }
                    return Ok(ControlFlow::Continue(true));
                }
            }
            KeyCode::Char('s') => {
                if let Some(todo) = app.selected_todo() {
                    let id = todo.id.clone();
                    let is_assigned = todo.is_assigned();
                    if is_assigned {
                        ops::unassign_todo(&mut app.store, &id)?;
                    } else {
                        ops::assign_todo(&mut app.store, &id, None)?;
                    }
                    return Ok(ControlFlow::Continue(true));
                }
            }
            KeyCode::Char('x') => {
                if let Some(todo) = app.selected_todo() {
                    let id = todo.id.clone();
                    let title = todo.title().to_string();
                    app.mode = Mode::ConfirmDelete { id, title };
                }
            }
            _ => {}
        }
        return Ok(ControlFlow::Continue(false));
    }

    // Non-ctrl keys: input field is always active for typing
    match key.code {
        KeyCode::Char(c) => {
            app.input.push(c);
            app.refilter();
        }
        KeyCode::Backspace => {
            app.input.pop();
            app.refilter();
        }
        KeyCode::Enter => {
            if app.is_on_create_new() {
                ops::create_todo(&mut app.store, &app.input.clone(), None)?;
                app.input.clear();
                app.refilter();
                return Ok(ControlFlow::Continue(true));
            } else if let Some(todo) = app.selected_todo() {
                let id = todo.id.clone();
                // Suspend TUI for editor
                crossterm::terminal::disable_raw_mode()?;
                let viewport = terminal.get_frame().area();
                crossterm::execute!(
                    std::io::stdout(),
                    crossterm::cursor::MoveTo(0, viewport.y + viewport.height),
                    crossterm::cursor::Show,
                )?;

                let edit_result = ops::edit_todo(&mut app.store, &id, None, true);

                // Resume TUI. Check edit_result before resume errors so
                // a more actionable editor error isn't swallowed.
                let resume = crossterm::terminal::enable_raw_mode();
                edit_result?;
                resume?;

                // Recreate the terminal so it picks up the (possibly
                // changed) terminal size and redraws cleanly.
                let area = terminal.get_frame().area();
                *terminal = create_inline_terminal(area.y, app.viewport_height())?;
                return Ok(ControlFlow::Continue(true));
            }
        }
        KeyCode::Esc => {
            if !app.input.is_empty() {
                app.input.clear();
                app.refilter();
            } else {
                return Ok(ControlFlow::Break(()));
            }
        }
        KeyCode::Down => app.cursor_down(),
        KeyCode::Up => app.cursor_up(),
        _ => {}
    }
    Ok(ControlFlow::Continue(false))
}

fn handle_confirm_delete(app: &mut App, key: KeyEvent) -> Result<ControlFlow<(), bool>> {
    let id = if let Mode::ConfirmDelete { ref id, .. } = app.mode {
        id.clone()
    } else {
        return Ok(ControlFlow::Continue(false));
    };

    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => {
            app.store.delete(&id)?;
            app.mode = Mode::Normal;
            return Ok(ControlFlow::Continue(true));
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            app.mode = Mode::Normal;
        }
        _ => {}
    }
    Ok(ControlFlow::Continue(false))
}
