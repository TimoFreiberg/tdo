#!/bin/sh
set -e

# Resolve the directory this script lives in
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
    Darwin)
        case "${ARCH}" in
            arm64|aarch64) BIN="tdo-macos-arm64" ;;
            *) echo "Unsupported macOS architecture: ${ARCH}" >&2; exit 1 ;;
        esac
        ;;
    Linux)
        case "${ARCH}" in
            x86_64|amd64) BIN="tdo-linux-x86_64" ;;
            *) echo "Unsupported Linux architecture: ${ARCH}" >&2; exit 1 ;;
        esac
        ;;
    *)
        echo "Unsupported OS: ${OS}" >&2
        exit 1
        ;;
esac

BIN_PATH="${SCRIPT_DIR}/${BIN}"

# Decompress on first run if only the .zst file exists
if [ ! -x "${BIN_PATH}" ] && [ -f "${BIN_PATH}.zst" ]; then
    zstd -d -o "${BIN_PATH}" "${BIN_PATH}.zst"
    chmod +x "${BIN_PATH}"
fi

exec "${BIN_PATH}" "$@"
