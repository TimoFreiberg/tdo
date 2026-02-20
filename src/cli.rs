use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "tdo",
    about = "A local todo manager",
    override_usage = "tdo [TEXT]...\n       tdo --edit <ID> [--title <TEXT>] [--body <TEXT>]\n       tdo --done <ID>\n       tdo --reopen <ID>\n       tdo --delete <ID> [--force]\n       tdo --list [--all]"
)]
pub struct Cli {
    /// Words that become the title of a new todo
    #[arg(trailing_var_arg = true)]
    pub text: Vec<String>,

    /// Open todo in $EDITOR (or update non-interactively with --title/--body)
    #[arg(long, value_name = "ID", conflicts_with_all = ["done", "reopen", "delete", "list"])]
    pub edit: Option<String>,

    /// Mark a todo as done
    #[arg(long, value_name = "ID", conflicts_with_all = ["edit", "reopen", "delete", "list"])]
    pub done: Option<String>,

    /// Reopen a done todo
    #[arg(long, value_name = "ID", conflicts_with_all = ["edit", "done", "delete", "list"])]
    pub reopen: Option<String>,

    /// Delete a todo
    #[arg(long, value_name = "ID", conflicts_with_all = ["edit", "done", "reopen", "list"])]
    pub delete: Option<String>,

    /// Delete without confirmation (non-interactive)
    #[arg(long, requires = "delete")]
    pub force: bool,

    /// List todos
    #[arg(long, conflicts_with_all = ["edit", "done", "reopen", "delete"])]
    pub list: bool,

    /// With --list: include done todos
    #[arg(long, requires = "list")]
    pub all: bool,

    /// With --edit: set new title non-interactively
    #[arg(long, value_name = "TEXT", requires = "edit")]
    pub title: Option<String>,

    /// With --edit: set new body non-interactively
    #[arg(long, value_name = "TEXT", requires = "edit")]
    pub body: Option<String>,

    /// Override .todo/ directory location
    #[arg(long, value_name = "PATH")]
    pub dir: Option<PathBuf>,
}

pub enum Command {
    Create(String),
    Edit {
        id: String,
        title: Option<String>,
        body: Option<String>,
    },
    Done(String),
    Reopen(String),
    Delete {
        id: String,
        force: bool,
    },
    List {
        all: bool,
    },
    Tui,
    PlainList,
}

pub fn resolve_command(cli: &Cli, is_tty: bool) -> Command {
    if !cli.text.is_empty() {
        return Command::Create(cli.text.join(" "));
    }
    if let Some(id) = &cli.edit {
        return Command::Edit {
            id: id.clone(),
            title: cli.title.clone(),
            body: cli.body.clone(),
        };
    }
    if let Some(id) = &cli.done {
        return Command::Done(id.clone());
    }
    if let Some(id) = &cli.reopen {
        return Command::Reopen(id.clone());
    }
    if let Some(id) = &cli.delete {
        return Command::Delete {
            id: id.clone(),
            force: cli.force,
        };
    }
    if cli.list {
        return Command::List { all: cli.all };
    }
    if is_tty {
        Command::Tui
    } else {
        Command::PlainList
    }
}
