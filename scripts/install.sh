#!/usr/bin/env sh
# One-line install for unix-like systems:
#   curl -fsSL https://raw.githubusercontent.com/AkaraChen/para/main/scripts/install.sh | sh
set -eu

REPO="${PARA_REPO:-AkaraChen/para}"
BIN="${PARA_BIN:-para}"
PREFIX="${PARA_PREFIX:-}"
VERSION="${PARA_VERSION:-}"

say() { printf '%s\n' "$*"; }
err() { say "para-install: $*" >&2; exit 1; }

need() {
  command -v "$1" >/dev/null 2>&1 || err "missing required command: $1"
}

need uname
need mktemp

if command -v curl >/dev/null 2>&1; then
  fetch() { curl -fsSL "$1"; }
  fetch_file() { curl -fsSL "$1" -o "$2"; }
elif command -v wget >/dev/null 2>&1; then
  fetch() { wget -qO- "$1"; }
  fetch_file() { wget -qO "$2" "$1"; }
else
  err "need curl or wget"
fi

os=$(uname -s | tr '[:upper:]' '[:lower:]')
arch=$(uname -m)
case "$os" in
  linux|darwin|freebsd) ;;
  mingw*|msys*|cygwin*) err "use scripts/install.ps1 on Windows" ;;
  *) err "unsupported OS: $os" ;;
esac
case "$arch" in
  x86_64|amd64) arch=amd64 ;;
  aarch64|arm64) arch=arm64 ;;
  *) err "unsupported architecture: $arch" ;;
esac

if [ -z "$VERSION" ]; then
  VERSION=$(fetch "https://api.github.com/repos/${REPO}/releases/latest" | sed -n 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1)
  [ -n "$VERSION" ] || err "could not resolve the latest GitHub release for ${REPO}"
fi
VERSION="${VERSION#v}"

if [ -z "$PREFIX" ]; then
  if [ -w /usr/local/bin ] 2>/dev/null; then
    PREFIX=/usr/local/bin
  else
    PREFIX="${HOME}/.local/bin"
  fi
fi

asset="${BIN}_${VERSION}_${os}_${arch}.tar.gz"
base="https://github.com/${REPO}/releases/download/v${VERSION}"
tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT

say "installing ${BIN} v${VERSION} (${os}/${arch}) to ${PREFIX}"
fetch_file "${base}/${asset}" "${tmpdir}/${asset}"
fetch_file "${base}/checksums.txt" "${tmpdir}/checksums.txt"

if command -v sha256sum >/dev/null 2>&1; then
  (cd "$tmpdir" && grep " ${asset}\$" checksums.txt | sha256sum -c -)
elif command -v shasum >/dev/null 2>&1; then
  (cd "$tmpdir" && grep " ${asset}\$" checksums.txt | shasum -a 256 -c -)
else
  say "warning: no sha256 tool; skipping checksum verification"
fi

tar -xzf "${tmpdir}/${asset}" -C "$tmpdir"
src=$(find "$tmpdir" -type f -name "$BIN" | head -n 1)
[ -n "$src" ] || err "archive did not contain ${BIN}"

mkdir -p "$PREFIX"
install -m 0755 "$src" "${PREFIX}/${BIN}"

case ":$PATH:" in
  *":${PREFIX}:"*) ;;
  *) say "add ${PREFIX} to PATH to run ${BIN}" ;;
esac

"${PREFIX}/${BIN}" --help >/dev/null
say "installed ${PREFIX}/${BIN}"
