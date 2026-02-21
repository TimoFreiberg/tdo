# tdo

A local todo manager written in Rust. Stores todos as markdown files with YAML frontmatter in `.todo/`.

## Documentation

- **README.md** — Usage guide, CLI examples, todo format, filename/ID scheme
- **ARCHITECTURE.md** — Design docs: storage format, CLI modes, TUI keybindings, dependency table
- **SKILL.md** — Claude Code skill spec for natural-language todo operations
- **RELEASE.md** — Release process, distribution model, wrapper script details

When making changes, update the relevant documentation files to reflect the new behavior. In particular:

- CLI changes (new flags, changed semantics) → update **README.md** and **ARCHITECTURE.md**
- New operations or changed dispatch logic → update **SKILL.md**
- Non-interactive interface changes (flags for automation) → update **SKILL.md**
- TUI keybinding or behavior changes → update **ARCHITECTURE.md**
- Dependency additions or removals → update **ARCHITECTURE.md**

## Project layout

```
src/
  main.rs        — entry point, TTY detection, dispatch
  cli.rs         — clap argument parsing, SubCommand enum
  todo.rs        — Todo struct, frontmatter parsing, Status enum
  storage.rs     — Store: file I/O, locking, directory management
  ops.rs         — high-level operations (create, edit, done, delete, list)
  util.rs        — ID generation, slugify, TTY helpers
  tui/           — ratatui-based terminal UI (mod.rs, ui.rs, events.rs)
tests/
  cli.rs         — integration tests
  helpers/mod.rs — test utilities
```

## Build and test

```
cargo build --release
cargo test
```

Or via Justfile: `just build`, `just test`.
