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
    let store = Store::open(&dir)?;

    match cli::resolve_command(&cli, is_tty) {
        Command::Create(title) => {
            let id = ops::create_todo(&store, &title)?;
            println!("{id}");
        }
        Command::Edit { id, title, body } => {
            let interactive = is_tty && title.is_none() && body.is_none();
            ops::edit_todo(&store, &id, title.as_deref(), body.as_deref(), interactive)?;
        }
        Command::Done(id) => ops::mark_done(&store, &id)?,
        Command::Delete(id) => ops::delete_todo(&store, &id, is_tty)?,
        Command::List { all } => ops::list_todos(&store, all)?,
        Command::PlainList => ops::list_todos(&store, false)?,
        Command::Tui => tui::run_tui(store)?,
    }
    Ok(())
}
