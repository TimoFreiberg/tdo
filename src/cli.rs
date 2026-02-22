use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "tdo", version, about = "A local todo manager", after_help = "Titles are immutable after creation. To change a title, delete and recreate the todo.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<SubCommand>,

    /// Override .todo/ directory location
    #[arg(long, global = true, value_name = "PATH")]
    pub dir: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum SubCommand {
    /// Create a new todo
    Add {
        /// Words that become the title
        #[arg(required = true)]
        text: Vec<String>,
        /// Set body text (supports \n for newlines)
        #[arg(long, value_name = "TEXT")]
        body: Option<String>,
    },
    /// Open a todo in $EDITOR, or update body with --body
    Edit {
        /// Todo ID (or unique prefix)
        id: String,
        /// Set new body non-interactively
        #[arg(long, value_name = "TEXT")]
        body: Option<String>,
    },
    /// Mark a todo as done
    Done {
        /// Todo ID (or unique prefix)
        id: String,
    },
    /// Reopen a done todo
    Reopen {
        /// Todo ID (or unique prefix)
        id: String,
    },
    /// Delete a todo
    Delete {
        /// Todo ID (or unique prefix)
        id: String,
        /// Delete without confirmation (non-interactive)
        #[arg(long)]
        force: bool,
    },
    /// List todos
    List {
        /// Include done todos
        #[arg(long)]
        all: bool,
    },
    /// Assign a todo
    Assign {
        /// Todo ID (or unique prefix)
        id: String,
        /// Optional assignee name
        name: Option<String>,
    },
    /// Unassign a todo
    Unassign {
        /// Todo ID (or unique prefix)
        id: String,
    },
}

pub enum Command {
    Create { title: String, body: Option<String> },
    Edit {
        id: String,
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
    Assign { id: String, name: Option<String> },
    Unassign(String),
    Tui,
    PlainList,
}

pub fn resolve_command(cli: &Cli, is_tty: bool) -> Command {
    match &cli.command {
        Some(SubCommand::Add { text, body }) => Command::Create {
            title: text.join(" "),
            body: body.clone(),
        },
        Some(SubCommand::Edit { id, body }) => Command::Edit {
            id: id.clone(),
            body: body.clone(),
        },
        Some(SubCommand::Done { id }) => Command::Done(id.clone()),
        Some(SubCommand::Reopen { id }) => Command::Reopen(id.clone()),
        Some(SubCommand::Delete { id, force }) => Command::Delete {
            id: id.clone(),
            force: *force,
        },
        Some(SubCommand::List { all }) => Command::List { all: *all },
        Some(SubCommand::Assign { id, name }) => Command::Assign {
            id: id.clone(),
            name: name.clone(),
        },
        Some(SubCommand::Unassign { id }) => Command::Unassign(id.clone()),
        None if is_tty => Command::Tui,
        None => Command::PlainList,
    }
}
