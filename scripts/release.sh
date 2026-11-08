#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

command -v git-cliff >/dev/null || exec nix develop --command "$0" "$@"

if [ -n "$(git status --porcelain -- . ':!CHANGELOG.md')" ]; then
  echo "the working tree has changes, commit or stash them first" >&2
  exit 1
fi

if git describe --tags --abbrev=0 >/dev/null 2>&1; then
  version=$(cz bump --get-next)
  cz bump --files-only --yes
  git-cliff --unreleased --tag "$version" --prepend CHANGELOG.md
else
  version=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)
  link="https://gitlab.com/garuda-linux/applications/toolbox/-/releases/$version"
  sed -i "s|^## Unreleased$|## [$version]($link) — $(date +%F)|" CHANGELOG.md
fi

treefmt CHANGELOG.md
git add --all
git commit --message "chore(release): $version"
git tag --annotate --message "$version" "$version"
echo "tagged $version. Push with: git push --follow-tags"
