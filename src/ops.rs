use std::io::{self, BufRead, Write};

use anyhow::{bail, Context, Result};
use jiff::civil::DateTime;

use crate::storage::Store;
use crate::todo::{Frontmatter, Status};

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

/// Mark a todo as done. Returns (full_id, title).
pub fn mark_done(store: &mut Store, id: &str) -> Result<(String, String)> {
    let mut todo = store.find_by_id(id)?;
    todo.frontmatter.status = Status::Done;
    let full_id = todo.id.clone();
    let title = todo.title().to_string();
    store.save(&todo)?;
    Ok((full_id, title))
}

/// Reopen a done todo. Returns (full_id, title).
pub fn reopen_todo(store: &mut Store, id: &str) -> Result<(String, String)> {
    let mut todo = store.find_by_id(id)?;
    todo.frontmatter.status = Status::Open;
    let full_id = todo.id.clone();
    let title = todo.title().to_string();
    store.save(&todo)?;
    Ok((full_id, title))
}

/// Delete a todo. If interactive, prompts for confirmation.
/// Returns Some((full_id, title)) if deleted, None if cancelled.
pub fn delete_todo(
    store: &mut Store,
    id: &str,
    interactive: bool,
    force: bool,
) -> Result<Option<(String, String)>> {
    let todo = store.find_by_id(id)?;
    let title = todo.title().to_string();
    let full_id = todo.id.clone();

    if interactive && !force {
        eprint!("Delete '{title}'? [y/N] ");
        io::stderr().flush()?;
        let mut line = String::new();
        io::stdin().lock().read_line(&mut line)?;
        if !line.trim().eq_ignore_ascii_case("y") {
            return Ok(None);
        }
    } else if !interactive && !force {
        bail!("use --force to delete non-interactively");
    }

    store.delete(&full_id)?;
    Ok(Some((full_id, title)))
}

/// Edit a todo. If title/body are provided, update non-interactively.
/// Otherwise spawn $VISUAL/$EDITOR.
pub fn edit_todo(
    store: &mut Store,
    id: &str,
    new_title: Option<&str>,
    new_body: Option<&str>,
    interactive: bool,
) -> Result<()> {
    let mut todo = store.find_by_id(id)?;
    if new_title.is_some() || new_body.is_some() {
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
        let path = store.path_for(&todo);
        let editor = std::env::var("VISUAL")
            .or_else(|_| std::env::var("EDITOR"))
            .unwrap_or_else(|_| "vi".to_string());
        let status = std::process::Command::new(&editor)
            .arg(&path)
            .status()
            .with_context(|| {
                format!("failed to launch editor '{editor}' (set $VISUAL or $EDITOR)")
            })?;
        if !status.success() {
            bail!("editor exited with status {status}");
        }
        Ok(())
    } else {
        bail!("cannot open editor non-interactively; use --title or --body");
    }
}

/// Print todos to stdout.
pub fn list_todos(store: &mut Store, all: bool) -> Result<()> {
    let todos = if all {
        store.list_all()?
    } else {
        store.list_open()?
    };
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for todo in &todos {
        if all && !todo.is_open() {
            writeln!(out, "{}  [done] {}", todo.id, todo.title())?;
        } else {
            writeln!(out, "{}  {}", todo.id, todo.title())?;
        }
    }
    Ok(())
}
