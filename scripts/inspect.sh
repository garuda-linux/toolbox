#!/usr/bin/env bash
#   scripts/inspect.sh [app arguments]
# Dev only: the window serves egui_inspection on 127.0.0.1:5719 without
# authentication, for egui_mcp to read and drive it.
set -euo pipefail
cd "$(dirname "$0")/.."

command -v cargo >/dev/null || exec nix develop --command "$0" "$@"

EGUI_INSPECTION=1 exec cargo run --bin garuda-toolbox --features toolbox-gui/inspection -- "$@"
