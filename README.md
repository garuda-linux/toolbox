# Garuda Linux's Toolbox

[![pipeline status](https://gitlab.com/garuda-linux/applications/toolbox/badges/main/pipeline.svg)](https://gitlab.com/garuda-linux/applications/toolbox/-/pipelines)
[![Commitizen friendly](https://img.shields.io/badge/commitizen-friendly-brightgreen.svg)](http://commitizen.github.io/cz-cli/)
[![Latest Release](https://gitlab.com/garuda-linux/applications/toolbox/-/badges/release.svg)](https://gitlab.com/garuda-linux/applications/toolbox/-/releases)

## Found any issue?

- If any packaging issues occur, don't hesitate to report them via our issues section of our PKGBUILD repo. You can
  click [here](https://gitlab.com/garuda-linux/pkgbuilds/-/issues/new) to create a new one.
- If issues concerning the configurations and settings occur, please open a new issue on this repository.
  Click [here](https://gitlab.com/garuda-linux/applications/toolbox/-/issues/new) to start the process. The log file
  `~/.local/state/garuda-toolbox/garuda-toolbox.log` helps a lot.
- Security problems go through [`SECURITY.md`](SECURITY.md).

## How to contribute?

We highly appreciate contributions of any sort! 😊 To do so, please follow these steps:

- [Create a fork of this repository](https://gitlab.com/garuda-linux/applications/toolbox/-/forks/new).
- Clone your fork locally ([short git tutorial](https://rogerdudler.github.io/git-guide/)).
- Add the desired changes to PKGBUILDs or source code. [`AGENTS.md`](AGENTS.md) describes the layout and the rules.
- Commit using a [conventional commit message](https://www.conventionalcommits.org/en/v1.0.0/#summary) and push any
  changes back to your fork. This is crucial as it allows our CI to generate changelogs easily.
  - The [commitizen](https://github.com/commitizen-tools/commitizen) application helps with creating a fitting commit
    message.
  - It comes with the development shell (`nix develop`), and a commit hook checks every message with it.
  - Then proceed by running `cz commit` in the cloned folder.
- [Create a new merge request at our main repository](https://gitlab.com/garuda-linux/applications/toolbox/-/merge_requests/new).
- Check if any of the pipeline runs fail and apply eventual suggestions.

We will then review the changes and eventually merge them.

## Where is the PKGBUILD?

The PKGBUILD can be found in our [PKGBUILDs](https://gitlab.com/garuda-linux/pkgbuilds) repository. Accordingly,
packaging changes need to be happening over there. [`packaging/PKGBUILD`](packaging/PKGBUILD) in this repository only
builds a test package of the checkout.

## How to deploy a new version?

Run `scripts/release.sh` on `main`. It sets the version, writes the changelog with git-cliff and tags the release.
Pushing the new tag is sufficient. The deployment will happen automatically via the
[PKGBUILDs repo's pipelines](https://gitlab.com/garuda-linux/pkgbuilds/-/pipelines), which check half-hourly for the
existence of a more recent tag.

## About the app

Garuda Toolbox is a desktop app that maintains and configures Garuda Linux and other Arch based systems. It is written
in Rust with an [egui](https://github.com/emilk/egui) interface.

Changes are collected first, shown for review and then applied together with one password prompt. Everything that
needs root rights runs through one small helper (`garuda-toolbox-helper`) started with `pkexec`. The same binary is
also a command line tool that can do everything the window can.

### Pages

| Page             | What it does                                                                                      |
| ---------------- | ------------------------------------------------------------------------------------------------- |
| Home             | System status (updates, pacdiff files, pending reboot, health), each figure leading to its row    |
| Setup Assistant  | A wizard over `data/catalog/setup.toml` for a fresh installation                                  |
| Components       | Optional system parts from `data/catalog/components.toml`                                         |
| Gaming           | Launchers, tools and games from `data/catalog/gaming.toml`                                        |
| App Catalog      | Applications by category from `data/catalog/apps/`                                                |
| Services         | systemd units of the system and of the user                                                       |
| System Tweaks    | Login shell, DNS, hblock, Wi-Fi backend, garuda-update flags and toggles from the catalog         |
| Kernels          | Kernel packages with their headers and DKMS modules                                               |
| Languages        | Enabled locales, the system language and language packs                                           |
| Users and Groups | Accounts, memberships, shells, locks and passwords                                                |
| Config Files     | Config files from `data/catalog/config-files.toml`, edited as a copy and written back with a diff |
| Network          | Connection status, drivers, hardware and tools (ping, traceroute)                                 |
| Boot Options     | GRUB defaults and the Plymouth theme                                                              |
| Maintenance      | System update, mirrors, keyring, caches, orphans, reset of user configuration files, repairs      |
| Boot Repair      | Reinstalls GRUB from the live ISO                                                                 |
| Diagnostics      | Reports to copy or to upload to `bin.garudalinux.org`                                             |
| Settings, About  | Language, theme, motion, update checks, log detail, and the links to Garuda Linux's services      |

The changes pill at the bottom right lists the queued changes in plain words, each one removable, and applies them
together. The Console shows the running apply as a terminal that takes keyboard input. A command palette (`Ctrl+K`)
finds pages, commands, Maintenance operations and apps. A second start brings the running window to the front instead
of opening another one.

### Command line

`garuda-toolbox` without a subcommand opens the window. With one it runs in the terminal and never opens a window:

```console
$ garuda-toolbox status --json
$ garuda-toolbox services restart sshd.service
$ garuda-toolbox toggle on pipewire-support
$ garuda-toolbox --page maintenance
```

Commands that change the system show the plan and ask first (`--yes` skips the question). The guide's
[command line chapter](docs/src/cli.md) lists every command, its exit codes and its JSON output. Shell completions and
a man page are installed with the app.

## Development setup

The target is Arch Linux based distributions only. The development shell comes from the Nix flake and brings the Rust
toolchain and every tool the checks need:

```console
$ nix develop
$ cargo build
$ cargo run --bin garuda-toolbox
```

`scripts/dev.sh [app arguments]` rebuilds and restarts the app while you edit (`start-dev` inside the dev shell).
Catalogs are read from the checkout in debug builds.

On a machine without pacman (NixOS, for example) the app starts and shows its pages, but the package, pacman and Garuda
probes report "not available". Test real behaviour in an Arch container or VM:

- `ci/arch/install-deps.sh` installs the build dependencies on Arch, then `cargo build --release --locked` builds
  against Arch's libraries.
- `scripts/arch-package.sh` builds the test package from [`packaging/PKGBUILD`](packaging/PKGBUILD) in the Arch test
  image.

The checks that must pass are listed in [`AGENTS.md`](AGENTS.md#checks-all-must-pass). CI runs them through
`nix flake check` and the GitLab jobs in [`.gitlab-ci.yml`](.gitlab-ci.yml).

### Documentation

- The user guide is an mdBook in [`docs/src/`](docs/src/SUMMARY.md). `scripts/build-docs.sh` builds it into
  `docs/book`.
- Notes for contributors and coding agents are in [`AGENTS.md`](AGENTS.md) and [`docs/agents/`](docs/agents).

### Translations

The app's texts are [Fluent](https://projectfluent.org/) files in [`app/gui/i18n/`](app/gui/i18n), one folder per
language. Translators work on [Transifex](https://www.transifex.com/), see
[`docs/agents/translations.md`](docs/agents/translations.md).

## License

Garuda Toolbox is free software, licensed under the [GNU General Public License v3.0 or later](LICENSE).
