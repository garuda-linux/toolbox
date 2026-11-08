#!/usr/bin/env bash
#   scripts/arch-it.sh [cargo test arguments]
set -euo pipefail
cd "$(dirname "$0")/.."

# shellcheck source=scripts/arch-image.sh
source scripts/arch-image.sh

if [ $# -eq 0 ]; then
  set -- -p toolbox-helper --test arch
fi

# The tests share the container's system.
exec "$engine" run --rm \
  --volume "$PWD:/src:ro" \
  --volume garuda-toolbox-it-target:/build/target \
  --volume garuda-toolbox-it-cargo:/build/cargo \
  "$image" \
  cargo test --locked "$@" -- --test-threads=1
