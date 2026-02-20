macos_target := "aarch64-apple-darwin"
linux_target := "x86_64-unknown-linux-gnu"
dist := "dist"

# Build all targets
build-all: build-macos build-linux

# Build for macOS ARM
build-macos:
    cargo build --release --target {{macos_target}}
    mkdir -p {{dist}}
    cp target/{{macos_target}}/release/tdo {{dist}}/tdo-macos-arm64

# Build for Linux x86_64 (requires cross: cargo install cross)
build-linux:
    cross build --release --target {{linux_target}}
    mkdir -p {{dist}}
    cp target/{{linux_target}}/release/tdo {{dist}}/tdo-linux-x86_64

# Copy wrapper script to dist/
dist: build-all
    cp tdo.sh {{dist}}/tdo
    chmod +x {{dist}}/tdo

# Run tests
test:
    cargo test

# Clean build artifacts and dist
clean:
    cargo clean
    rm -rf {{dist}}
