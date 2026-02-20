# tdo

A local todo manager. Stores todos as markdown files in `.todo/`.

## Usage

```
tdo                          # TUI (interactive) or list open todos (non-interactive)
tdo some text here           # Create a todo with title "some text here"
tdo --edit <id>              # Open todo in $VISUAL/$EDITOR
tdo --edit <id> --title X    # Update title non-interactively
tdo --done <id>              # Mark todo as done
tdo --reopen <id>            # Reopen a done todo
tdo --delete <id>            # Delete todo file (prompts if interactive)
tdo --delete <id> --force    # Delete without confirmation
tdo --list                   # List open todos
tdo --list --all             # List all todos including done
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
---

Optional body content.
```

Files are named `<hex>-<slug>.md` (e.g. `a3f9-fix-the-login-bug.md`) and
stored in `.todo/` relative to the current directory. The hex prefix is the
ID used in commands.
