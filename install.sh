#!/usr/bin/env sh
set -e

REPO="vika2603/comment-checker-rs"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

detect_platform() {
  os=$(uname -s)
  arch=$(uname -m)

  case "$os" in
    Darwin) os="darwin" ;;
    Linux)  os="linux" ;;
    *)      echo "Unsupported OS: $os" >&2; exit 1 ;;
  esac

  case "$arch" in
    x86_64|amd64)  arch="x86_64" ;;
    arm64|aarch64) arch="arm64" ;;
    *)             echo "Unsupported architecture: $arch" >&2; exit 1 ;;
  esac

  echo "${os}-${arch}"
}

get_latest_version() {
  curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' \
    | sed 's/.*"tag_name": *"//;s/".*//'
}

main() {
  platform=$(detect_platform)
  version="${VERSION:-$(get_latest_version)}"

  if [ -z "$version" ]; then
    echo "Error: could not determine latest version" >&2
    exit 1
  fi

  url="https://github.com/${REPO}/releases/download/${version}/comment-checker-${platform}"
  echo "Downloading comment-checker ${version} for ${platform}..."

  tmpfile=$(mktemp)
  curl -fsSL "$url" -o "$tmpfile"
  chmod +x "$tmpfile"

  if [ -w "$INSTALL_DIR" ]; then
    mv "$tmpfile" "${INSTALL_DIR}/comment-checker"
  else
    echo "Installing to ${INSTALL_DIR} (requires sudo)..."
    sudo mv "$tmpfile" "${INSTALL_DIR}/comment-checker"
  fi

  echo "Installed comment-checker ${version} to ${INSTALL_DIR}/comment-checker"

  if [ -n "${HOOK_TARGET:-}" ]; then
    echo "Setting up hook for ${HOOK_TARGET}..."
    "${INSTALL_DIR}/comment-checker" init "$HOOK_TARGET"
  fi
}

usage() {
  cat <<EOF
Usage:
  curl -fsSL <url>/install.sh | sh
  curl -fsSL <url>/install.sh | sh -s -- --claude
  curl -fsSL <url>/install.sh | sh -s -- --codex

Options:
  --claude    Install and set up Claude Code hook
  --codex     Install and set up Codex hook
  --dir PATH  Install to PATH (default: /usr/local/bin)
EOF
  exit 0
}

parse_args() {
  while [ $# -gt 0 ]; do
    case "$1" in
      --claude) HOOK_TARGET="claude" ;;
      --codex)  HOOK_TARGET="codex" ;;
      --dir)    INSTALL_DIR="$2"; shift ;;
      --help|-h) usage ;;
      *) echo "Unknown option: $1" >&2; exit 1 ;;
    esac
    shift
  done
}

parse_args "$@"
main
