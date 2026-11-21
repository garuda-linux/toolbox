#!/usr/bin/env bash
#   scripts/garuda-selftest.sh [--keep] [--no-build] [suite...]
# Runs the self-test on a real Garuda system: the newest garuda-distrobox
# image with systemd as init, so pages read pacman, systemd, locales and
# kernels instead of showing "could not be read". The Arch container builds
# the app with the selftest feature, the Garuda container runs each suite
# under a headless weston as a user of its own, since the app refuses root.
#   --keep      keep the container (podman exec -it garuda-toolbox-selftest bash)
#   --no-build  take the binary of the last build in target/garuda-selftest
#   suite...    default `all` and `packages-arch` as the arch-selftest CI job,
#               then `services-systemd`
# Shots and logs land in target/garuda-selftest/runs/<suite>/, the JUnit
# report in target/garuda-selftest/junit.xml.
# shellcheck source-path=SCRIPTDIR
set -euo pipefail
cd "$(dirname "$0")/.."

keep=false
build=true
suites=()
while [ $# -gt 0 ]; do
  case $1 in
  --keep) keep=true ;;
  --no-build) build=false ;;
  -*)
    echo "garuda-selftest.sh: unknown option '$1'" >&2
    exit 2
    ;;
  *) suites+=("$1") ;;
  esac
  shift
done
[ ${#suites[@]} -gt 0 ] || suites=(all packages-arch services-systemd)

out=target/garuda-selftest
binary=$out/garuda-toolbox
merge_fixtures=(
  crates/system/src/packages/pacman_conf/fixtures/pacman.conf
  crates/plan/src/testing/locales/fixtures/locale.gen
)
mkdir -p "$out"

# shellcheck source=scripts/arch-image.sh
source scripts/arch-image.sh

if $build; then
  echo "Building in the Arch container"
  # shellcheck source=scripts/arch-build.sh
  source scripts/arch-build.sh
  arch_build "$out" selftest toolbox-gui/selftest
fi

[ -x "$binary" ] && [ -x "$out/garuda-toolbox-helper" ] || {
  echo "garuda-selftest.sh: no build in $out, run without --no-build" >&2
  exit 2
}

garuda=registry.gitlab.com/garuda-linux/tools/image-builder/garuda-distrobox:latest
name="garuda-toolbox-selftest"

in_box() {
  "$engine" exec "$name" "$@"
}

# shellcheck disable=SC2329 # run by the trap
cleanup() {
  $keep && return
  "$engine" rm --force "$name" >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "Starting $name from the newest $garuda"
"$engine" rm --force "$name" >/dev/null 2>&1 || true
"$engine" pull --quiet "$garuda" >/dev/null
"$engine" run --detach --name "$name" --systemd=always "$garuda" /usr/lib/systemd/systemd >/dev/null

# systemctl answers only once systemd made the system bus, which takes a
# moment after the container starts. Units that need hardware fail in a
# container and leave the state "degraded", which is fine.
state=
for _ in $(seq 60); do
  state=$(in_box systemctl is-system-running --wait 2>&1 || true)
  case $state in
  running | degraded) break ;;
  esac
  sleep 1
done
echo "systemd: $state"
case $state in
running | degraded) ;;
*)
  echo "garuda-selftest.sh: systemd did not come up in $name. Its init and the processes:" >&2
  "$engine" logs "$name" 2>&1 | tail -20 >&2
  in_box ps -eo pid,comm >&2 || true
  exit 1
  ;;
esac

# dbus-broker dies in this container ("Package not installed" from its
# launcher), so the classic dbus-daemon serves the system bus here. The app
# talks to either the same way. Garuda's dbus-units needs the broker's
# units by name, so they go without a dependency check.
echo "Installing weston, the CPU renderer and dbus-daemon"
{
  in_box pacman -Syu --noconfirm --needed weston vulkan-swrast mesa &&
    in_box pacman -Rdd --noconfirm dbus-broker-units &&
    in_box pacman -S --noconfirm dbus-daemon-units
} >"$out/pacman.log" 2>&1 ||
  { echo "garuda-selftest.sh: pacman failed, see $out/pacman.log" >&2 && exit 1; }

# root's systemctl talks to systemd's private socket, the app as a user
# needs the system bus.
in_box systemctl daemon-reload
in_box systemctl reset-failed >/dev/null 2>&1 || true
in_box systemctl restart dbus.socket dbus.service >/dev/null 2>&1 || true
if ! in_box runuser -u nobody -- busctl --system list >/dev/null 2>&1; then
  echo "garuda-selftest.sh: no system bus for users in $name. Failed units:" >&2
  in_box systemctl --failed --no-legend >&2 || true
  echo "The bus's log:" >&2
  in_box journalctl -b -u dbus.service -u dbus.socket --no-pager -n 30 >&2 || true
  exit 1
fi

echo "Copying the build, the catalog and the self-test"
in_box useradd --create-home selftest
in_box mkdir -p /tmp/toolbox/scripts /tmp/toolbox/tests/selftest /tmp/toolbox/data /tmp/toolbox/runs \
  /usr/lib/garuda-toolbox
"$engine" cp "$binary" "$name:/tmp/toolbox/garuda-toolbox"
"$engine" cp "$out/garuda-toolbox-helper" "$name:/usr/lib/garuda-toolbox/garuda-toolbox-helper"
"$engine" cp scripts/selftest.sh "$name:/tmp/toolbox/scripts/"
"$engine" cp tests/selftest/configs "$name:/tmp/toolbox/tests/selftest/"
"$engine" cp data/catalog "$name:/tmp/toolbox/data/"

for fixture in "${merge_fixtures[@]}"; do
  in_box mkdir -p "/tmp/toolbox/$(dirname "$fixture")"
  "$engine" cp "$fixture" "$name:/tmp/toolbox/$fixture"
done

in_box chown -R selftest: /tmp/toolbox

status=0
for suite in "${suites[@]}"; do
  echo "Suite $suite"

  # The build's source path is not in the container, so the catalog comes
  # from the copy.
  in_box runuser -u selftest -- env HOME=/home/selftest \
    SELFTEST_BINARY=/tmp/toolbox/garuda-toolbox \
    GARUDA_TOOLBOX_CATALOG_DIR=/tmp/toolbox/data/catalog SELFTEST_ROOT=/tmp/toolbox/runs \
    bash /tmp/toolbox/scripts/selftest.sh "$suite" || status=1
done

rm -rf "$out/runs"
"$engine" cp "$name:/tmp/toolbox/runs" "$out/runs"
scripts/selftest-junit.sh "$out/runs" "$out/junit.xml"
echo "Runs: $out/runs"
exit "$status"
