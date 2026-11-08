#!/usr/bin/env bash
#   scripts/lint.sh [files or folders]   (default: app crates)
set -euo pipefail
cd "$(dirname "$0")/.."
exec cargo run --quiet -p toolbox-lint -- "$@"
