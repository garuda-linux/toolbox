# shellcheck shell=bash
# The faults suite, sourced by scripts/lima-it.sh. A power loss while
# pacman installs, then a system without network, from the snapshot.

lock=/var/lib/pacman/db.lck
offline=garuda_toolbox_it_offline

suite_faults() {
  check "a power loss while pacman installs" stop_during pacman tb packages install linux-lts --yes
  check "remove-database-lock removes the stale lock" t_stale_lock
  check "the install works afterwards, the database is whole" t_install_after
  setup "back to the snapshot" restore_base
  setup "nftables" as_root 'pacman -S --needed --noconfirm nftables'
  setup "no network" go_offline
  check "without network an install fails, without hanging" t_offline
  setup "the network back" as_root "nft delete table inet $offline"
}

# The lock survives only when the file system wrote it before the power
# went, and the next power loss may leave one.
t_stale_lock() {
  if vm test -e "$lock"; then
    echo "the lock survived the power loss"
  else
    echo "no lock survived, one is made"
    as_root "touch $lock" || return 1
  fi
  tb maintenance run remove-database-lock --yes && ! vm test -e "$lock"
}

t_install_after() {
  tb packages install linux-lts --yes && vm pacman -Q linux-lts && as_root 'pacman -Dk'
}

# Refused at once, as without a route, and SSH keeps its own connection.
go_offline() {
  as_root "nft add table inet $offline &&
    nft add chain inet $offline out '{ type filter hook output priority 0; }' &&
    nft add rule inet $offline out ct state established,related accept &&
    nft add rule inet $offline out oif lo accept &&
    nft add rule inet $offline out reject"
}

t_offline() {
  local out code=0
  out=$(tb packages install cowsay --yes) || code=$?
  echo "$out"
  echo "exit: $code"
  [ "$code" != 0 ] && [ "$code" != 124 ] && ! vm pacman -Q cowsay
}
