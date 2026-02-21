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

# Deploy skill file to global Claude skills
deploy-skill:
    mkdir -p ~/.claude/skills/tdo
    cp SKILL.md ~/.claude/skills/tdo/SKILL.md

# Clean build artifacts
clean:
    cargo clean
