#!/usr/bin/env bash
set -euo pipefail

[ -f /etc/arch-release ] || {
  echo "install-deps.sh: only for the Arch test container" >&2
  exit 2
}

# The image's NoExtract drops the glibc locale data the Locales tests need.
sed -i '/^NoExtract/d' /etc/pacman.conf
pacman -Sy --noconfirm glibc

# pacman ships libalpm's headers and libalpm.pc for the alpm crate.
pacman -Syu --noconfirm --needed \
  base-devel \
  pacman \
  rust \
  cargo-nextest \
  clang \
  pkgconf \
  git \
  pacman-contrib \
  namcap \
  archlinux-appstream-data \
  wayland \
  libxkbcommon \
  vulkan-icd-loader \
  vulkan-swrast \
  weston \
  breeze-icons \
  ttf-dejavu
pacman -Scc --noconfirm
