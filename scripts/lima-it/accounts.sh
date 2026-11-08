# shellcheck shell=bash
# The accounts suite, sourced by scripts/lima-it.sh. A login through su
# on a terminal proves each change, as a person would log in.

it_user=garuda-it
first_password=Garuda-it-first-1
second_password=Garuda-it-second-2

suite_accounts() {
  check "users add creates the account with its password" t_user_add
  check "su logs in with the password" t_login "$first_password"
  check "su refuses a wrong password" t_no_login wrong-password
  check "users password sets a new one" t_user_password
  check "users edit --shell changes the login shell" t_user_shell
  check "users lock refuses the login" t_user_lock
  check "users unlock lets it log in again" t_user_unlock
  check "users remove --delete-home removes the account and its home" t_user_remove
}

# su_as <password>
# su drops what was typed before its prompt, so the password comes later.
su_as() {
  local typed
  typed=$(printf '%q' "$1")
  vm bash -c "{ sleep 3; printf '%s\n' $typed; } |
    timeout 60 script -qec 'su - $it_user -c whoami' /dev/null" | tr -d '\r'
}

t_login() {
  local out
  out=$(su_as "$1")
  echo "$out"
  grep -qx "$it_user" <<<"$out"
}

t_no_login() {
  local out
  out=$(su_as "$1")
  echo "$out"
  ! grep -qx "$it_user" <<<"$out"
}

t_user_add() {
  printf '%s\n' "$first_password" |
    tp users add "$it_user" --password-stdin --shell /bin/bash --yes &&
    vm getent passwd "$it_user"
}

t_user_password() {
  printf '%s\n' "$second_password" | tp users password "$it_user" --password-stdin --yes &&
    t_login "$second_password"
}

t_user_shell() {
  local shell
  vm cat /etc/shells

  # The image decides which other shells it has.
  # shellcheck disable=SC2016 # expands in the VM
  shell=$(vm sh -c 'grep -v -e "^#" -e "bash$" /etc/shells |
    while read -r s; do [ -x "$s" ] && echo "$s" && break; done')
  echo "shell: $shell"
  [ -n "$shell" ] || return 1
  tb users edit "$it_user" --shell "$shell" --yes &&
    [ "$(vm getent passwd "$it_user" | cut -d: -f7)" = "$shell" ] &&
    t_login "$second_password"
}

t_user_lock() { tb users lock "$it_user" --yes && t_no_login "$second_password"; }
t_user_unlock() { tb users unlock "$it_user" --yes && t_login "$second_password"; }

t_user_remove() {
  local home
  home=$(vm getent passwd "$it_user" | cut -d: -f6) || return 1
  echo "home: $home"
  tb users remove "$it_user" --delete-home --yes &&
    ! vm getent passwd "$it_user" &&
    ! vm test -e "$home"
}
