#!/bin/sh
set -e

REPO="TimoFreiberg/tdo"
INSTALL_DIR="${TDO_INSTALL_DIR:-$HOME/dotfiles/bin}"
WRAPPER_URL="https://raw.githubusercontent.com/${REPO}/main/tdo-wrapper.sh"
SKILL_URL="https://raw.githubusercontent.com/${REPO}/main/SKILL.md"

mkdir -p "${INSTALL_DIR}"
echo "Downloading tdo wrapper..."
curl -fsSL -o "${INSTALL_DIR}/tdo" "${WRAPPER_URL}"
chmod +x "${INSTALL_DIR}/tdo"
echo "Installed tdo wrapper to ${INSTALL_DIR}/tdo"

SKILL_DIR="${HOME}/.claude/skills/tdo"
mkdir -p "${SKILL_DIR}"
curl -fsSL -o "${SKILL_DIR}/SKILL.md" "${SKILL_URL}"
echo "Installed skill to ${SKILL_DIR}/SKILL.md"
