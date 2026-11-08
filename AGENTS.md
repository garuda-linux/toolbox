# Garuda Toolbox notes for agents

Rust desktop app and CLI (egui and eframe 0.37 on wgpu, tokio, Rust
edition 2024) that maintains and configures Garuda Linux and other Arch
based systems ([constraints](docs/agents/architecture.md#constraints)).
`reference/` (git-ignored) holds the Electron app this one replaces. Read
it to learn what a feature does. Never edit or copy it.

## Topic notes

Open the matching note before you work in its area:

| Note                                           | Before you touch                                                         |
| ---------------------------------------------- | ------------------------------------------------------------------------ |
| [architecture.md](docs/agents/architecture.md) | a crate, page, service, catalog file or root action                      |
| [security.md](docs/agents/security.md)         | `app/helper`, `crates/plan`, polkit, pkexec, sudo, the CLI, outside text |
| [ui-rules.md](docs/agents/ui-rules.md)         | a page, a gui-kit widget, CLI output                                     |
| [toolchain.md](docs/agents/toolchain.md)       | `Cargo.toml`, lints, flake checks, CI, packaging, the dev loop           |
| [logging.md](docs/agents/logging.md)           | a log call, a span or a Console message                                  |
| [selftest.md](docs/agents/selftest.md)         | the self-test                                                            |
| [docs.md](docs/agents/docs.md)                 | the user guide in `docs/src/`                                            |
| [icons.md](docs/agents/icons.md)               | an icon of a page, heading, row or catalog section                       |
| [translations.md](docs/agents/translations.md) | Transifex, `i18n/`, `transifex.yml`, the i18n CI jobs                    |

## Layout and adding things

Libraries are `toolbox-<x>` in `crates/<x>`, binaries in `app/` (`gui`,
`helper`, `xtask`, `lint`). Dependencies point inward to `toolbox-plan`
and `toolbox-state` ([crates](docs/agents/architecture.md#crates)).

- A toggle, app, game or file: an entry in `data/catalog/`, then
  `cargo xtask catalog-validate` and `cargo xtask catalog-ftl`. A
  root-owned configuration file also needs its line in
  `crates/plan/src/config_file/writable.rs`.
- A kind of queued change: a `toolbox_core::Domain`, its probe, its
  line in `domains::all()` and its CLI command
  ([Extending](docs/agents/architecture.md#extending)). Never edit
  `ChangeSet`, `Snapshot` or the planner for it.
- A root action: a `RootAction` variant with value objects only, its
  validation and argv test in `plan`, its executor in `helper`, its Arch
  test ([security.md](docs/agents/security.md)).
- A page: the checklist [A page](docs/agents/architecture.md#a-page).
- An installed file: its line in `scripts/install.sh`.

## Checks (all must pass)

```sh
nix develop            # toolchain, weston, cargo tools, prek hooks
cargo clippy --workspace --all-targets --features toolbox-gui/testing
cargo nextest run --workspace --features toolbox-gui/testing
nix fmt                # treefmt
scripts/lint.sh        # toolbox-lint: comments and shape
scripts/check-file-size.sh
cargo xtask catalog-validate
cargo xtask catalog-ftl --check
cargo xtask i18n-export --check && cargo xtask i18n-import --check
cargo xtask cli-docs --check
scripts/selftest.sh [suite]   # headless weston, helper in dry-run mode
scripts/arch-it.sh            # helper as root against real pacman in the Arch container
scripts/arch-package.sh       # makepkg and namcap in the Arch container
scripts/lima-it.sh [--suite LIST]   # systemd, locales, DKMS, upgrades and faults in a Garuda VM
```

## Done means

- Test first: a failing test, the simplest code that passes, then
  refactor. Logic lives in `core`, `plan`, `system` and `catalog`, tested
  without egui. Ports get fakes, not mocks.
- A user-visible feature has self-test steps (happy path plus one edge
  case), kittest tests, a page in `docs/src/` and CLI commands. It works
  with the keyboard alone and passes one Orca pass.
- UI work is done only after opening the kittest snapshots and the
  self-test shots of every changed page: no hidden control, clipped or
  overlapping text, unexpected banner, empty or wrong page.

## Rules for every change

- `.rs` files in `app/` and `crates/` stay at most 400 lines.
  `.file-size-allow` may only shrink. Tests live in `foo/tests.rs`.
- Comments state why, never what, and only where the code cannot. No
  semicolons in prose. A real exception to `scripts/lint.sh` gets
  `// toolbox-lint: allow(<rule>) <reason>` above the line.
- Domain values are `toolbox-plan` value objects, never bare `String`s.
- Root only through `garuda-toolbox-helper` or a D-Bus service that checks
  polkit (systemd1, login1). Argv only, no shell strings. No `sudo` except
  the [CLI fallback](docs/agents/security.md#sudo-on-the-command-line).
  `paru` runs as the user.
- Every visible string is a Fluent text in `app/gui/i18n/en-US` with its
  German counterpart, looked up by id.
- No `std::sync` lock across `.await`, blocking work in `spawn_blocking`,
  the egui frame never blocks.
- New files must be `git add`ed before `nix build` sees them.
