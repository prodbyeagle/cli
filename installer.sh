#!/usr/bin/env bash
# eagle installer for macOS
# Usage: curl -fsSL https://raw.githubusercontent.com/prodbyeagle/cli/main/installer.sh | bash

set -euo pipefail

REPO="prodbyeagle/cli"
INSTALL_DIR="/usr/local/bin"
BINARY="eagle"
RELEASE_URL="https://github.com/${REPO}/releases/latest/download/${BINARY}"

# ── helpers ────────────────────────────────────────────────────────────────────

info()    { printf '\033[34m[info]\033[0m  %s\n' "$*"; }
success() { printf '\033[32m[ok]\033[0m    %s\n' "$*"; }
warn()    { printf '\033[33m[warn]\033[0m  %s\n' "$*"; }
die()     { printf '\033[31m[error]\033[0m %s\n' "$*" >&2; exit 1; }

# ── checks ─────────────────────────────────────────────────────────────────────

[[ "$(uname)" == "Darwin" ]] || die "This installer is for macOS only."

command -v curl >/dev/null 2>&1 || die "curl is required but not installed."

# ── dev mode: build from source ────────────────────────────────────────────────

if [[ "${1:-}" == "--dev" ]]; then
  info "Dev mode: building from source..."
  command -v cargo >/dev/null 2>&1 || die "cargo is required for --dev mode."
  cargo build --release
  src="$(pwd)/target/release/${BINARY}"
  [[ -f "$src" ]] || die "Build artifact not found: $src"
  install -m 755 "$src" "${INSTALL_DIR}/${BINARY}"
  success "Dev build installed to ${INSTALL_DIR}/${BINARY}"
  exit 0
fi

# ── download release binary ────────────────────────────────────────────────────

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

info "Downloading ${BINARY} from GitHub releases..."
curl -fsSL --output "$tmp" "$RELEASE_URL" \
  || die "Download failed. Check your network or visit https://github.com/${REPO}/releases"

chmod +x "$tmp"

# ── install ────────────────────────────────────────────────────────────────────

if [[ -w "$INSTALL_DIR" ]]; then
  mv "$tmp" "${INSTALL_DIR}/${BINARY}"
else
  info "Installing to ${INSTALL_DIR} (sudo required)..."
  sudo mv "$tmp" "${INSTALL_DIR}/${BINARY}"
fi

success "${BINARY} installed to ${INSTALL_DIR}/${BINARY}"

# ── shell integration hint ─────────────────────────────────────────────────────

if ! command -v eagle >/dev/null 2>&1; then
  warn "${INSTALL_DIR} is not in your PATH. Add the following to ~/.zshrc:"
  warn "  export PATH=\"${INSTALL_DIR}:\$PATH\""
fi

info "Run 'eagle init' to set up the 'g' goto shortcut in your shell."
