#!/usr/bin/env bash
# Sourced from the checkout's root after scripts/arch-image.sh.
# `arch_build <out dir> <name> <features>` builds garuda-toolbox with the
# features and its helper in the Arch container and puts both binaries into
# <out dir>. A release app runs only the installed helper, so a caller that
# applies a plan puts the helper at /usr/lib/garuda-toolbox/. The cargo
# target lives in the volume garuda-toolbox-<name>-target.

# shellcheck disable=SC2154 # engine and image come from scripts/arch-image.sh
arch_build() {
  local out=$1 name=$2 features=$3

  # Thin LTO: fat LTO takes far longer for the same code. FEATURES expands
  # in the container.
  # shellcheck disable=SC2016
  "$engine" run --rm \
    --volume "$PWD:/src:ro" \
    --volume "$PWD/$out:/out" \
    --volume "garuda-toolbox-$name-target:/build/target" \
    --volume garuda-toolbox-pkg-cargo:/build/pkg-cargo \
    --env CARGO_TARGET_DIR=/build/target \
    --env CARGO_HOME=/build/pkg-cargo \
    --env FEATURES="$features" \
    "$image" \
    bash -c 'cargo build --release --locked --config "profile.release.lto=\"thin\"" \
      -p toolbox-gui -p toolbox-helper --features "$FEATURES" \
      && install -m 755 /build/target/release/garuda-toolbox \
        /build/target/release/garuda-toolbox-helper /out/'
}
