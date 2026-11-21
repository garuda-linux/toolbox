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
merge_fixtures=(
  crates/system/src/packages/pacman_conf/fixtures/pacman.conf
  crates/plan/src/testing/locales/fixtures/locale.gen
)
captures=.claude/tracy
mkdir -p "$out" "$captures"

if $build; then
  # shellcheck source=scripts/arch-image.sh
  source scripts/arch-image.sh
  echo "Building in the Arch container"

  # shellcheck source=scripts/arch-build.sh
  source scripts/arch-build.sh
  arch_build "$out" profile toolbox-gui/selftest,toolbox-gui/profiling
fi

[ -x "$binary" ] || {
  echo "lima-profile.sh: no $binary, run without --no-build" >&2
  exit 2
}

# shellcheck source=scripts/lima-image.sh
source scripts/lima-image.sh
images=$(lima_images lima-profile.sh) || exit 2

name=garuda-toolbox-profile
lima_dir=${LIMA_HOME:-$HOME/.lima}/$name
tunnel=
run=

vm() {
  limactl shell --workdir / "$name" -- "$@"
}

# shellcheck disable=SC2329 # run by the trap
cleanup() {
  [ -z "$tunnel" ] || kill "$tunnel" 2>/dev/null || true
  [ -z "$run" ] || kill "$run" 2>/dev/null || true
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

for fixture in "${merge_fixtures[@]}"; do
  vm mkdir -p "/tmp/toolbox/$(dirname "$fixture")"
  limactl copy "$fixture" "$name:/tmp/toolbox/$fixture"
done

limactl copy -r data/catalog "$name:/tmp/toolbox/data/"

status=0
for suite in "${suite_list[@]}"; do
  capture=$captures/vm-$suite.tracy
  echo "Suite $suite"

  # The build's source path is not in the VM, so the catalog comes from the
  # copy. Run from here: a job backgrounded inside the VM never wrote its log.
  # Tracy refuses to start on the VM's CPU, which reports no invariant TSC.
  log=$captures/vm-$suite.log
  vm env SELFTEST_BINARY=/tmp/toolbox/garuda-toolbox \
    GARUDA_TOOLBOX_CATALOG_DIR=/tmp/toolbox/data/catalog SELFTEST_ROOT=/tmp/toolbox/runs \
    TRACY_NO_INVARIANT_CHECK=1 bash /tmp/toolbox/scripts/selftest.sh "$suite" >"$log" 2>&1 &
  run=$!

  # The Tracy client listens on every address, IPv6 as well, so only the
  # port is matched.
  listening=false
  for _ in $(seq 120); do
    if vm ss -Hltn 2>/dev/null | grep -q ':8086 '; then
      listening=true
      break
    fi
    kill -0 "$run" 2>/dev/null || break
    sleep 1
  done

  $listening || {
    echo "lima-profile.sh: $suite never started profiling, $log says:" >&2
    kill "$run" 2>/dev/null || true
    wait "$run" 2>/dev/null || true
    run=
    tail -n 20 "$log" >&2
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
  wait "$run" 2>/dev/null || true
  run=

  verdict=$(vm cat "/tmp/toolbox/runs/$suite/shots/result" 2>/dev/null || echo "no verdict")
  echo "  $verdict, capture: $capture"
  [ "$verdict" = PASS ] || status=1
done

exit "$status"
