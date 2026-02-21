# Build release binary for current platform
build:
    cargo build --release

# Run tests
test:
    cargo test

# Tag and push a release (triggers CI build + GitHub release)
release version:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! echo "{{version}}" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?(\+[a-zA-Z0-9.]+)?$'; then
        echo "error: '{{version}}' is not a valid semver version" >&2
        exit 1
    fi
    sed -i '' 's/^version = ".*"/version = "{{version}}"/' Cargo.toml
    cargo check --quiet
    jj commit Cargo.toml Cargo.lock -m "chore: bump version to {{version}}"
    jj tag set "v{{version}}" -r @-
    git push origin tag "v{{version}}"

# Install wrapper script and skill file into dotfiles
install:
    mkdir -p ~/dotfiles/bin
    cp tdo-wrapper.sh ~/dotfiles/bin/tdo
    chmod +x ~/dotfiles/bin/tdo
    mkdir -p ~/.claude/skills/tdo
    cp SKILL.md ~/.claude/skills/tdo/SKILL.md
    @echo "Installed tdo wrapper to ~/dotfiles/bin/tdo"
    @echo "Installed skill to ~/.claude/skills/tdo/SKILL.md"

# Clean build artifacts
clean:
    cargo clean
