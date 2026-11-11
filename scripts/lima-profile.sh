#!/usr/bin/env bash
#   scripts/lima-profile.sh [--keep] [--no-build] [--suite LIST]
# Profiles the window on the Garuda Lima VM, against a real system's
# package lists, kernels and DKMS modules. The Arch container builds it with
# the selftest and profiling features, the VM runs each self-test suite
# under a headless weston, and tracy-capture on this machine records it
# through a forwarded port.
#   --keep  keep the VM afterwards (limactl shell garuda-toolbox-profile)
#   --no-build  take the binary of the last build in target/lima-profile
#   --suite LIST  comma separated self-test suites, default packages-arch,kernels
# The VM renders on the CPU (lavapipe), so only the CPU zones of a capture
# count. Captures land in .claude/tracy/vm-<suite>.tracy.
# shellcheck source-path=SCRIPTDIR
set -euo pipefail
cd "$(dirname "$0")/.."

keep=false
build=true
suites=packages-arch,kernels
while [ $# -gt 0 ]; do
  case $1 in
  --keep) keep=true ;;
  --no-build) build=false ;;
  --suite)
    suites=${2:-}
    shift
    ;;
  *)
    echo "lima-profile.sh: unknown option '$1'" >&2
    exit 2
    ;;
  esac
  shift
done

IFS=, read -ra suite_list <<<"$suites"
for tool in limactl jq nix ssh; do
  command -v "$tool" >/dev/null || {
    echo "lima-profile.sh: $tool is missing" >&2
    exit 2
  }
done

out=target/lima-profile
binary=$out/garuda-toolbox
captures=.claude/tracy
mkdir -p "$out" "$captures"

if $build; then
  # shellcheck source=scripts/arch-image.sh
  source scripts/arch-image.sh
  echo "Building in the Arch container"

  # Thin LTO: the capture compares one build with the next, and fat LTO
  # takes far longer for the same code.
  "$engine" run --rm \
    --volume "$PWD:/src:ro" \
    --volume "$PWD/$out:/out" \
    --volume garuda-toolbox-profile-target:/build/profile-target \
    --volume garuda-toolbox-pkg-cargo:/build/pkg-cargo \
    --env CARGO_TARGET_DIR=/build/profile-target \
    --env CARGO_HOME=/build/pkg-cargo \
    "$image" \
    bash -c 'cargo build --release --locked --config "profile.release.lto=\"thin\"" \
      -p toolbox-gui --features toolbox-gui/selftest,toolbox-gui/profiling \
      && install -m 755 /build/profile-target/release/garuda-toolbox /out/'
fi

[ -x "$binary" ] || {
  echo "lima-profile.sh: no $binary, run without --no-build" >&2
  exit 2
}

image=${GARUDA_LIMA_IMAGE:-target/lima/garuda-lima.qcow2}
if [ -z "${GARUDA_LIMA_IMAGE:-}" ]; then
  # shellcheck source=scripts/lima-image.sh
  source scripts/lima-image.sh
  lima_image "$image" || exit 2
fi

case $image in
http://* | https://*) location=$image ;;
*) location=$(readlink -f "$image") ;;
esac
images=$(jq -nc --arg l "$location" --arg d "${GARUDA_LIMA_IMAGE_DIGEST:-}" \
  '[{location: $l, arch: "x86_64"} + (if $d == "" then {} else {digest: $d} end)]')

name=garuda-toolbox-profile
lima_dir=${LIMA_HOME:-$HOME/.lima}/$name
tunnel=

vm() {
  limactl shell --workdir / "$name" -- "$@"
}

# shellcheck disable=SC2329 # run by the trap
cleanup() {
  [ -z "$tunnel" ] || kill "$tunnel" 2>/dev/null || true
  $keep && return
  limactl delete --force "$name" >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "Starting $name"
limactl delete --force "$name" >/dev/null 2>&1 || true
limactl start --name="$name" --tty=false --set ".images = $images" tests/lima/garuda.yaml
vm systemctl is-system-running --wait >/dev/null 2>&1 || true

echo "Installing weston and the CPU renderer"
vm sudo pacman -Syu --noconfirm --needed weston vulkan-swrast mesa >/dev/null

echo "Copying the build, the catalog and the self-test"
vm mkdir -p /tmp/toolbox/scripts /tmp/toolbox/tests/selftest /tmp/toolbox/data
limactl copy "$binary" "$name:/tmp/toolbox/garuda-toolbox"
limactl copy scripts/selftest.sh "$name:/tmp/toolbox/scripts/"
limactl copy -r tests/selftest/configs "$name:/tmp/toolbox/tests/selftest/"
limactl copy -r data/catalog "$name:/tmp/toolbox/data/"

status=0
for suite in "${suite_list[@]}"; do
  capture=$captures/vm-$suite.tracy
  echo "Suite $suite"

  # The build's own source path does not exist in the VM, so the catalog
  # comes from the copy.
  vm bash -c "cd /tmp/toolbox && nohup env SELFTEST_BINARY=/tmp/toolbox/garuda-toolbox \
    GARUDA_TOOLBOX_CATALOG_DIR=/tmp/toolbox/data/catalog SELFTEST_ROOT=/tmp/toolbox/runs \
    bash scripts/selftest.sh $(printf '%q' "$suite") >/tmp/toolbox/$suite.log 2>&1 &"

  # The Tracy client listens on every address, IPv6 as well, so only the
  # port is matched.
  listening=false
  for _ in $(seq 120); do
    if vm ss -Hltn 2>/dev/null | grep -q ':8086 '; then
      listening=true
      break
    fi
    sleep 1
  done

  $listening || {
    echo "lima-profile.sh: $suite never started profiling:" >&2
    vm tail -n 20 "/tmp/toolbox/$suite.log" >&2 || true
    vm tail -n 20 "/tmp/toolbox/runs/$suite/out.log" >&2 || true
    status=1
    continue
  }

  ssh -F "$lima_dir/ssh.config" -N -L 8086:127.0.0.1:8086 "lima-$name" &
  tunnel=$!
  sleep 1

  # Ends when the window closes at the suite's end.
  nix shell nixpkgs#tracy -c tracy-capture -o "$capture" -f -a 127.0.0.1 >/dev/null || status=1
  kill "$tunnel" 2>/dev/null || true
  tunnel=

  verdict=$(vm cat "/tmp/toolbox/runs/$suite/shots/result" 2>/dev/null || echo "no verdict")
  echo "  $verdict, capture: $capture"
  [ "$verdict" = PASS ] || status=1
done

exit "$status"
