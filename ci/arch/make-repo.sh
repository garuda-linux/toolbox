#!/usr/bin/env bash
#   make-repo.sh <repository dir> <cache dir for older versions>
set -euo pipefail

repo=$1
older=$2
work=$(mktemp -d)
mkdir -p "$repo" "$older"

build() {
  local name=$1 version=$2 extra=${3:-}
  local dir="$work/$name-$version"

  mkdir -p "$dir"
  cat >"$dir/PKGBUILD" <<EOF
pkgname=$name
pkgver=$version
pkgrel=1
pkgdesc="Garuda Toolbox integration test package"
arch=(any)
license=(GPL-3.0-or-later)
options=(!debug)
$extra
package() {
  install -Dm644 /dev/null "\$pkgdir/usr/share/toolbox-test/$name"
}
EOF
  (cd "$dir" && makepkg --nodeps --noconfirm --clean >/dev/null)
  echo "$dir"/*.pkg.tar.zst
}

cp "$(build toolbox-test-dep 1.0)" "$repo/"
cp "$(build toolbox-test-a 1.0 'depends=(toolbox-test-dep)')" "$repo/"
cp "$(build toolbox-test-conflict 1.0 'conflicts=(toolbox-test-a)')" "$repo/"

# Installed at 1.0 in the image, offered at 1.1, so the system upgrade has work.
cp "$(build toolbox-test-up 1.0)" "$older/"
cp "$(build toolbox-test-up 1.1)" "$repo/"

# The Kernels page looks for a `linux*` name with a `-headers` package of the same version.
cp "$(build linux-toolbox-test 1.0)" "$repo/"
cp "$(build linux-toolbox-test-headers 1.0)" "$repo/"

repo-add --quiet "$repo/toolbox-test.db.tar.gz" "$repo"/*.pkg.tar.zst
rm -rf "$work"
