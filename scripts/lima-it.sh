#!/usr/bin/env bash
#   scripts/lima-it.sh [--keep] [--no-build] [--suite LIST]
# Installs the package into a Garuda Lima VM and drives through the CLI what
# the Arch container cannot run: systemd, locales, DKMS and NetworkManager.
# Then it restarts the VM once.
#   --keep  keep the VM afterwards (limactl shell garuda-toolbox-it)
#   --no-build  take the package of this version from target/arch-package
#   --suite LIST  comma separated, of basic (the default), upgrade, faults,
#           accounts and sdboot. Each one after the first starts from a snapshot
#           taken after the setup.
# GARUDA_LIMA_IMAGE is a path or URL of the qcow2, GARUDA_LIMA_IMAGE_DIGEST
# its optional "sha256:<hex>" for a URL. Unset, target/lima/garuda-lima.qcow2
# is used, downloaded from image-builder's packages when it is missing.
# shellcheck source-path=SCRIPTDIR
set -euo pipefail
cd "$(dirname "$0")/.."

keep=false
build=true
suites=basic
while [ $# -gt 0 ]; do
  case $1 in
  --keep) keep=true ;;
  --no-build) build=false ;;
  --suite)
    suites=${2:-}
    shift
    ;;
  *)
    echo "lima-it.sh: unknown option '$1'" >&2
    exit 2
    ;;
  esac
  shift
done

IFS=, read -ra suite_list <<<"$suites"
[ "${#suite_list[@]}" -gt 0 ] || {
  echo "lima-it.sh: --suite needs a list" >&2
  exit 2
}
for suite in "${suite_list[@]}"; do
  case $suite in
  basic | upgrade | faults | accounts | sdboot) ;;
  *)
    echo "lima-it.sh: unknown suite '$suite'" >&2
    exit 2
    ;;
  esac
done

for tool in limactl jq; do
  command -v "$tool" >/dev/null || {
    echo "lima-it.sh: $tool is missing" >&2
    exit 2
  }
done

image=${GARUDA_LIMA_IMAGE:-target/lima/garuda-lima.qcow2}
if [ -z "${GARUDA_LIMA_IMAGE:-}" ]; then
  # shellcheck source=scripts/lima-image.sh
  source scripts/lima-image.sh
  lima_image "$image" || exit 2
fi

case $image in
http://* | https://*) location=$image ;;
*)
  [ -f "$image" ] || {
    echo "lima-it.sh: no image at $image, set GARUDA_LIMA_IMAGE" >&2
    exit 2
  }
  location=$(readlink -f "$image")
  ;;
esac
images=$(jq -nc --arg l "$location" --arg d "${GARUDA_LIMA_IMAGE_DIGEST:-}" \
  '[{location: $l, arch: "x86_64"} + (if $d == "" then {} else {digest: $d} end)]')

name=garuda-toolbox-it
if [ -n "${LIMA_IT_ROOT:-}" ]; then
  dir="$LIMA_IT_ROOT/$name"
  rm -rf "$dir"
  mkdir -p "$dir"
else
  # A fixed name in a shared /tmp lets another user plant the directory.
  dir=$(mktemp -d -t "$name.XXXXXX")
fi

vm() {
  limactl shell --workdir / "$name" -- "$@"
}

# shellcheck disable=SC2329 # used by scripts/lima-it/checks.sh
as_root() {
  vm sudo bash -c "$1"
}

# The CLI runs on a terminal as a person's would, which script(1) gives it.
# The limit turns a hang, such as a password question, into a failed check.
# shellcheck disable=SC2329 # used by scripts/lima-it/checks.sh
tb() {
  vm timeout 600 script -qefc "$(printf '%q ' env GARUDA_TOOLBOX_PAGER=cat NO_COLOR=1 TERM=dumb \
    garuda-toolbox "$@")" /dev/null | tr -d '\r'
}

# Without a terminal, for a secret on standard input: script(1) would echo
# it into the log.
# shellcheck disable=SC2329 # used by the suites
tp() {
  vm timeout 600 env GARUDA_TOOLBOX_PAGER=cat NO_COLOR=1 garuda-toolbox "$@"
}

# Lima's SSH master outlives a restore or a forced stop, and every new
# connection through it hangs until it times out.
ssh_socket=${LIMA_HOME:-$HOME/.lima}/$name/ssh.sock
# shellcheck disable=SC2329 # run through setup and the suites
wait_ssh() {
  rm -f "$ssh_socket"
  local _
  for _ in $(seq 120); do
    vm true 2>/dev/null && return 0
    sleep 5
  done

  echo "lima-it.sh: $name does not answer" >&2
  return 1
}

# A stopped QEMU lets go of its disk a moment later, and a new one started
# before that finds the disk locked.
# shellcheck disable=SC2329 # used by restore_base and stop_during
wait_qemu_gone() {
  local _
  for _ in $(seq 60); do
    pgrep -f "${LIMA_HOME:-$HOME/.lima}/$name/disk" >/dev/null || return 0
    sleep 1
  done
}

# A cold restore: memory loaded into a QEMU started after the snapshot loses SSH or name lookups.
# shellcheck disable=SC2329 # used by the suites
restore_base() {
  limactl stop "$name" || limactl stop --force "$name" || return 1
  wait_qemu_gone

  limactl snapshot apply "$name" --tag base &&
    limactl start --tty=false "$name" &&
    wait_ssh || return 1

  local _
  for _ in $(seq 12); do
    vm getent hosts archlinux.org && return 0
    sleep 5
  done

  echo "lima-it.sh: no name resolves in $name after the restore" >&2
  vm resolvectl status || true

  return 1
}

# stop_during <program> <command...>
# A power loss while <program> runs.
# shellcheck disable=SC2329 # used by the suites
stop_during() {
  local program=$1 pid seen=false _
  shift
  "$@" &
  pid=$!

  for _ in $(seq 600); do
    if vm pgrep -x "$program" >/dev/null 2>&1; then
      seen=true
      break
    fi
    sleep 0.5
  done

  limactl stop --force "$name"
  wait "$pid" || true
  wait_qemu_gone

  limactl start --tty=false "$name" && wait_ssh && $seen
}

status=0
count=0
# shellcheck disable=SC2329 # used by the suites, called by name
check() {
  local desc=$1 log
  shift
  count=$((count + 1))

  log=$dir/$(printf '%02d' "$count").log
  if "$@" >"$log" 2>&1; then
    printf '  \033[1;32mok\033[0m    %s\n' "$desc"
  else
    printf '  \033[1;31mfail\033[0m  %s (%s)\n' "$desc" "$log"
    status=1
  fi
}

setup() {
  local desc=$1
  shift

  echo "  $desc"
  echo "== $desc" >>"$dir/setup.log"
  "$@" >>"$dir/setup.log" 2>&1 || {
    echo "lima-it.sh: $desc failed, see $dir/setup.log" >&2
    exit 1
  }
}

# shellcheck disable=SC2329 # run by the trap
cleanup() {
  $keep && return
  limactl delete --force "$name" >/dev/null 2>&1 || true
}
trap cleanup EXIT

version=$(sed -n '/^\[workspace\.package\]/,/^\[/s/^version = "\(.*\)"/\1/p' Cargo.toml)
if $build; then
  echo "Building the package (log: $dir/setup.log)"
  # The VM test needs the package, not a second run of the test suite.
  setup "the package build" scripts/arch-package.sh --nocheck
fi

# Only this version, so an old package left in the folder is never tested.
# shellcheck disable=SC2012 # names come from makepkg
package=$(ls -t target/arch-package/garuda-toolbox-"$version"-*.pkg.tar.zst 2>/dev/null | head -n 1 || true)
[ -n "$package" ] || {
  echo "lima-it.sh: no garuda-toolbox $version package in target/arch-package, run without --no-build" >&2
  exit 2
}

echo "Starting $name"
limactl delete --force "$name" >/dev/null 2>&1 || true
setup "limactl start" limactl start --name="$name" --tty=false --set ".images = $images" \
  tests/lima/garuda.yaml
vm systemctl is-system-running --wait >>"$dir/setup.log" 2>&1 || true

echo "Installing into $name"
setup "limactl copy" limactl copy "$package" "$name:/tmp/"
setup "pacman" as_root "pacman -Syu --noconfirm --needed dkms v4l2loopback-dkms networkmanager \
  && pacman -U --noconfirm /tmp/${package##*/}"

# Over SSH no polkit agent runs, so pkexec would ask on the terminal for a
# password the Lima user does not have.
user=$(vm id -un)
setup "the polkit rule" as_root "printf '%s\n' 'polkit.addRule(function (action, subject) {' \
  '  if (action.id.indexOf(\"org.garudalinux.toolbox.\") === 0 && subject.user === \"$user\") {' \
  '    return polkit.Result.YES;' '  }' '});' >/etc/polkit-1/rules.d/50-garuda-toolbox-it.rules"

# A unit of the test's own, so no service the VM needs is touched. In
# /usr/lib, since systemctl mask refuses a unit whose file is in /etc.
setup "the test unit" as_root "printf '%s\n' '[Unit]' 'Description=garuda-toolbox Lima test' \
  '[Service]' 'Type=oneshot' 'RemainAfterExit=yes' 'ExecStart=/usr/bin/true' \
  '[Install]' 'WantedBy=multi-user.target' >/usr/lib/systemd/system/garuda-toolbox-it.service \
  && systemctl daemon-reload"

# Lima's own link stays with systemd-networkd, so ssh survives NetworkManager.
setup "the NetworkManager config" as_root "printf '[keyfile]\nunmanaged-devices=*\n' \
  >/etc/NetworkManager/conf.d/garuda-toolbox-it.conf"
own_root=$(vm findmnt -nvo SOURCE /)

# shellcheck source=lima-it/checks.sh
source scripts/lima-it/checks.sh
# shellcheck source=lima-it/upgrade.sh
source scripts/lima-it/upgrade.sh
# shellcheck source=lima-it/faults.sh
source scripts/lima-it/faults.sh
# shellcheck source=lima-it/accounts.sh
source scripts/lima-it/accounts.sh
# shellcheck source=lima-it/sdboot.sh
source scripts/lima-it/sdboot.sh

if [ "${#suite_list[@]}" -gt 1 ] || [ "${suite_list[0]}" != basic ]; then
  # On the disk, or the faults suite's power loss takes the setup with it.
  setup "sync" vm sync
  setup "the snapshot after the setup" limactl snapshot create "$name" --tag base
fi

first=true
for suite in "${suite_list[@]}"; do
  $first || setup "back to the snapshot" restore_base
  first=false
  echo "Checking $name, suite $suite:"
  "suite_$suite"
done

echo "logs: $dir"
exit "$status"
