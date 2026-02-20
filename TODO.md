# TODO

## Decide error handling for malformed todo files

`load_all_todos` silently skips `.md` files that fail to parse, printing a
warning to stderr. This means `--list --all` can produce incomplete output with
no indication of data loss beyond the startup warning. Consider returning an
error instead, or adding a `--strict` flag, so corrupted frontmatter is not
silently dropped from the working set.

## Document stale slugs in README (#18)

Editing a title with `--edit --title` updates the YAML frontmatter but the slug
in the filename remains from the original title. This is documented in
ARCHITECTURE.md but should also be mentioned in README.md so users browsing
`.todo/` with `ls` aren't confused.
