#!/usr/bin/env bash
set -euo pipefail

# ── apl 一键安装 ────────────────────────────────────────────────
#
#   curl -fsSL https://raw.githubusercontent.com/AruNi-01/apl-cli/main/install.sh | sh
#
# ────────────────────────────────────────────────────────────────

REPO="AruNi-01/apl-cli"   # ← 改成你的 GitHub 用户名/仓库名
INSTALL_DIR="${HOME}/.local/bin"
BIN="apl"

info()  { printf '\033[1;34m[info]\033[0m %s\n' "$*"; }
ok()    { printf '\033[1;32m[ ok ]\033[0m %s\n' "$*"; }
err()   { printf '\033[1;31m[err]\033[0m  %s\n' "$*" >&2; exit 1; }

detect_target() {
    local os arch
    os="$(uname -s)";  arch="$(uname -m)"
    case "$os"   in Linux) os="unknown-linux-gnu";; Darwin) os="apple-darwin";; *) err "Unsupported OS: $os";; esac
    case "$arch" in x86_64|amd64) arch="x86_64";; arm64|aarch64) arch="aarch64";; *) err "Unsupported arch: $arch";; esac
    echo "${arch}-${os}"
}

main() {
    local target
    target="$(detect_target)"
    info "Platform: ${target}"

    local url="https://github.com/${REPO}/releases/latest/download/apl-${target}.tar.gz"
    info "Downloading ${url} ..."

    local tmp
    tmp="$(mktemp -d)"
    if curl -fsSL "$url" | tar xz -C "$tmp" 2>/dev/null; then
        mkdir -p "$INSTALL_DIR"
        mv "$tmp/$BIN" "$INSTALL_DIR/$BIN"
        chmod +x "$INSTALL_DIR/$BIN"
        rm -rf "$tmp"
        ok "Installed ${BIN} → ${INSTALL_DIR}/${BIN}"
    else
        rm -rf "$tmp"
        err "Download failed. Check https://github.com/${REPO}/releases for available binaries."
    fi

    # PATH hint
    case ":$PATH:" in
        *":${INSTALL_DIR}:"*) ;;
        *) info "Add to your shell profile:  export PATH=\"${INSTALL_DIR}:\$PATH\"" ;;
    esac

    info "Run: ${BIN} --version"
}

main "$@"
