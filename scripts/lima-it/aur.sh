# shellcheck shell=bash
# The aur suite, sourced by scripts/lima-it.sh. The CLI installs AUR packages
# on a terminal through the split path: the review asks, paru fetches,
# makepkg builds as the user, the helper installs. Needs the network and the
# AUR, and takes long, since octopi-git builds for most of it. paru comes
# from chaotic-aur, and a package that a repository offers is never built.

aur_pkg=
aur_pair=octopi-git

# Runs the CLI on a terminal and types the answer once the question shows,
# since a key typed before the question is dropped by the review. Arguments:
# the answer, the question's text, then the CLI's arguments.
# shellcheck disable=SC2016 # the body expands in the VM
answer_body='
answer=$1 question=$2
shift 2
printf -v cmd "%q " env GARUDA_TOOLBOX_PAGER=cat NO_COLOR=1 TERM=dumb garuda-toolbox "$@"
log=$(mktemp)
fifo=$(mktemp -u)
mkfifo "$fifo"
timeout 1800 script -qefc "$cmd" "$log" <"$fifo" >/dev/null 2>&1 &
pid=$!
exec 3>"$fifo"
for _ in $(seq 600); do
  grep -qF -- "$question" "$log" 2>/dev/null && break
  kill -0 "$pid" 2>/dev/null || break
  sleep 1
done
printf "%s\n" "$answer" >&3
wait "$pid"
status=$?
exec 3>&-
tr -d "\r" <"$log"
rm -f "$log" "$fifo"
exit "$status"
'

# shellcheck disable=SC2329 # used by the checks below
answer_tb() {
  local answer=$1 question=$2
  shift 2
  vm bash -c "$answer_body" answer-tb "$answer" "$question" "$@"
}

# The review's question, which every install that builds shows.
review_question="press Enter to build"

# A package that no repository offers and the AUR has, as the owner's capture
# script tested it. The suite stops when none qualifies.
# shellcheck disable=SC2329 # used by the setup
pick_aur_package() {
  local name
  for name in pipes.sh cbonsai bash-pipes tty-clock-git nyancat-git; do
    if ! vm pacman -Si "$name" >/dev/null 2>&1 && vm paru -Si --aur "$name" >/dev/null 2>&1; then
      aur_pkg=$name
      echo "picked $name"
      return 0
    fi
  done

  echo "no candidate is an AUR package that no repository offers" >&2
  return 1
}

t_review_enter() {
  local out code=0
  out=$(answer_tb "" "$review_question" packages install --aur "$aur_pkg" --yes) || code=$?
  echo "$out"
  echo "exit: $code"
  [ "$code" = 3 ] && grep -qF "$review_question" <<<"$out" && ! vm pacman -Q "$aur_pkg" >/dev/null 2>&1
}

t_install() {
  local out code=0
  out=$(answer_tb "y" "$review_question" packages install --aur "$aur_pkg" --yes) || code=$?
  echo "$out"
  echo "exit: $code"
  [ "$code" = 0 ] || return 1
  # The VM's polkit rule runs the helper without a password, so no
  # password question may show on the terminal.
  ! grep -qi password <<<"$out" || return 1
  vm pacman -Q "$aur_pkg" || return 1
  vm pacman -Qmq | grep -qx "$aur_pkg" || return 1
  # shellcheck disable=SC2016 # expands in the VM
  vm sudo sh -c 'test -z "$(ls -A /var/cache/garuda-toolbox/aur 2>/dev/null)"'
}

t_json_refused() {
  local out code=0
  out=$(tb packages install --aur "$aur_pkg" --json --yes) || code=$?
  echo "$out"
  echo "exit: $code"
  [ "$code" = 4 ] || return 1
  grep '"event":"error"' <<<"$out" | jq -e '.message | contains("asks questions")' >/dev/null
}

t_reinstall() {
  local code=0
  answer_tb "y" "$review_question" packages install --aur "$aur_pkg" --yes >/dev/null || code=$?
  [ "$code" = 0 ] && vm pacman -Qeq | grep -qx "$aur_pkg"
}

t_aur_dependency() {
  local before makes out code=0 dep
  before=$(vm pacman -Qq) || return 1
  out=$(answer_tb "y" "$review_question" packages install --aur "$aur_pair" --yes) || code=$?
  echo "$out"
  echo "exit: $code"
  [ "$code" = 0 ] || return 1
  vm pacman -Q "$aur_pair" >/dev/null || return 1

  makes=$(tj packages show --aur "$aur_pair" | jq -r '(.make_depends // []) - (.depends // []) | .[]' | sed 's/[<>=].*//') || return 1
  for dep in $makes; do
    grep -qx "$dep" <<<"$before" && continue
    if vm pacman -Q "$dep" >/dev/null 2>&1; then
      echo "make dependency $dep is still installed"
      return 1
    fi
  done
}

t_symlinked_cache() {
  local out code=0
  # shellcheck disable=SC2016 # expands in the VM
  vm bash -c 'cd && if [ -e .cache ] && [ ! -L .cache ]; then mv .cache .cache-it; fi;
    mkdir -p .cache-it && rm -f .cache && ln -s .cache-it .cache' || return 1
  out=$(tb packages install --aur "$aur_pkg" --dry-run) || code=$?
  # shellcheck disable=SC2016 # expands in the VM
  vm bash -c 'cd && rm -f .cache && if [ -d .cache-it ]; then mv .cache-it .cache; fi'
  echo "$out"
  echo "exit: $code"
  [ "$code" = 0 ] && grep -qF "old paru path" <<<"$out"
}

# A `-git` package of the candidates that conflicts with a repository
# package, as "name conflict". Nothing of that kind fails the check.
conflict_pair() {
  local name c conflicts
  for name in tty-clock-git nyancat-git; do
    conflicts=$(tj packages show --aur "$name" 2>/dev/null | jq -r '.conflicts[]?' | sed 's/[<>=].*//') || true
    while read -r c; do
      [ -n "$c" ] || continue
      if vm pacman -Si "$c" >/dev/null 2>&1; then
        echo "$name $c"
        return 0
      fi
    done <<<"$conflicts"
  done

  return 1
}

t_conflict() {
  local pair name c out code=0
  pair=$(conflict_pair) || {
    echo "no candidate conflicts with a repository package"
    return 1
  }
  read -r name c <<<"$pair"
  echo "conflict: $name with $c"

  tb packages install "$c" --yes >/dev/null || return 1
  out=$(tb packages install --aur "$name" --yes) || code=$?
  tb packages remove "$c" --yes >/dev/null 2>&1 || true
  echo "$out"
  echo "exit: $code"
  [ "$code" != 0 ] && grep -qF "$c" <<<"$out" && ! vm pacman -Q "$name" >/dev/null 2>&1
}

# The helper run by hand under sudo, with a socket on its stdin made by
# python3. The CLI never reaches the split path under sudo, so this asks the
# helper whether sudo passed the socket on, with a read that needs the caller.
sudo_probe='import socket, subprocess
app, helper = socket.socketpair()
run = subprocess.run(["sudo", "--", "/usr/lib/garuda-toolbox/garuda-toolbox-helper", "read", "shadow-locks"], stdin=helper, capture_output=True, text=True)
print("exit", run.returncode)
print("stdout", run.stdout.strip()[:300])
print("stderr", run.stderr.strip()[:300])'

t_sudo_socket() {
  local out
  vm id -u || true
  vm sh -c 'grep -rhs use_pty /etc/sudoers /etc/sudoers.d || echo "use_pty not set"' || true
  out=$(vm python3 -I -c "$sudo_probe") || {
    echo "the probe did not run"
    return 1
  }
  echo "$out"
  if grep -q '^exit 0$' <<<"$out"; then
    echo "K8: sudo keeps the socket as the helper's stdin, the helper learned the caller"
  else
    echo "K8: sudo does not keep the socket, or the caller was refused (see the exit above)"
  fi
  grep -q '^exit ' <<<"$out"
}

suite_aur() {
  setup "paru, git, base-devel and python" as_root "pacman -Sy --noconfirm --needed paru git base-devel python"
  setup "an AUR package no repository offers" pick_aur_package
  check "the review asks with --yes, and Enter declines: nothing built, exit 3" t_review_enter
  check "y builds and installs it: foreign, no password asked, root store empty" t_install
  check "--json refuses the review up front with exit 4" t_json_refused
  check "a second install keeps the install reason" t_reinstall
  check "an AUR dependency installs and the make dependencies are removed" t_aur_dependency
  check "a symlinked ~/.cache takes the old paru path, and says so" t_symlinked_cache
  check "a package that conflicts with an installed one is refused, naming it" t_conflict
  check "K8: whether sudo keeps the helper's stdin socket" t_sudo_socket
}
