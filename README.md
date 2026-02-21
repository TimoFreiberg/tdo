# tdo

A local todo manager. Stores todos as markdown files in `.todo/`.

## Usage

```
tdo                          # TUI (interactive) or list open todos (non-interactive)
tdo some text here           # Create a todo with title "some text here"
tdo --edit <id>              # Open todo in $VISUAL/$EDITOR
tdo --done <id>              # Mark todo as done
tdo --reopen <id>            # Reopen a done todo
tdo --delete <id>            # Delete todo file (prompts if interactive)
tdo --delete <id> --force    # Delete without confirmation
tdo --list                   # List open todos
tdo --list --all             # List all todos including done
tdo --assign <id> [name]     # Assign todo (optionally to a person)
tdo --unassign <id>          # Remove assignment from todo
```

ID arguments accept unique prefixes (e.g. `a3` instead of `a3f9`).

### Flags

```
--dir <path>                 # Override .todo/ directory location
--force                      # Skip confirmation (with --delete)
```

## Todo format

```markdown
---
title: some text here
created: 2026-02-20T14:30:52
status: open
assigned: someone
assigned_at: 2026-02-21T22:58:00
done_at: 2026-02-21T23:00:00
---

Optional body content.
```

The `assigned`, `assigned_at`, and `done_at` fields are optional and omitted
when not set. `assigned_at` records the timestamp when the todo was first
assigned. `done_at` records the timestamp when the todo was first marked done.
When listing todos, assigned todos show a magenta `(assigned)` or
`(assigned: name)` suffix.

Files are named `<hex>-<slug>.md` (e.g. `a3f9-fix-the-login-bug.md`) and
stored in `.todo/` relative to the current directory. The hex prefix is the
ID used in commands.

Titles are immutable after creation — the filename slug is derived from the
title at creation time and is not updated. If a title needs to change, delete
the todo and create a new one.

## Disclaimer

This codebase is 100% vibecoded and intended for personal use.
