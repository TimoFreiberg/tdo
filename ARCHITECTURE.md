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

Files are named `<hex>-<slug>.md`, e.g. `a3f9-fix-the-login-bug.md`.

- **hex** — 4 random hex characters. This is the todo's ID, used in all
  commands (`tdo --done a3f9`). On collision, regenerate.
- **slug** — slugified title, set once at creation. Cosmetic only — makes
  `ls .todo/` browsable without opening files. Not updated on title edits.

Slugification: lowercase, replace non-alphanumeric runs with a single hyphen,
strip leading/trailing hyphens, truncate to a reasonable length (~50 chars).
If the slug is empty after sanitization, use the hex ID alone.

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
- `assigned` — optional assignee name; omitted when not set
- `assigned_at` — optional timestamp of first assignment; omitted when not set
- `done_at` — optional timestamp of first completion; omitted when not set

## CLI design

### Modes

- **No args, interactive terminal** (`isatty(stdout)`) → launch TUI
- **No args, non-interactive** → print open todos as plain text
- **With args** → CLI operation (create, edit, done, delete, list)

### Operations

| Command | Description |
|---|---|
| `tdo <text>` | Create a new todo with the given title. Prints the new ID to stdout |
| `tdo add <text> --body <body>` | Create a new todo with title and body. Prints the new ID to stdout |
| `tdo --edit <id>` | Open the todo file in `$EDITOR` (fallback: `vim`). Must also support non-interactive editing (e.g. `--edit <id> --title <text>` or accepting new content on stdin) so Claude Code can rewrite todos without spawning an editor |
| `tdo --done <id>` | Mark a todo as done |
| `tdo --delete <id>` | Delete a todo file (confirms if interactive) |
| `tdo --list` | List open todos (with GC) |
| `tdo --list --all` | List all todos including done (with GC) |
| `tdo --assign <id> [name]` | Assign a todo, optionally to a named person |
| `tdo --unassign <id>` | Remove assignment from a todo |
| `tdo view <id>` | View a todo's full details (ID, title, status, timestamps, body) |

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
- `Ctrl+S` to toggle assignment on selected todo
- `n` to create new (prompts for title)
- `q` to quit

Assigned todos are sorted last in the list and displayed with a magenta
`(assigned)` suffix (or `(assigned: name)` if a name is set).

## Garbage collection

The list operation (`--list` and bare non-interactive invocation) performs
automatic GC before printing:

1. **Stale done cleanup** — Done todos with `done_at` older than 7 days are
   deleted from disk and the store cache. A `♻ gc:` line is printed to stderr
   per deletion.
2. **Stale assignment warnings** — Open assigned todos with `assigned_at`
   older than 7 days emit a `⚠ stale assignment:` warning to stderr.

All GC output goes to stderr so stdout remains a clean list suitable for
piping.

## Dependencies

| Crate | Purpose |
|---|---|
| `clap` | CLI argument parsing |
| `ratatui` | Terminal UI framework |
| `crossterm` | Terminal backend for ratatui |
| `serde` | Serialization framework |
| `serde_yaml` | YAML frontmatter parsing |
| `jiff` | Timestamps |
