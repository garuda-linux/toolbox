#!/usr/bin/env bash
# Cloudflare Pages has no mdbook. Keep the pinned version in step with nixpkgs.
set -euo pipefail
cd "$(dirname "$0")/.."

version=0.5.4
# SHA-256 of the musl archive of this version.
sha256=5222beabd3e37dc5be0d18ff99b79058469354db5c220153a1b92db5ba12be89

if ! command -v mdbook >/dev/null; then
  bin=$(mktemp -d)
  trap 'rm -rf "$bin"' EXIT
  curl -fsSL --proto '=https' --tlsv1.2 -o "$bin/mdbook.tar.gz" \
    "https://github.com/rust-lang/mdBook/releases/download/v$version/mdbook-v$version-x86_64-unknown-linux-musl.tar.gz"
  echo "$sha256  $bin/mdbook.tar.gz" | sha256sum -c --quiet
  tar -xzf "$bin/mdbook.tar.gz" -C "$bin"
  PATH="$bin:$PATH"
fi

mdbook --version
mdbook build docs
