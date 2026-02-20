use std::io::{self, BufRead, Write};

use anyhow::{bail, Context, Result};
use jiff::civil::DateTime;

use crate::storage::Store;
use crate::todo::{Frontmatter, Status, Todo};

/// Create a new todo, returning the assigned ID.
pub fn create_todo(store: &mut Store, title: &str) -> Result<String> {
    let now: DateTime = DateTime::try_from(jiff::Zoned::now())?;
    let fm = Frontmatter {
        title: title.to_string(),
        created: now,
        status: Status::Open,
    };
    store.create(&fm, None)
}

/// Mark a todo as done. Returns the updated todo.
pub fn mark_done(store: &mut Store, id: &str) -> Result<Todo> {
    let mut todo = store.find_by_id(id)?;
    todo.frontmatter.status = Status::Done;
    store.save(&todo)?;
    Ok(todo)
}

/// Reopen a done todo. Returns the updated todo.
pub fn reopen_todo(store: &mut Store, id: &str) -> Result<Todo> {
    let mut todo = store.find_by_id(id)?;
    todo.frontmatter.status = Status::Open;
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
/// - With --title/--body: update fields directly
/// - Interactive: spawn $VISUAL/$EDITOR
/// - Non-interactive without flags: error
pub fn edit_todo(
    store: &mut Store,
    id: &str,
    new_title: Option<&str>,
    new_body: Option<&str>,
    interactive: bool,
) -> Result<()> {
    if new_title.is_some() || new_body.is_some() {
        let mut todo = store.find_by_id(id)?;
        if let Some(t) = new_title {
            todo.frontmatter.title = t.to_string();
        }
        if let Some(b) = new_body {
            todo.body = if b.is_empty() {
                None
            } else {
                Some(b.to_string())
            };
        }
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
        bail!("cannot open editor non-interactively; use --title or --body");
    }
}

/// Print todos to stdout.
pub fn list_todos(store: &Store, all: bool) -> Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    if all {
        for todo in store.list_all() {
            if !todo.is_open() {
                writeln!(out, "{}  [done] {}", todo.id, todo.title())?;
            } else {
                writeln!(out, "{}  {}", todo.id, todo.title())?;
            }
        }
    } else {
        for todo in store.list_open() {
            writeln!(out, "{}  {}", todo.id, todo.title())?;
        }
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
