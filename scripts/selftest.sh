#!/usr/bin/env bash
#   scripts/selftest.sh [suite]
set -euo pipefail

repo=$(cd "$(dirname "$0")/.." && pwd)
suite=${1:-all}
case $suite in
"" | . | .. | */*)
  echo "bad suite name '$suite': one name, not a path" >&2
  exit 2
  ;;
esac

command -v weston >/dev/null || {
  echo "weston is missing: run inside the dev shell" >&2
  exit 2
}

cargo build --quiet --manifest-path "$repo/Cargo.toml" \
  -p toolbox-helper -p toolbox-gui --features toolbox-gui/selftest

root=${SELFTEST_ROOT:-${TMPDIR:-/tmp}/garuda-toolbox-selftest}
dir="$root/$suite"
rm -rf "$dir"
mkdir -p "$dir/shots" "$dir/config" "$dir/cache" "$dir/state" "$dir/data"
mkdir -m 700 "$dir/runtime"

export XDG_CONFIG_HOME="$dir/config" XDG_CACHE_HOME="$dir/cache" XDG_STATE_HOME="$dir/state"

# eframe keeps the window state there, which must not touch the real one.
export XDG_DATA_HOME="$dir/data"
export XDG_RUNTIME_DIR="$dir/runtime" XDG_CURRENT_DESKTOP=
export GARUDA_TOOLBOX_HELPER=dry-run
export GARUDA_TOOLBOX_SELFTEST_DIR="$dir/shots" GARUDA_TOOLBOX_SELFTEST_SUITE="$suite"

# Made here, not in git, because many users' ignore files leave these out.
configs="$dir/configs"
cp -r "$repo/tests/selftest/configs" "$configs"
chmod -R u+w "$configs"
mkdir -p "$configs/root/usr/lib" "$configs/home"
printf '%s\n' '# ~/.bashrc' "alias ll='ls -l'" >"$configs/home/.bashrc"
printf 'NAME="Garuda Linux"\n' >"$configs/root/usr/lib/os-release"
ln -s ../usr/lib/os-release "$configs/root/etc/os-release"
printf 'root ALL=(ALL:ALL) ALL\n\0' >"$configs/root/etc/sudoers"
export GARUDA_TOOLBOX_SELFTEST_CONFIGS="$configs"

mkdir -p "$XDG_CONFIG_HOME/garuda-toolbox"
printf '[packages]\nshow_aur = true\n' >"$XDG_CONFIG_HOME/garuda-toolbox/settings.toml"

# Larger than the window's default size (app/gui/src/main.rs).
screen_width=1632
screen_height=1104
# The runners render on the CPU, where a full run takes most of an hour.
run_timeout=3600

weston --backend=headless --width="$screen_width" --height="$screen_height" \
  --socket=selftest --idle-time=0 >"$dir/weston.log" 2>&1 &
weston_pid=$!
trap 'kill "$weston_pid" 2>/dev/null || true' EXIT

for _ in $(seq 100); do
  [ -S "$XDG_RUNTIME_DIR/selftest" ] && break
  sleep 0.1
done
[ -S "$XDG_RUNTIME_DIR/selftest" ] || {
  echo "weston did not start, see $dir/weston.log" >&2
  exit 2
}

# Boot Repair only shows on the live ISO, its suite runs it in test mode.
flags=()
case $suite in
bootrepair | all) flags+=(--boot-repair) ;;
esac

started=$(date +%s)
status=0
vulkan=()

[ -z "${TOOLBOX_TEST_VULKAN:-}" ] || vulkan=("VK_DRIVER_FILES=$TOOLBOX_TEST_VULKAN")

env -u DISPLAY -u DBUS_SESSION_BUS_ADDRESS WAYLAND_DISPLAY=selftest "${vulkan[@]}" \
  timeout "$run_timeout" "${CARGO_TARGET_DIR:-$repo/target}/debug/garuda-toolbox" "${flags[@]}" >"$dir/out.log" 2>&1 || status=$?

verdict=$(cat "$dir/shots/result" 2>/dev/null || echo "FAIL: no verdict (exit $status), see $dir/out.log")
# Read by scripts/selftest-junit.sh.
code=$status
[ "$verdict" = PASS ] || [ "$code" != 0 ] || code=1
echo "$code $(($(date +%s) - started))" >"$dir/result"
echo "SELFTEST $verdict"
echo "shots: $dir/shots"
[ "$verdict" = PASS ]
