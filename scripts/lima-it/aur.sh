# shellcheck shell=bash
# The aur suite, sourced by scripts/lima-it.sh. The CLI installs AUR packages
# on a terminal through the split path: the review asks, paru fetches,
# makepkg builds as the user, the helper installs. Needs the network and the
# AUR. paru comes from chaotic-aur, and a package that a repository offers
# is never built.

aur_pkg=
# Builds in seconds and has a make dependency no Garuda image installs.
aur_make=cbonsai

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
  local candidate
  for candidate in pipes.sh cbonsai bash-pipes tty-clock-git nyancat-git; do
    if ! vm pacman -Si "$candidate" >/dev/null 2>&1 && vm paru -Si --aur "$candidate" >/dev/null 2>&1; then
      aur_pkg=$candidate
      echo "picked $candidate"
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
  local out code=0
  out=$(tb packages install --aur "$aur_pkg" --yes) || code=$?
  echo "$out"
  echo "exit: $code"
  [ "$code" = 0 ] && ! grep -qF "makepkg" <<<"$out" && vm pacman -Qeq | grep -qx "$aur_pkg"
}

t_make_dependency() {
  local before makes out code=0 dep
  before=$(vm pacman -Qq) || return 1
  out=$(answer_tb "y" "$review_question" packages install --aur "$aur_make" --yes) || code=$?
  echo "$out"
  echo "exit: $code"
  [ "$code" = 0 ] || return 1
  vm pacman -Q "$aur_make" >/dev/null || return 1

  makes=$(tj packages show --aur "$aur_make" | jq -r '(.make_depends // []) - (.depends // []) | .[]' | sed 's/[<>=].*//') || return 1
  for dep in $makes; do
    grep -qx "$dep" <<<"$before" && continue
    if vm pacman -Q "$dep" >/dev/null 2>&1; then
      echo "make dependency $dep is still installed"
      return 1
    fi
  done
}

# Makes $aur_make outdated as an earlier split install would have left it:
# the build of the AUR's commit before the newest is installed, and the
# app's clone and its reviewed mark sit at that commit. Then the upgrade must
# show the changes, build the new version and keep the install reason.
# An older commit that no longer builds skips the check, since that is the
# AUR's and not the toolbox's.
# shellcheck disable=SC2016 # the body expands in the VM
outdated_body='
name=$1
clone=$HOME/.cache/garuda-toolbox/aur-clone/$name
rm -rf /tmp/old-aur
git clone -q "https://aur.archlinux.org/$name.git" /tmp/old-aur || exit 3
cd /tmp/old-aur || exit 3
old=$(git log --format=%H -n 2 | tail -n 1)
[ "$old" != "$(git rev-parse HEAD)" ] || exit 3
git checkout -q "$old" || exit 3
makepkg -s --noconfirm >/dev/null 2>&1 || exit 3
sudo pacman -U --noconfirm --asexplicit ./*.pkg.tar.zst >/dev/null 2>&1 || exit 3
git -C "$clone" reset -q --hard "$old" || exit 4
git -C "$clone" update-ref refs/garuda-toolbox/reviewed "$old" || exit 4
'

t_upgrade() {
  local before after out code=0 made=0
  vm bash -c "$outdated_body" outdated "$aur_make" || made=$?
  if [ "$made" = 3 ]; then
    echo "skipped: the commit before the newest of $aur_make does not build"
    return 0
  fi
  [ "$made" = 0 ] || return 1

  before=$(vm pacman -Q "$aur_make") || return 1
  out=$(answer_tb "y" "$review_question" maintenance run aur-upgrade --yes) || code=$?
  echo "$out"
  echo "exit: $code"
  after=$(vm pacman -Q "$aur_make") || return 1
  echo "before: $before, after: $after"

  [ "$code" = 0 ] &&
    grep -qF "$aur_make: changes since the last review" <<<"$out" &&
    [ "$before" != "$after" ] &&
    vm pacman -Qeq | grep -qx "$aur_make"
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
  local candidate c conflicts
  for candidate in tty-clock-git nyancat-git; do
    conflicts=$(tj packages show --aur "$candidate" 2>/dev/null | jq -r '.conflicts[]?' | sed 's/[<>=].*//') || true
    while read -r c; do
      [ -n "$c" ] || continue
      if vm pacman -Si "$c" >/dev/null 2>&1; then
        echo "$candidate $c"
        return 0
      fi
    done <<<"$conflicts"
  done

  return 1
}

t_conflict() {
  local pair candidate c out code=0
  pair=$(conflict_pair) || {
    echo "no candidate conflicts with a repository package"
    return 1
  }
  read -r candidate c <<<"$pair"
  echo "conflict: $candidate with $c"

  tb packages install "$c" --yes >/dev/null || return 1
  out=$(tb packages install --aur "$candidate" --yes) || code=$?
  tb packages remove "$c" --yes >/dev/null 2>&1 || true
  echo "$out"
  echo "exit: $code"
  [ "$code" != 0 ] && grep -qF "$c" <<<"$out" && ! vm pacman -Q "$candidate" >/dev/null 2>&1
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
  check "--json refuses the review up front with exit 4" t_json_refused
  check "y builds and installs it: foreign, no password asked, root store empty" t_install
  check "a second install builds nothing and keeps the install reason" t_reinstall
  check "a make dependency is installed for the build and removed after it" t_make_dependency
  check "an outdated package is upgraded: the changes are shown, the reason is kept" t_upgrade
  check "a symlinked ~/.cache takes the old paru path, and says so" t_symlinked_cache
  check "a package that conflicts with an installed one is refused, naming it" t_conflict
  check "K8: whether sudo keeps the helper's stdin socket" t_sudo_socket
}
