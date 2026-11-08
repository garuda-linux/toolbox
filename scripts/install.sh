#!/usr/bin/env bash
#   scripts/install.sh <destdir> <prefix> <dir with the built binaries>
set -euo pipefail

[ $# -eq 3 ] || {
  echo "usage: install.sh <destdir> <prefix> <dir with the built binaries>" >&2
  exit 2
}

destdir=$1
prefix=$2
binaries=$3
repo=$(cd "$(dirname "$0")/.." && pwd)
root="$destdir$prefix"
share="$root/share"

for binary in garuda-toolbox garuda-toolbox-helper; do
  [ -x "$binaries/$binary" ] || {
    echo "install.sh: build first, $binaries/$binary is missing" >&2
    exit 2
  }
done

install -Dm755 "$binaries/garuda-toolbox" "$root/bin/garuda-toolbox"
# pkexec starts the helper by its full path. It stays off PATH.
install -Dm755 "$binaries/garuda-toolbox-helper" "$root/lib/garuda-toolbox/garuda-toolbox-helper"

mkdir -p "$share/garuda-toolbox"
cp -r --no-preserve=mode,ownership "$repo/data/catalog" "$share/garuda-toolbox/catalog"

# The policy pins the helper's path, so it follows the prefix.
install -d "$share/polkit-1/actions"
sed "s|/usr/lib/garuda-toolbox/|$prefix/lib/garuda-toolbox/|g" \
  "$repo/data/org.garudalinux.toolbox.policy" \
  >"$share/polkit-1/actions/org.garudalinux.toolbox.policy"
chmod 644 "$share/polkit-1/actions/org.garudalinux.toolbox.policy"

# Named like the Wayland app_id, so the compositor finds the window's icon.
install -Dm644 "$repo/data/garuda-toolbox.desktop" "$share/applications/garuda-toolbox.desktop"
install -Dm644 "$repo/data/org.garudalinux.toolbox.metainfo.xml" \
  "$share/metainfo/org.garudalinux.toolbox.metainfo.xml"
install -Dm644 "$repo/app/gui/assets/garuda-logo.svg" \
  "$share/icons/hicolor/scalable/apps/garuda-toolbox.svg"
install -Dm644 "$repo/LICENSE" "$share/licenses/garuda-toolbox/LICENSE"

install -Dm644 "$repo/data/cli/garuda-toolbox.bash" \
  "$share/bash-completion/completions/garuda-toolbox"
install -Dm644 "$repo/data/cli/garuda-toolbox.fish" \
  "$share/fish/vendor_completions.d/garuda-toolbox.fish"
install -Dm644 "$repo/data/cli/_garuda-toolbox" "$share/zsh/site-functions/_garuda-toolbox"
for page in "$repo"/data/cli/*.1; do
  install -Dm644 "$page" "$share/man/man1/$(basename "$page")"
done
