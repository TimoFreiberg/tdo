use std::io::{self, BufRead, Write};

use anyhow::{bail, Result};
use jiff::civil::DateTime;

use crate::storage::Store;
use crate::todo::{Frontmatter, Status};

/// Create a new todo, returning the assigned ID.
pub fn create_todo(store: &Store, title: &str) -> Result<String> {
    let now: DateTime = DateTime::try_from(jiff::Zoned::now())?;
    let fm = Frontmatter {
        title: title.to_string(),
        created: now,
        status: Status::Open,
    };
    store.create(&fm, None)
}

/// Mark a todo as done.
pub fn mark_done(store: &Store, id: &str) -> Result<()> {
    let mut todo = store.find_by_id(id)?;
    todo.frontmatter.status = Status::Done;
    store.save(&todo)
}

/// Delete a todo. If interactive, prompts for confirmation.
pub fn delete_todo(store: &Store, id: &str, interactive: bool) -> Result<()> {
    let todo = store.find_by_id(id)?;
    if interactive {
        eprint!("Delete '{}'? [y/N] ", todo.title());
        io::stderr().flush()?;
        let mut line = String::new();
        io::stdin().lock().read_line(&mut line)?;
        if !line.trim().eq_ignore_ascii_case("y") {
            bail!("cancelled");
        }
    }
    store.delete(&todo.id)
}

/// Edit a todo. If title/body are provided, update non-interactively.
/// Otherwise spawn $EDITOR.
pub fn edit_todo(
    store: &Store,
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
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
        let status = std::process::Command::new(&editor)
            .arg(&path)
            .status()?;
        if !status.success() {
            bail!("editor exited with status {status}");
        }
        Ok(())
    } else {
        // Non-interactive with no flags: nothing to do
        Ok(())
    }
}

/// Print todos to stdout.
pub fn list_todos(store: &Store, all: bool) -> Result<()> {
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
