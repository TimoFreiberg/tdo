#!/bin/sh
set -e

REPO="TimoFreiberg/tdo"
INSTALL_DIR="${TDO_INSTALL_DIR:-$HOME/dotfiles/bin}"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
    Darwin)
        case "${ARCH}" in
            arm64|aarch64) ASSET="tdo-macos-arm64" ;;
            *) echo "Unsupported macOS architecture: ${ARCH}" >&2; exit 1 ;;
        esac
        ;;
    Linux)
        case "${ARCH}" in
            x86_64|amd64) ASSET="tdo-linux-x86_64" ;;
            *) echo "Unsupported Linux architecture: ${ARCH}" >&2; exit 1 ;;
        esac
        ;;
    *)
        echo "Unsupported OS: ${OS}" >&2
        exit 1
        ;;
esac

mkdir -p "${INSTALL_DIR}"

if command -v gh >/dev/null 2>&1; then
    echo "Downloading ${ASSET} from latest release..."
    gh release download --repo "${REPO}" --pattern "${ASSET}" --output "${INSTALL_DIR}/tdo" --clobber
else
    echo "Downloading ${ASSET} from latest release..."
    curl -fsSL -o "${INSTALL_DIR}/tdo" "https://github.com/${REPO}/releases/latest/download/${ASSET}"
fi

chmod +x "${INSTALL_DIR}/tdo"
echo "Installed tdo to ${INSTALL_DIR}/tdo"
