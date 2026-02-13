#!/bin/sh
set -eu

REPO="${REPO:-ayuukumakuma/cf-page-to-md}"
BINARY_NAME="page2md"
TARGET="aarch64-apple-darwin"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

error() {
  printf 'Error: %s\n' "$*" >&2
  exit 1
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || error "'$1' が必要です。"
}

require_cmd curl
require_cmd tar
require_cmd uname
require_cmd mktemp
require_cmd install

OS="$(uname -s)"
ARCH="$(uname -m)"

[ "$OS" = "Darwin" ] || error "このインストーラは macOS 専用です。現在: ${OS}"
case "$ARCH" in
  arm64 | aarch64)
    ;;
  *)
    error "このインストーラは Apple Silicon (arm64) 専用です。現在: ${ARCH}"
    ;;
esac

if [ -n "${VERSION:-}" ]; then
  RELEASE_TAG="$VERSION"
else
  LATEST_RELEASE_JSON="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest")"
  RELEASE_TAG="$(
    printf '%s' "$LATEST_RELEASE_JSON" \
      | tr -d '\n' \
      | sed -nE 's/.*"tag_name"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/p'
  )"
  [ -n "$RELEASE_TAG" ] || error "最新リリースの tag_name を取得できませんでした。"
fi

ASSET_NAME="${BINARY_NAME}-${RELEASE_TAG}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${RELEASE_TAG}/${ASSET_NAME}"

TMP_DIR="$(mktemp -d)"
cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT INT HUP TERM

printf 'Downloading %s\n' "$DOWNLOAD_URL"
curl -fL "$DOWNLOAD_URL" -o "${TMP_DIR}/${ASSET_NAME}"

mkdir -p "$INSTALL_DIR"
tar -xzf "${TMP_DIR}/${ASSET_NAME}" -C "$TMP_DIR"
[ -f "${TMP_DIR}/${BINARY_NAME}" ] || error "アーカイブ内に ${BINARY_NAME} が見つかりません。"

install -m 755 "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"

printf 'Installed: %s\n' "${INSTALL_DIR}/${BINARY_NAME}"
case ":$PATH:" in
  *":${INSTALL_DIR}:"*)
    ;;
  *)
    printf 'PATH に %s が含まれていません。以下をシェル設定に追加してください:\n' "$INSTALL_DIR"
    printf 'export PATH="%s:$PATH"\n' "$INSTALL_DIR"
    ;;
esac
