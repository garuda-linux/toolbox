#!/usr/bin/env bash
#   scripts/arch-package.sh [makepkg arguments]
# The image knows only the test repository, so makepkg runs with --nodeps.
set -euo pipefail
cd "$(dirname "$0")/.."

# shellcheck source=scripts/arch-image.sh
source scripts/arch-image.sh

out=target/arch-package
mkdir -p "$out"

# Own volumes: the integration tests fill theirs as root, makepkg builds as another user.
exec "$engine" run --rm \
  --volume "$PWD:/src:ro" \
  --volume "$PWD/$out:/out" \
  --volume garuda-toolbox-pkg-target:/build/pkg-target \
  --volume garuda-toolbox-pkg-cargo:/build/pkg-cargo \
  --env CARGO_TARGET_DIR=/build/pkg-target \
  --env CARGO_HOME=/build/pkg-cargo \
  "$image" \
  /src/ci/arch/package.sh /src /out --nodeps "$@"
