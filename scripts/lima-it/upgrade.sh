# shellcheck shell=bash
# The upgrade suite, sourced by scripts/lima-it.sh. Real mirrors: an
# update, a package in and out, the maintenance tasks, and a second kernel
# in and out again.

lts=linux-lts
first_kernel=

suite_upgrade() {
  check "update leaves nothing to update" t_update
  check "packages install cowsay" t_package_install
  check "packages remove cowsay" t_package_remove
  check "remove-orphans removes a package left as a dependency" t_orphans
  check "clean-package-cache runs" tb maintenance run clean-package-cache --yes
  check "kernels install $lts" t_kernel_install
  check "kernels remove refuses the running kernel" t_kernel_running
  check "kernels remove $lts" t_kernel_remove
}

t_update() {
  tb update --yes || return 1
  local left
  left=$(vm pacman -Qu || true)
  echo "left: $left"
  [ -z "$left" ]
}

t_package_install() { tb packages install cowsay --yes && vm pacman -Q cowsay; }
t_package_remove() { tb packages remove cowsay --yes && ! vm pacman -Q cowsay; }

t_orphans() {
  as_root 'pacman -S --noconfirm --asdeps cowsay' &&
    vm pacman -Qdtq | grep -qx cowsay &&
    tb maintenance run remove-orphans --yes &&
    ! vm pacman -Q cowsay
}

t_kernel_install() {
  first_kernel=$(tj kernels list | jq -er '.rows[] | select(.running) | .id') || return 1
  echo "running: $first_kernel"
  tb kernels install "$lts" --yes && vm pacman -Q "$lts" && vm test -e "/boot/vmlinuz-$lts"
}

t_kernel_running() {
  local code=0
  tb kernels remove "$first_kernel" --yes || code=$?
  echo "exit: $code"
  [ "$code" != 0 ] && vm pacman -Q "$first_kernel"
}

t_kernel_remove() {
  tb kernels remove "$lts" --yes && ! vm pacman -Q "$lts" && ! vm test -e "/boot/vmlinuz-$lts"
}
