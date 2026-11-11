#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

command -v git-cliff >/dev/null || exec nix develop --command "$0" "$@"

if [ -n "$(git status --porcelain)" ]; then
  echo "the working tree has changes, commit or stash them first" >&2
  exit 1
fi

# No CI runner has KVM, so the VM tests are a promise made here.
read -r -p "Did scripts/lima-it.sh pass on $(git rev-parse --short HEAD)? [y/N] " answer
case $answer in
y | Y) ;;
*)
  echo "run scripts/lima-it.sh first" >&2
  exit 1
  ;;
esac

version=$(cz bump --get-next)
today=$(date +%F)
cz bump --files-only --yes

# Software centres list the versions from the metainfo.
metainfo=data/org.garudalinux.toolbox.metainfo.xml
sed -i "s|^  <releases>$|&\n    <release version=\"$version\" date=\"$today\"/>|" "$metainfo"
if ! grep -q "<release version=\"$version\"" "$metainfo"; then
  echo "no <releases> in $metainfo to add $version to" >&2
  exit 1
fi

if grep -q '^## Unreleased$' CHANGELOG.md; then
  link="https://gitlab.com/garuda-linux/applications/toolbox/-/releases/$version"
  sed -i "s|^## Unreleased$|## [$version]($link) — $today|" CHANGELOG.md
else
  git-cliff --unreleased --tag "$version" --prepend CHANGELOG.md
fi

# The commit hook fails on files treefmt would still change.
treefmt CHANGELOG.md
git add --all
git commit --message "chore(release): $version"
git tag --annotate --message "$version" "$version"
echo "tagged $version. Push with: git push --follow-tags"
