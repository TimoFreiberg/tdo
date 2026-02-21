---
name: tdo
description: "Manage TODOs with the tdo CLI. Handles natural-language requests like 'create a todo for that' or 'mark that done'."
argument-hint: "[add <title> | list | done <query> | reopen <query> | edit <query> | delete <query>]"
---

## Operations

Parse `$ARGUMENTS` and dispatch:

| Argument pattern | Action |
|---|---|
| *(empty)* | Run `tdo list`. If there are any, ask the user what they want to do next. If none, say so. |
| `add <title>` | Create a new todo |
| `list` | Run `tdo list` (open only) or `tdo list --all` (include done) |
| `done <query>` | Mark a todo as done |
| `reopen <query>` | Reopen a done todo |
| `edit <query>` | Edit a todo's body |
| `delete <query>` | Delete a todo |
| Free-form text without a known verb | Treat as `add <text>` |

## Add

Run `tdo add <title words>`. It prints the assigned 4-char hex ID to stdout. Confirm creation to the user.

Titles are immutable after creation. To change a title, delete and recreate.

If the user provided additional context beyond the title, follow up with `tdo edit <id> --body "details"`.

## Matching queries to IDs

If the query is a hex ID or prefix (e.g. `a3f9`, `a3`), use it directly. Otherwise, run `tdo list --all`, match by title substring, and disambiguate with AskUserQuestion if needed.

## Done / Reopen / Edit / Delete

Match the query to an ID (see above), then run the command:

- `tdo done <id>`
- `tdo reopen <id>`
- `tdo edit <id> --body "new body content"` (`--body` required for non-interactive use)
- `tdo delete <id> --force` (`--force` required for non-interactive use)

Confirm the result to the user.
