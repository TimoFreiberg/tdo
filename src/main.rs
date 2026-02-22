mod cli;
mod ops;
mod storage;
mod todo;
mod tui;
mod util;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Command};
use storage::Store;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let is_tty = util::stdout_is_tty();
    let dir = Store::resolve_dir(cli.dir.as_deref());
    let mut store = Store::open(&dir)?;

    match cli::resolve_command(&cli, is_tty) {
        Command::Create { title, body } => {
            let id = ops::create_todo(&mut store, &title, body.as_deref())?;
            println!("{id}");
        }
        Command::Edit { id, body } => {
            let interactive = is_tty && body.is_none();
            ops::edit_todo(&mut store, &id, body.as_deref(), interactive)?;
        }
        Command::Done(id) => {
            let todo = ops::mark_done(&mut store, &id)?;
            eprintln!("done: {}  {}", todo.id, todo.title());
        }
        Command::Reopen(id) => {
            let todo = ops::reopen_todo(&mut store, &id)?;
            eprintln!("reopened: {}  {}", todo.id, todo.title());
        }
        Command::Delete { id, force } => match ops::delete_todo(&mut store, &id, is_tty, force)? {
            Some(todo) => eprintln!("deleted: {}  {}", todo.id, todo.title()),
            None => eprintln!("cancelled"),
        },
        Command::Assign { id, name } => {
            let todo = ops::assign_todo(&mut store, &id, name.as_deref())?;
            eprintln!("assigned: {}  {}", todo.id, todo.title());
        }
        Command::Unassign(id) => {
            let todo = ops::unassign_todo(&mut store, &id)?;
            eprintln!("unassigned: {}  {}", todo.id, todo.title());
        }
        Command::List { all } => ops::list_todos(&mut store, all)?,
        Command::PlainList => ops::list_todos(&mut store, false)?,
        Command::Tui => tui::run_tui(store)?,
    }
    Ok(())
}
