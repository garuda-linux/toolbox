#!/usr/bin/env bash
#   package.sh <checkout> <output dir> [makepkg arguments]
set -euo pipefail

[ $# -ge 2 ] || {
  echo "usage: package.sh <checkout> <output dir> [makepkg arguments]" >&2
  exit 2
}

if [ ! -f /etc/arch-release ] || [ "$(id -u)" != 0 ]; then
  echo "package.sh: only as root in the Arch container" >&2
  exit 2
fi

checkout=$(cd "$1" && pwd)
mkdir -p "$2"
out=$(cd "$2" && pwd)
shift 2

id builder >/dev/null 2>&1 || useradd --create-home builder

# A copy, because the checkout may be read-only and makepkg writes next to the PKGBUILD.
work=/home/builder/garuda-toolbox
rm -rf "$work"
mkdir -p "$work"
tar -C "$checkout" \
  --exclude=./target --exclude=./.git --exclude=./reference \
  --exclude=./.direnv --exclude=./.cargo-home --exclude="./result*" \
  -cf - . | tar -C "$work" -xf -

cache=()
for variable in CARGO_HOME CARGO_TARGET_DIR; do
  if [ -n "${!variable:-}" ]; then
    mkdir -p "${!variable}"
    chown -R builder: "${!variable}"
    cache+=("$variable=${!variable}")
  fi
done
# Rootless podman would otherwise leave files the host user cannot remove.
trap 'chown -R 0:0 "$out"' EXIT
chown -R builder: "$work" "$out"

# runuser keeps root's HOME, where cargo and the tests could not write.
runuser -u builder -- env -C "$work/packaging" HOME=/home/builder "${cache[@]}" PKGDEST="$out" \
  makepkg --noconfirm --force "$@"

{
  echo "== PKGBUILD"
  namcap "$work/packaging/PKGBUILD" || true
  for package in "$out"/garuda-toolbox-*.pkg.tar.*; do
    echo "== ${package##*/}"
    namcap "$package" || true
  done
} 2>&1 | tee "$out/namcap.log"
