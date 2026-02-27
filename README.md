# Garuda Linux's Toolbox

[![pipeline status](https://gitlab.com/garuda-linux/applications/toolbox/badges/main/pipeline.svg)](https://gitlab.com/garuda-linux/applications/toolbox/-/pipelines)
[![Commitizen friendly](https://img.shields.io/badge/commitizen-friendly-brightgreen.svg)](http://commitizen.github.io/cz-cli/)
[![Latest Release](https://gitlab.com/garuda-linux/applications/toolbox/-/badges/release.svg)](https://gitlab.com/garuda-linux/applications/toolbox/-/releases)

## Dependencies

- `pacman-contrib`: Required for the `checkupdates` command and some other functionalities.
- `garuda-libs`: Required for functionality like opening a terminal based on desktop environment.
- `garuda-update`: Used for updating the system.
- `paru`: Required for the AUR functionalities.
- `electron36` and all of its deps (version might change, check `package.json`)

## Found any issue?

- If any packaging issues occur, don't hesitate to report them via our issues section of our PKGBUILD repo. You can
  click [here](https://gitlab.com/garuda-linux/pkgbuilds/-/issues/new) to create a new one.
- If issues concerning the configurations and settings occur, please open a new issue on this repository.
  Click [here](https://gitlab.com/garuda-linux/applications/toolbox/-/issues/new) to start the process.

## How to contribute?

We highly appreciate contributions of any sort! 😊 To do so, please follow these steps:

- [Create a fork of this repository](https://gitlab.com/garuda-linux/applications/toolbox/-/forks/new).
- Clone your fork locally ([short git tutorial](https://rogerdudler.github.io/git-guide/)).
- Add the desired changes to PKGBUILDs or source code.
- Commit using a [conventional commit message](https://www.conventionalcommits.org/en/v1.0.0/#summary) and push any
  changes back to your fork. This is crucial as it allows our CI to generate changelogs easily.
  - The [commitizen](https://github.com/commitizen-tools/commitizen) application helps with creating a fitting commit
    message.
  - You can install it via [pip](https://pip.pypa.io/) as there is currently no package in Arch repos:
    `pip install --user -U Commitizen`.
  - Then proceed by running `cz commit` in the cloned folder.
- [Create a new merge request at our main repository](https://gitlab.com/garuda-linux/applications/toolbox/-/merge_requests/new).
- Check if any of the pipeline runs fail and apply eventual suggestions.

We will then review the changes and eventually merge them.

## Development setup

To set up a development environment, all that is needed is Node.js and Chromium dependencies.
The installation of Chromium or system-wide Electron is the easiest way to get started.

### NixOS

For NixOS, the best thing to do is putting all the libraries appimage-run provides into nix-ld.
Otherwise, `electron` from node_modules won't be able to start. Everything else is just super tedious.
It is recommended to develop inside of a Archlinux Distrobox instance, so all the required commands are available.

## Where is the PKGBUILD?

The PKGBUILD can be found in our [PKGBUILDs](https://gitlab.com/garuda-linux/pkgbuilds) repository. Accordingly,
packaging changes need to be happening over there.

## How to deploy a new version?

To deploy a new version, pushing a new tag is sufficient. The deployment will happen automatically via
the [PKGBUILDs repo's pipelines](https://gitlab.com/garuda-linux/pkgbuilds/-/pipelines), which check half-hourly for the
existance of a more recent tag.
