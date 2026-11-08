#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

command -v git-cliff >/dev/null || exec nix develop --command "$0" "$@"

if [ -n "$(git status --porcelain)" ]; then
  echo "the working tree has changes, commit or stash them first" >&2
  exit 1
fi

version=$(cz bump --get-next)
cz bump --files-only --yes

if grep -q '^## Unreleased$' CHANGELOG.md; then
  link="https://gitlab.com/garuda-linux/applications/toolbox/-/releases/$version"
  sed -i "s|^## Unreleased$|## [$version]($link) — $(date +%F)|" CHANGELOG.md
else
  git-cliff --unreleased --tag "$version" --prepend CHANGELOG.md
fi

# The commit hook fails on files treefmt would still change.
treefmt CHANGELOG.md
git add --all
git commit --message "chore(release): $version"
git tag --annotate --message "$version" "$version"
echo "tagged $version. Push with: git push --follow-tags"
