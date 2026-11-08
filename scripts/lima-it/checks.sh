# shellcheck shell=bash
# shellcheck disable=SC2154 # own_root comes from scripts/lima-it.sh
# Sourced by scripts/lima-it.sh. Each check drives the CLI, then reads the
# VM's state with the VM's own tools, not with the toolbox.

unit=garuda-toolbox-it.service

# A read as JSON, also copied into the check's log. Not tee /dev/stderr,
# which opens the log again at its start and overwrites what is there.
tj() {
  local out
  out=$(tb "$@" --json) || return 1
  printf '%s\n' "$out" >&2
  printf '%s\n' "$out"
}

service_checks() {
  check "services list shows the test unit" t_service_list
  check "services enable" t_unit enable is-enabled enabled
  check "services start" t_unit start is-active active
  check "services restart" t_unit restart is-active active
  check "services stop" t_unit stop is-active inactive
  check "services disable" t_unit disable is-enabled disabled
  check "services mask" t_unit mask is-enabled masked
  check "services unmask" t_unit unmask is-enabled disabled
  # Enabled again for the check after the restart.
  check "services enable again" t_unit enable is-enabled enabled
}

t_service_list() { tj services list | jq -e --arg u "$unit" 'any(.rows[]; .id == $u)'; }

# <command> <systemctl query> <expected answer>
t_unit() {
  tb services "$1" "$unit" --yes || return 1
  [ "$(vm systemctl "$2" "$unit" || true)" = "$3" ]
}

locale_checks() {
  check "locales enable generates the locale" t_locale_enable
  check "locales system writes /etc/locale.conf" t_locale_system
  check "locales list marks it enabled and the system's" t_locale_list
}
t_locale_enable() {
  tb locales enable de_DE.UTF-8 --yes &&
    as_root 'grep -qx "de_DE.UTF-8 UTF-8" /etc/locale.gen && locale -a | grep -qx de_DE.utf8'
}
t_locale_system() {
  tb locales system de_DE.UTF-8 --yes && as_root 'grep -qx LANG=de_DE.UTF-8 /etc/locale.conf'
}
t_locale_list() {
  tj locales list | jq -e 'any(.rows[]; .id == "de_DE.UTF-8" and .enabled and .system)'
}

dkms_checks() {
  # pacman's hook built the module already, the test wants it unbuilt.
  # shellcheck disable=SC2016 # expands in the VM
  setup "dkms unbuild" as_root \
    'for m in $(dkms status -m v4l2loopback | sed "s/[,:].*//" | sort -u); do dkms unbuild "$m" --all; done'
  check "dkms list shows v4l2loopback unbuilt" t_dkms_unbuilt
  check "dkms build installs v4l2loopback" t_dkms_build
}
t_dkms_unbuilt() {
  tj dkms list | jq -e 'any(.rows[]; .id == "v4l2loopback" and .state == "added")'
}
t_dkms_build() {
  tb dkms build --yes &&
    tj dkms list |
    jq -e '[.rows[] | select(.id == "v4l2loopback")] | length > 0 and all(.state == "installed")'
}

network_checks() {
  check "services start NetworkManager.service" t_nm_start
  # No radio checks: the VM has no radio, so rfkill has nothing to switch.
  check "network drivers blacklist virtio_net" t_driver blacklist true
  # Before the restart, or the VM comes back without its network.
  check "network drivers allow virtio_net" t_driver allow false
}
t_nm_start() { tb services start NetworkManager.service --yes && vm nmcli -t general status; }
# <blacklist|allow> <blacklisted afterwards>
t_driver() {
  tb network drivers "$1" virtio_net --yes || return 1
  local listed=false
  as_root 'grep -qsw virtio_net /etc/modprobe.d/garuda-toolbox-blacklist.conf' && listed=true
  [ "$listed" = "$2" ] &&
    tj network drivers list |
    jq -e --argjson b "$2" 'any(.rows[]; .id == "virtio_net" and .blacklisted == $b)'
}

after_restart_checks() {
  check "the VM starts from its own disk" t_own_root
  check "the enabled test unit started at boot" vm systemctl is-active --quiet "$unit"
  # No locale check: Lima gives cloud-init a new instance id on each start,
  # so its locale module sets en_US.UTF-8 again.
}
t_own_root() { [ "$(vm findmnt -nvo SOURCE /)" = "$own_root" ]; }

suite_basic() {
  service_checks
  locale_checks
  dkms_checks
  network_checks
  echo "Restarting $name"
  setup "limactl stop" limactl stop "$name"
  setup "limactl start" limactl start --tty=false "$name"
  setup "SSH" wait_ssh
  after_restart_checks
}
