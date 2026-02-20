# tdo — TODO

## TUI: scroll support
The TUI renders a flat list with no scrolling. With more than ~17 open items
the cursor can move past the visible area. Switch `draw_list` to use
ratatui's stateful `ListState` with `highlight_symbol` so the viewport
follows the cursor automatically.

## TUI: empty state message
When there are zero open todos the TUI shows an empty bordered box. Display
a hint like "No open todos. Press n to create one." inside the list area.

## TUI: handle terminal resize events
Only `Event::Key` is matched in the event loop. A terminal resize will not
trigger a redraw until the next keypress. Handle `Event::Resize` to redraw
immediately.

## CLI: consider subcommand interface
The current flag-based interface (`tdo --done <id>`) is unusual. Most CLI
tools use subcommands (`tdo done <id>`). Consider supporting both via clap
aliases, or migrating to subcommands outright.

## CLI: list output formatting
`--list` output has no color or alignment. Adding colored IDs and column
alignment would improve scannability for larger lists.

## Docs: stale slugs after title edits
`save` writes to the original filename, so the slug becomes stale after
`--edit --title`. This is intentional but should be documented in the README
alongside the file format section.
