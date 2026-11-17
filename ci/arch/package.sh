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
  --exclude=./.direnv --exclude=./.cargo-home --exclude=./.claude --exclude="./result*" \
  -cf - . | tar -C "$work" -xf -

cache=()
for variable in CARGO_HOME CARGO_TARGET_DIR; do
  if [ -n "${!variable:-}" ]; then
    mkdir -p "${!variable}"
    chown -R builder: "${!variable}"
    cache+=("$variable=${!variable}")
  fi
done

dest=$work/pkgdest
mkdir -p "$dest"
chown -R builder: "$work"

# runuser keeps root's HOME, where cargo and the tests could not write.
# The container is removed afterwards, so a failed snapshot's pictures are
# kept in the output folder.
runuser -u builder -- env -C "$work/packaging" HOME=/home/builder "${cache[@]}" PKGDEST="$dest" \
  makepkg --noconfirm --force "$@" || {
  failed=$?
  find "$work" \( -name '*.new.png' -o -name '*.diff.png' \) -exec cp --parents -t "$out" {} + 2>/dev/null || true
  echo "package.sh: snapshot pictures of the failed run are below $out$work" >&2
  exit "$failed"
}
cp "$dest"/garuda-toolbox-*.pkg.tar.* "$out"/

{
  echo "== PKGBUILD"
  namcap "$work/packaging/PKGBUILD" || true
  for package in "$dest"/garuda-toolbox-*.pkg.tar.*; do
    echo "== ${package##*/}"
    namcap "$package" || true
  done
} 2>&1 | tee "$out/namcap.log"
