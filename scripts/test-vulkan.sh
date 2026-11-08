#!/usr/bin/env bash
# nextest setup script: tests render on lavapipe wherever the dev shell or
# the Nix check names it, so snapshots match with or without a GPU.
set -euo pipefail

if [ -n "${TOOLBOX_TEST_VULKAN:-}" ]; then
  echo "VK_DRIVER_FILES=$TOOLBOX_TEST_VULKAN" >>"$NEXTEST_ENV"
fi
