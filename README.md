# tdo

A local todo manager. Stores todos as markdown files in `.todo/`.

## Usage

```
tdo                          # TUI (interactive) or list open todos (non-interactive)
tdo some text here           # Create a todo with title "some text here"
tdo --edit <id>              # Open todo in $EDITOR
tdo --done <id>              # Mark todo as done
tdo --delete <id>            # Delete todo file
tdo --list                   # List open todos
tdo --list --all             # List all todos including done
```

### Flags

```
--json                       # Machine-readable JSON output
--dir <path>                 # Override .todo/ directory location
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

Files are named by creation timestamp (`YYYYMMDD-HHMMSS.md`) and stored in
`.todo/` relative to the current directory.
