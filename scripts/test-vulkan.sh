#!/usr/bin/env bash
# nextest setup script: tests render on lavapipe wherever the dev shell or
# the Nix check names it, so snapshots match with or without a GPU.
set -euo pipefail

if [ -n "${TOOLBOX_TEST_VULKAN:-}" ]; then
  echo "VK_DRIVER_FILES=$TOOLBOX_TEST_VULKAN" >>"$NEXTEST_ENV"
fi

# lavapipe starts a thread per core in every test process, and nextest runs
# a process per core. On 32 cores tests waited minutes for 1024 threads.
if [ -z "${LP_NUM_THREADS:-}" ]; then
  echo "LP_NUM_THREADS=1" >>"$NEXTEST_ENV"
fi
