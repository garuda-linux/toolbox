# shellcheck shell=bash
# shellcheck disable=SC2154 # name comes from scripts/lima-it.sh
# The systemd-boot suite, sourced by scripts/lima-it.sh. The image boots
# UKIs that kernel-install builds with /etc/kernel/cmdline inside, so a
# restart proves the rebuilt image starts.

loader_conf=/efi/loader/loader.conf
cmdline=/etc/kernel/cmdline
page_rule=8

suite_sdboot() {
  setup "binutils for objcopy" as_root 'pacman -S --noconfirm --needed binutils'
  conf_before=$(as_root "cat $loader_conf")
  check "boot show names systemd-boot" t_sdboot_show
  check "boot set timeout writes loader.conf and keeps the old one" t_sdboot_timeout
  check "the same timeout again keeps the first backup" t_sdboot_same_timeout
  check "boot set editor off" t_sdboot_editor
  check "boot set refuses a GRUB setting" t_sdboot_grub_key
  check "kernels default linux-garuda saves its flavour" t_sdboot_default
  check "boot param add rebuilds the UKI with it" t_sdboot_param
  check "boot param remove refuses rw" t_sdboot_protected
  echo "Restarting $name"
  setup "limactl stop" limactl stop "$name"
  setup "limactl start" limactl start --tty=false "$name"
  setup "SSH" wait_ssh
  check "the kernel started with the new parameter" t_sdboot_running_param
  check "systemd-boot started the default entry" t_sdboot_selected
}

t_sdboot_show() { tj boot show | jq -e '.loader == "systemd-boot"'; }

t_sdboot_timeout() {
  tb boot set timeout 5 --yes || return 1
  as_root "grep -qx 'timeout 5' $loader_conf" &&
    [ "$(as_root "cat $loader_conf.bak")" = "$conf_before" ]
}

t_sdboot_same_timeout() {
  tb boot set timeout 5 --yes || return 1
  [ "$(as_root "cat $loader_conf.bak")" = "$conf_before" ]
}

t_sdboot_editor() { tb boot set editor off --yes && as_root "grep -qx 'editor no' $loader_conf"; }

t_sdboot_grub_key() {
  local before code=0
  before=$(as_root "cat $loader_conf")
  tb boot set submenus on --yes || code=$?
  echo "exit: $code"
  [ "$code" = "$page_rule" ] && [ "$(as_root "cat $loader_conf")" = "$before" ]
}

t_sdboot_default() {
  tb kernels default linux-garuda --yes &&
    as_root "grep -qxF 'default garuda-*-garuda.efi' $loader_conf"
}

# objcopy with one file would rewrite the UKI in place, so it writes a copy.
t_sdboot_param() {
  tb boot param add nowatchdog --yes || return 1
  as_root "grep -qw nowatchdog $cmdline && ! grep -qw nowatchdog $cmdline.bak" || return 1
  # shellcheck disable=SC2016 # expands in the VM
  as_root 'for uki in /efi/EFI/Linux/*.efi; do
    case $uki in *-fallback.efi) continue ;; esac
    objcopy --dump-section .cmdline=/tmp/uki-cmdline "$uki" /tmp/uki-copy || exit 1
    grep -qw nowatchdog /tmp/uki-cmdline || exit 1
  done'
}

t_sdboot_protected() {
  local before code=0
  before=$(as_root "cat $cmdline")
  tb boot param remove rw --yes || code=$?
  echo "exit: $code"
  [ "$code" = "$page_rule" ] && [ "$(as_root "cat $cmdline")" = "$before" ]
}

t_sdboot_running_param() { vm grep -qw nowatchdog /proc/cmdline; }

t_sdboot_selected() {
  as_root 'bootctl list --json=short --no-pager' |
    jq -e 'any(.[]; .isSelected and .isDefault and (.id | test("^garuda-.*-garuda(\\+[0-9-]+)?\\.efi$")))'
}
