# tdo — Architecture

## Overview

`tdo` is a local todo manager that stores items as markdown files with YAML
frontmatter in a `.todo/` directory. It has two interfaces: a TUI for
interactive terminal use, and a plain-text CLI for automation (e.g. driving
from Claude Code).

## Storage

Todos live in `.todo/` relative to the working directory (overridable with
`--dir`). Each todo is a single markdown file.

### Filename / ID scheme

Files are named by creation timestamp: `YYYYMMDD-HHMMSS.md`.

If a file for that second already exists, append a counter:
`YYYYMMDD-HHMMSS-1.md`, `YYYYMMDD-HHMMSS-2.md`, etc.

The filename (without `.md`) is the todo's ID. IDs are designed to be
selectable via double-click in a terminal.

### File format

```markdown
---
title: some text here
created: 2026-02-20T14:30:52
status: open
---

Optional body content added via editor.
```

- `title` — short description, set at creation time
- `created` — ISO 8601 timestamp
- `status` — `open` or `done`

## CLI design

### Modes

- **No args, interactive terminal** (`isatty(stdout)`) → launch TUI
- **No args, non-interactive** → print open todos as plain text
- **With args** → CLI operation (create, edit, done, delete, list)

### Operations

| Command | Description |
|---|---|
| `tdo <text>` | Create a new todo with the given title. Prints the new ID to stdout |
| `tdo --edit <id>` | Open the todo file in `$EDITOR` (fallback: `vim`). Must also support non-interactive editing (e.g. `--edit <id> --title <text>` or accepting new content on stdin) so Claude Code can rewrite todos without spawning an editor |
| `tdo --done <id>` | Mark a todo as done |
| `tdo --delete <id>` | Delete a todo file (confirms if interactive) |
| `tdo --list` | List open todos |
| `tdo --list --all` | List all todos including done |

### Global flags

| Flag | Description |
|---|---|
| `--dir <path>` | Override the `.todo/` directory location |

## TUI

Minimal v1 interface built with `ratatui` + `crossterm`:

- List open todos (sorted by timestamp, newest last)
- `j`/`k` or arrow keys to navigate
- `Enter` to open in `$EDITOR`
- `d` to mark done
- `n` to create new (prompts for title)
- `q` to quit

## Dependencies

| Crate | Purpose |
|---|---|
| `clap` | CLI argument parsing |
| `ratatui` | Terminal UI framework |
| `crossterm` | Terminal backend for ratatui |
| `serde` | Serialization framework |
| `serde_yaml` | YAML frontmatter parsing |
| `jiff` | Timestamps |
