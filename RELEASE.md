# Release & Distribution

## Overview

tdo uses a two-tier distribution model:

- **Slow-moving pieces** (wrapper script, skill file) are installed once from this repo into dotfiles via `just install` and propagate to other machines through dotfile syncing.
- **The binary** auto-updates itself via the wrapper script, pulling from GitHub Releases.

## Releasing a new version

1. Bump the version in `Cargo.toml`.
2. Run `just release <version>` (e.g. `just release 0.2.0`).
   This creates an annotated git tag `v0.2.0` and pushes it.
3. The GitHub Actions workflow (`.github/workflows/release.yml`) triggers on the tag push and:
   - Builds release binaries for macOS ARM64 and Linux x86_64 (musl, static).
   - Creates a GitHub Release with both binaries attached.

Users running the wrapper will pick up the new binary within a day automatically.

## Installing into dotfiles

From a checkout of this repo:

```sh
just install
```

This copies:
- `tdo-wrapper.sh` → `~/dotfiles/bin/tdo`
- `SKILL.md` → `~/.claude/skills/tdo/SKILL.md`

On machines where this repo isn't cloned, the wrapper and skill file arrive through normal dotfile syncing.

## How the wrapper works

`~/dotfiles/bin/tdo` is a shell script that manages the real binary at `~/.local/bin/tdo`:

- **First run**: downloads the latest release binary, then executes it.
- **Subsequent runs**: executes the cached binary immediately. If the last update check was more than 24 hours ago, checks for a newer release in the background (non-blocking).
- **Forced update**: `tdo --update` runs an immediate foreground update check, regardless of the stamp age. Any remaining arguments are forwarded to the binary after updating.
- **Update check**: compares `tdo --version` output against the latest GitHub Release tag. Downloads only if they differ.

## When to re-run `just install`

Only when the wrapper script or skill file changes (i.e. rarely). The binary stays current on its own.
