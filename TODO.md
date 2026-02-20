# TODO

## TUI: scroll support (#4)

The TUI renders a flat `List` with no scrolling. If there are more open todos
than fit in the viewport (MAX_HEIGHT - 3 = 17 items), the cursor can move past
the visible area. Switch to ratatui's stateful `ListState` rendering which
handles scroll-to-cursor automatically.

## TUI: empty state message (#5)

When there are zero open todos, the TUI shows an empty bordered box. Add an
empty-state message like "No open todos. Press n to create one." inside the
list area for first-run discoverability.

## TUI: handle terminal resize events (#13)

Only `Event::Key` is handled in the event loop. Terminal resize events are
ignored, so resizing the terminal while the TUI is open corrupts the display
until the next keypress. Handle `Event::Resize` to trigger a redraw.

## Consider subcommand interface (#14)

The flag-based interface (`tdo --done <id>`) is unusual for CLI tools. Most
users expect subcommands (`tdo done <id>`). Consider offering both via clap
aliases, or switching entirely to subcommands.

## List output formatting (#16)

`--list` output is plain `<id>  <title>` with no alignment or color. Adding
colored IDs, column alignment, or other minimal formatting would improve
scannability for larger lists.

## Document stale slugs in README (#18)

Editing a title with `--edit --title` updates the YAML frontmatter but the slug
in the filename remains from the original title. This is documented in
ARCHITECTURE.md but should also be mentioned in README.md so users browsing
`.todo/` with `ls` aren't confused.
