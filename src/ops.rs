use std::io::{self, BufRead, Write};

use anyhow::{bail, Context, Result};
use jiff::civil::DateTime;

use crate::storage::Store;
use crate::todo::{Frontmatter, Status, Todo};
use crate::util::stdout_is_tty;

/// Create a new todo, returning the assigned ID.
pub fn create_todo(store: &mut Store, title: &str) -> Result<String> {
    let now: DateTime = DateTime::try_from(jiff::Zoned::now())?;
    let fm = Frontmatter {
        title: title.to_string(),
        created: now,
        status: Status::Open,
        assigned: None,
        assigned_at: None,
        done_at: None,
    };
    store.create(&fm, None)
}

/// Mark a todo as done. Returns the updated todo.
pub fn mark_done(store: &mut Store, id: &str) -> Result<Todo> {
    let mut todo = store.find_by_id(id)?;
    todo.frontmatter.status = Status::Done;
    if todo.frontmatter.done_at.is_none() {
        let now: DateTime = DateTime::try_from(jiff::Zoned::now())?;
        todo.frontmatter.done_at = Some(now);
    }
    store.save(&todo)?;
    Ok(todo)
}

/// Reopen a done todo. Returns the updated todo.
pub fn reopen_todo(store: &mut Store, id: &str) -> Result<Todo> {
    let mut todo = store.find_by_id(id)?;
    todo.frontmatter.status = Status::Open;
    todo.frontmatter.done_at = None;
    store.save(&todo)?;
    Ok(todo)
}

/// Delete a todo. Returns Some(deleted_todo) on success, None if cancelled.
///
/// - Interactive mode: prompts for confirmation
/// - Non-interactive mode: requires `force` flag
pub fn delete_todo(
    store: &mut Store,
    id: &str,
    interactive: bool,
    force: bool,
) -> Result<Option<Todo>> {
    let todo = store.find_by_id(id)?;
    if interactive {
        eprint!("Delete '{}'? [y/N] ", todo.title());
        io::stderr().flush()?;
        let mut line = String::new();
        io::stdin().lock().read_line(&mut line)?;
        if !line.trim().eq_ignore_ascii_case("y") {
            return Ok(None);
        }
    } else if !force {
        bail!("use --force to delete non-interactively");
    }
    let deleted = store.delete(&todo.id)?;
    Ok(Some(deleted))
}

/// Edit a todo.
///
/// - With --body: update body directly
/// - Interactive: spawn $VISUAL/$EDITOR
/// - Non-interactive without flags: error
pub fn edit_todo(
    store: &mut Store,
    id: &str,
    new_body: Option<&str>,
    interactive: bool,
) -> Result<()> {
    if let Some(b) = new_body {
        let mut todo = store.find_by_id(id)?;
        todo.body = if b.is_empty() {
            None
        } else {
            Some(b.to_string())
        };
        store.save(&todo)
    } else if interactive {
        let todo = store.find_by_id(id)?;
        let path = store.path_for(&todo);
        let editor = resolve_editor();
        let status = std::process::Command::new(&editor)
            .arg(&path)
            .status()
            .with_context(|| {
                format!(
                    "failed to run editor '{editor}': is it installed? Set $VISUAL or $EDITOR"
                )
            })?;
        if !status.success() {
            bail!("editor exited with status {status}");
        }
        store.refresh(&todo.id)?;
        Ok(())
    } else {
        bail!("cannot open editor non-interactively; use --body");
    }
}

/// Assign a todo. Returns the updated todo.
pub fn assign_todo(store: &mut Store, id: &str, name: Option<&str>) -> Result<Todo> {
    let mut todo = store.find_by_id(id)?;
    todo.frontmatter.assigned = Some(name.unwrap_or("").to_string());
    if todo.frontmatter.assigned_at.is_none() {
        let now: DateTime = DateTime::try_from(jiff::Zoned::now())?;
        todo.frontmatter.assigned_at = Some(now);
    }
    store.save(&todo)?;
    Ok(todo)
}

/// Unassign a todo. Returns the updated todo.
pub fn unassign_todo(store: &mut Store, id: &str) -> Result<Todo> {
    let mut todo = store.find_by_id(id)?;
    todo.frontmatter.assigned = None;
    todo.frontmatter.assigned_at = None;
    store.save(&todo)?;
    Ok(todo)
}

/// ANSI escape helpers — only used when stdout is a TTY.
const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[36m";
const MAGENTA: &str = "\x1b[35m";
const RESET: &str = "\x1b[0m";

/// Print todos to stdout, with color when connected to a terminal.
pub fn list_todos(store: &Store, all: bool) -> Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let color = stdout_is_tty();
    if all {
        for todo in store.list_all() {
            if todo.is_open() {
                write_todo(&mut out, todo, color)?;
            } else {
                write_done_todo(&mut out, todo, color)?;
            }
        }
    } else {
        for todo in store.list_open() {
            write_todo(&mut out, todo, color)?;
        }
    }
    Ok(())
}

fn write_todo(out: &mut impl Write, todo: &Todo, color: bool) -> Result<()> {
    let assigned_suffix = match &todo.frontmatter.assigned {
        Some(name) if !name.is_empty() => format!(" (assigned: {name})"),
        Some(_) => " (assigned)".to_string(),
        None => String::new(),
    };
    if color {
        write!(out, "{CYAN}{}{RESET}  {}", todo.id, todo.title())?;
        if !assigned_suffix.is_empty() {
            write!(out, "{MAGENTA}{assigned_suffix}{RESET}")?;
        }
        writeln!(out)?;
    } else {
        writeln!(out, "{}  {}{}", todo.id, todo.title(), assigned_suffix)?;
    }
    Ok(())
}

fn write_done_todo(out: &mut impl Write, todo: &Todo, color: bool) -> Result<()> {
    let assigned_suffix = match &todo.frontmatter.assigned {
        Some(name) if !name.is_empty() => format!(" (assigned: {name})"),
        Some(_) => " (assigned)".to_string(),
        None => String::new(),
    };
    if color {
        write!(out, "{DIM}{}  [done] {}", todo.id, todo.title())?;
        if !assigned_suffix.is_empty() {
            write!(out, "{assigned_suffix}")?;
        }
        writeln!(out, "{RESET}")?;
    } else {
        writeln!(out, "{}  [done] {}{}", todo.id, todo.title(), assigned_suffix)?;
    }
    Ok(())
}

fn resolve_editor() -> String {
    std::env::var("VISUAL")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("EDITOR").ok().filter(|s| !s.is_empty()))
        .unwrap_or_else(|| "vim".to_string())
}
