# Build release binary for current platform
build:
    cargo build --release

# Run tests
test:
    cargo test

# Tag and push a release (triggers CI build + GitHub release)
release version:
    git tag -a "v{{version}}" -m "v{{version}}"
    git push origin "v{{version}}"

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
