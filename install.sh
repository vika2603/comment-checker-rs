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
}

main
