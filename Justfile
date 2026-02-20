macos_target := "aarch64-apple-darwin"
linux_target := "x86_64-unknown-linux-musl"
dist := "dist"

# Build all targets
build-all: build-macos build-linux

# Build for macOS ARM
build-macos:
    cargo build --release --target {{macos_target}}
    mkdir -p {{dist}}
    zstd -19 -f -o {{dist}}/tdo-macos-arm64.zst target/{{macos_target}}/release/tdo

# Build for Linux x86_64 (uses rust-lld, no cross/Docker needed)
build-linux:
    cargo build --release --target {{linux_target}}
    mkdir -p {{dist}}
    zstd -19 -f -o {{dist}}/tdo-linux-x86_64.zst target/{{linux_target}}/release/tdo

# Copy wrapper script to dist/
dist: build-all
    cp tdo.sh {{dist}}/tdo
    chmod +x {{dist}}/tdo

# Install compressed binaries and wrapper to ~/dotfiles/bin
install: dist
    mkdir -p ~/dotfiles/bin
    cp {{dist}}/tdo-macos-arm64.zst {{dist}}/tdo-linux-x86_64.zst {{dist}}/tdo ~/dotfiles/bin/

# Run tests
test:
    cargo test

# Clean build artifacts and dist
clean:
    cargo clean
    rm -rf {{dist}}
