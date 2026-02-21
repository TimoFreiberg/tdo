---
name: tdo
description: "Manage project TODOs using the `tdo` CLI. Use when the user wants to create, list, complete, reopen, edit, or delete todos — including natural-language requests like 'create a todo for that' or 'mark that as done'."
argument-hint: "[add <title> | list | done <query> | reopen <query> | edit <query> | delete <query>]"
---

# tdo — Local Todo Manager

`tdo` stores todos as markdown files with YAML frontmatter in a `.todo/` directory. Each todo has a 4-character hex ID (e.g. `a3f9`), a title, a status (`open` or `done`), and an optional body.

**Important:** All `tdo` commands are non-interactive when run from this agent. Use the flags documented below.

## Current state

Run `tdo list` to see open todos, or `tdo list --all` to include done items.

## Operations

Parse `$ARGUMENTS` and dispatch:

| Argument pattern | Action |
|---|---|
| *(empty)* | List open TODOs. If there are any, ask the user what they want to do next. If none, say so. |
| `add <title>` | Create a new todo (see "Add" below) |
| `list` | List open todos |
| `list --all` | List all todos including done |
| `done <query>` | Mark a todo as done (see "Done" below) |
| `reopen <query>` | Reopen a done todo (see "Reopen" below) |
| `edit <query>` | Edit a todo's body (see "Edit" below) |
| `delete <query>` | Delete a todo (see "Delete" below) |
| Free-form text without a known verb | Treat as `add <text>` |

## Add

Run:
```bash
tdo add <title words>
```

`tdo` prints the assigned ID to stdout. Confirm creation to the user with the ID and title.

If the user provided additional context beyond the title, immediately follow up with an edit to set the body:
```bash
tdo edit <id> --body "detailed notes here"
```

## List

Run:
```bash
tdo list          # open todos only
tdo list --all    # include done todos
```

Output format is `<ID>  <title>` (one per line), with done items shown as `<ID>  [done] <title>`.

Present the list to the user. If no todos exist, say so.

## Match (for done/reopen/edit/delete)

When `<query>` is provided:

1. If it looks like a hex ID or prefix (e.g. `a3f9`, `a3`), use it directly with `tdo`.
2. If it's a natural-language description, first run `tdo list --all` to find the matching todo, then use the ID.
3. If multiple todos match the description, use AskUserQuestion to let the user pick.
4. If no matches, tell the user and show available todos.

`tdo` supports unique ID prefixes — you can use `a3` instead of `a3f9` if it's unambiguous.

## Done

1. Match the query to a todo ID (see "Match")
2. Run:
   ```bash
   tdo done <id>
   ```
3. Confirm to the user that the todo was marked as done.

## Reopen

1. Match the query to a todo ID (see "Match")
2. Run:
   ```bash
   tdo reopen <id>
   ```
3. Confirm to the user that the todo was reopened.

## Edit

1. Match the query to a todo ID (see "Match")
2. Run:
   ```bash
   tdo edit <id> --body "new body content"
   ```
   The `--body` flag is required for non-interactive use. It replaces the todo's body text.
3. Confirm the update to the user.

## Delete

1. Match the query to a todo ID (see "Match")
2. Run:
   ```bash
   tdo delete <id> --force
   ```
   The `--force` flag is required for non-interactive use.
3. Confirm deletion to the user.

## Notes

- Todo titles are immutable after creation. To change a title, delete and recreate.
- The `--dir <PATH>` flag can override the `.todo/` directory location if needed.
- IDs are 4 hex characters. Unique prefixes work for all commands.
