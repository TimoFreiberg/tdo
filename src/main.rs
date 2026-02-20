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
        Command::Create(title) => {
            let id = ops::create_todo(&mut store, &title)?;
            println!("{id}");
        }
        Command::Edit { id, title, body } => {
            let interactive = is_tty && title.is_none() && body.is_none();
            ops::edit_todo(&mut store, &id, title.as_deref(), body.as_deref(), interactive)?;
        }
        Command::Done(id) => {
            let (full_id, title) = ops::mark_done(&mut store, &id)?;
            eprintln!("done: {full_id}  {title}");
        }
        Command::Reopen(id) => {
            let (full_id, title) = ops::reopen_todo(&mut store, &id)?;
            eprintln!("reopened: {full_id}  {title}");
        }
        Command::Delete { id, force } => {
            match ops::delete_todo(&mut store, &id, is_tty, force)? {
                Some((full_id, title)) => eprintln!("deleted: {full_id}  {title}"),
                None => eprintln!("cancelled"),
            }
        }
        Command::List { all } => ops::list_todos(&mut store, all)?,
        Command::PlainList => ops::list_todos(&mut store, false)?,
        Command::Tui => tui::run_tui(store)?,
    }
    Ok(())
}
