#!/usr/bin/env bash
# Changes the system. Never run it on a machine you use.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)

[ -f /etc/arch-release ] || {
  echo "setup.sh: only for the Arch test container" >&2
  exit 2
}

id tester >/dev/null 2>&1 || useradd --create-home tester
getent group toolbox-test-group >/dev/null || groupadd toolbox-test-group

install -d -o tester /srv/toolbox-test /srv/toolbox-test-older
install -m755 "$here/make-repo.sh" /usr/local/bin/make-repo.sh
su tester -c "/usr/local/bin/make-repo.sh /srv/toolbox-test /srv/toolbox-test-older"

install -m644 "$here/pacman.conf" /etc/pacman.conf
pacman -Sy --noconfirm
pacman -U --noconfirm /srv/toolbox-test-older/toolbox-test-up-1.0-1-any.pkg.tar.zst
