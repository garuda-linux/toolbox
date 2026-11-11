#!/usr/bin/env bash
# Sourced from the checkout's root. `lima_image <path>` downloads the
# official Garuda Lima image to <path> when no file is there yet, and
# checks it against the SHA-256 the package lists.

lima_image_project=88656556
lima_image_api="https://gitlab.com/api/v4/projects/$lima_image_project/packages"
lima_image_url="$lima_image_api/generic/garuda-lima/latest/garuda-lima.qcow2"

lima_image() {
  local path=$1
  [ -f "$path" ] && return 0

  # `latest` keeps every upload under one name, the newest is the one served.
  local package sha
  package=$(curl -fsSL "$lima_image_api?package_name=garuda-lima&package_version=latest" |
    jq -r '[.[] | select(.name == "garuda-lima" and .version == "latest")][0].id') || return 1
  sha=$(curl -fsSL "$lima_image_api/$package/package_files" |
    jq -r '[.[] | select(.file_name == "garuda-lima.qcow2")] | sort_by(.created_at) | last | .file_sha256') || return 1

  [ -n "$sha" ] && [ "$sha" != null ] || {
    echo "lima-image: the package lists no garuda-lima.qcow2" >&2
    return 1
  }

  echo "Downloading the Garuda Lima image to $path (about 2 GB)"
  mkdir -p "$(dirname "$path")"
  curl -fL --retry 3 --output "$path.part" "$lima_image_url" || return 1

  if [ "$(sha256sum "$path.part" | cut -d' ' -f1)" != "$sha" ]; then
    rm -f "$path.part"
    echo "lima-image: the download does not match $sha, a newer upload may have come in, try again" >&2
    return 1
  fi

  mv "$path.part" "$path"
}
