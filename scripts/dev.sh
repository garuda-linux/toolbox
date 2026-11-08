#!/usr/bin/env bash
#   scripts/dev.sh [app arguments]
set -euo pipefail
cd "$(dirname "$0")/.."

command -v watchexec >/dev/null || exec nix develop --command "$0" "$@"

exec watchexec --restart --debounce 300ms \
  --watch app --watch crates --watch data/catalog \
  --watch Cargo.toml --watch Cargo.lock \
  --exts rs,ftl,toml,lock \
  -- cargo run --bin garuda-toolbox -- "$@"
