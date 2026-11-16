#!/usr/bin/env bash
# Sourced from the checkout's root. `lima_image <path>` keeps <path> at the
# newest official Garuda Lima image: it downloads one when the package's
# newest upload differs from the copy there, and checks it against the
# SHA-256 the package lists. <path>.sha256 remembers the copy's sum, so an
# unchanged image is not hashed again.

lima_image_project=88656556
lima_image_api="https://gitlab.com/api/v4/projects/$lima_image_project/packages"
lima_image_url="$lima_image_api/generic/garuda-lima/latest/garuda-lima.qcow2"

lima_image_newest_sha() {
  # `latest` keeps every upload under one name, the newest is the one served.
  local package
  package=$(curl -fsSL "$lima_image_api?package_name=garuda-lima&package_version=latest" |
    jq -r '[.[] | select(.name == "garuda-lima" and .version == "latest")][0].id') || return 1
  curl -fsSL "$lima_image_api/$package/package_files?per_page=100" |
    jq -r '[.[] | select(.file_name == "garuda-lima.qcow2")] | sort_by(.created_at) | last | .file_sha256'
}

lima_image() {
  local path=$1 sha
  sha=$(lima_image_newest_sha) || sha=

  if [ -z "$sha" ] || [ "$sha" = null ]; then
    if [ -f "$path" ]; then
      echo "lima-image: cannot ask for the newest image, using the copy in $path" >&2
      return 0
    fi

    echo "lima-image: the package lists no garuda-lima.qcow2" >&2
    return 1
  fi

  if [ -f "$path" ] && [ "$(cat "$path.sha256" 2>/dev/null)" = "$sha" ]; then
    return 0
  fi

  echo "Downloading the newest Garuda Lima image to $path (about 2 GB)"
  mkdir -p "$(dirname "$path")"
  curl -fL --retry 3 --output "$path.part" "$lima_image_url" || return 1

  if [ "$(sha256sum "$path.part" | cut -d' ' -f1)" != "$sha" ]; then
    rm -f "$path.part"
    echo "lima-image: the download does not match $sha, a newer upload may have come in, try again" >&2
    return 1
  fi

  mv "$path.part" "$path"
  echo "$sha" >"$path.sha256"
}

# `lima_images <script>` prints the `.images` list of a Lima config:
# GARUDA_LIMA_IMAGE, a path or URL with GARUDA_LIMA_IMAGE_DIGEST as its
# optional "sha256:<hex>", else target/lima/garuda-lima.qcow2 kept current.
lima_images() {
  local script=$1 image location
  image=${GARUDA_LIMA_IMAGE:-target/lima/garuda-lima.qcow2}
  if [ -z "${GARUDA_LIMA_IMAGE:-}" ]; then
    lima_image "$image" >&2 || return 1
  fi

  case $image in
  http://* | https://*) location=$image ;;
  *)
    [ -f "$image" ] || {
      echo "$script: no image at $image, set GARUDA_LIMA_IMAGE" >&2
      return 1
    }
    location=$(readlink -f "$image")
    ;;
  esac

  jq -nc --arg l "$location" --arg d "${GARUDA_LIMA_IMAGE_DIGEST:-}" \
    '[{location: $l, arch: "x86_64"} + (if $d == "" then {} else {digest: $d} end)]'
}
