#!/usr/bin/env bash
# Sourced from the checkout's root. Sets `engine` and `image`.

engine=${CONTAINER_ENGINE:-$(command -v podman || command -v docker || true)}
[ -n "$engine" ] || {
  echo "$(basename "$0" .sh): needs podman or docker" >&2
  exit 2
}

image=garuda-toolbox-it
"$engine" build --tag "$image" --file ci/arch/Containerfile ci/arch
