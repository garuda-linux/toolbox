# Toolchain, lints and CI

## Checks

- Nix (`nix/checks.nix`) runs the checks of
  [AGENTS.md](../../AGENTS.md#checks-all-must-pass) up to `cli-docs`, not
  the self-test or the container and VM tests. Nothing built by Nix ships.
  Arch (`ci/arch/`) runs the release build on stable Rust, the MSRV check,
  the self-test, the helper integration tests and the PKGBUILD.
- prek hooks (`nix/git-hooks.nix`) run clippy, file sizes, `toolbox-lint`,
  treefmt, trufflehog and commitizen on every commit.
- The `tests` check adds `cargo test --doc`, since nextest runs no doc
  tests. Advisories need the network, so the GitLab job `advisories` runs
  them on every default-branch and tag pipeline and a pipeline schedule.
  A release needs it to pass. `unmaintained = "all"` in `deny.toml`, so
  an exception is an `ignore` entry with the date and the reason.
- `arch-build` and the PKGBUILD build with `cargo auditable`, so the
  binaries carry their dependency list for `cargo audit bin`.
- The shared CI include is pinned to a commit and the `alpine` and
  `archlinux` images to a digest.
- Renovate merges minor and patch updates by itself, not majors and not
  the Nix lock. A crate release waits 7 days before Renovate offers it.
- A new GitLab check job goes into `.release-needs`, unless its rules
  leave it out of tag pipelines.

## Cargo and lints

- All versions in `[workspace.dependencies]`, metadata in
  `[workspace.package]`.
- One pinned nightly (`nix/rust-toolchain.nix`), only rustfmt uses its
  options. The code also builds on Arch's stable `rust` and on
  `rust-version`.
- clippy's pedantic, nursery and cargo groups are denied, plus picked
  `restriction` lints. Exceptions use `#[expect(lint, reason = …)]`, never
  `#[allow]`. `unsafe` also needs a `// SAFETY:` line above the block.
- `#[must_use]` only where dropping the value is a bug. Typed errors
  everywhere except [these](architecture.md#errors).
- Release profile: fat LTO, `overflow-checks` on, so a wrapped size stops
  the root helper. No `panic = "abort"`: a panic in a service ends that
  task alone.
- `scripts/release.sh` cuts a release, as in the other Garuda repositories.
  `cz bump --files-only` writes the version to `Cargo.toml`,
  `packaging/PKGBUILD` and `nix/package.nix` (`.cz.yaml`), the version and
  date go to the top of `<releases>` in the metainfo, git-cliff prepends
  the release to `CHANGELOG.md` (`cliff.toml`), then one
  `chore(release): <version>` commit and the tag. Prepending keeps earlier
  entries, the Electron app's releases up to 6.3.0 too. A hand-written
  `## Unreleased` section becomes the release's entry instead, as for the
  Rust rewrite (7.0.0).

## Packaging

- `scripts/install.sh` is the only place that knows where files go, for
  both `nix/package.nix` and `packaging/PKGBUILD`.
- The desktop file is named like the binary and the Wayland app_id
  (`APP_ID` in `app/gui/src/main.rs`), so the compositor finds its icon.
  Validate it and the metainfo with `desktop-file-validate` and
  `appstreamcli validate --no-net`.
- In the PKGBUILD only `prepare()` touches the network, `build()` and
  `check()` run `--frozen`. `check()` uses nextest: a plain `cargo test`
  runs a crate's tests as threads of one process, where tests that share
  process-wide state (signal handlers, caches, a folder) fail now and
  then. Tests still stay independent, so both runners pass.
- Every program the app or the helper starts is in `depends` or
  `optdepends`, with a comment naming the reason. The release PKGBUILD in
  `garuda-linux/pkgbuilds` copies these lists, `package()` and `options`.

## Arch integration tests

`scripts/arch-it.sh [cargo test arguments]` runs tests as root in
`ci/arch/Containerfile`, with an offline repository of test packages
(`ci/arch/make-repo.sh`). The container has no systemd, polkit or pkexec.

- Tests that change the system return at once unless `TOOLBOX_IT_ARCH=1`,
  root and `/etc/arch-release` are all given. They run with
  `--test-threads=1`.
- Every new root action adds a test in `app/helper/tests/arch.rs` or
  `app/helper/tests/arch/`. Account tests use `toolbox-test-user-*` and
  `toolbox-test-grp-*`.

## Lima integration tests

`scripts/lima-it.sh [--keep] [--no-build] [--suite LIST]` covers what the
container cannot run: systemd units, locales, DKMS and NetworkManager. It
builds the package with `scripts/arch-package.sh --nocheck`, or with
`--no-build` takes the package of the `Cargo.toml` version from
`target/arch-package/`. It starts a Garuda Lima VM
(`tests/lima/garuda.yaml`), installs the package with dkms,
v4l2loopback-dkms and networkmanager, and drives the CLI with `--yes`. At
the end it restarts the VM once and checks that the changes survive the
boot.

- `GARUDA_LIMA_IMAGE` is the qcow2, a path or a URL (default
  `target/lima/garuda-lima.qcow2`). `GARUDA_LIMA_IMAGE_DIGEST` takes
  `sha256:<hex>` for a URL. Lima downloads and caches it.
- Without `GARUDA_LIMA_IMAGE` and without the default file,
  `scripts/lima-image.sh` downloads the official image, the package
  `garuda-lima` version `latest` of image-builder (GitLab project
  88656556), and checks it against the SHA-256 of the newest upload the
  package lists. A run on another day may test a newer image.
- The image boots with systemd-boot and UKIs, ESP at `/efi`. The Boot
  page, Boot Repair, `kernels default` and Plymouth have no checks here.
  The first three need GRUB, and on systemd-boot an initramfs rebuild does
  not reach the UKI.
- The CLI runs on a terminal through `script(1)`, each call limited to
  10 minutes. A polkit rule lets the Lima user run the helper without a
  password: over SSH no agent runs, and pkexec would ask on the terminal
  for a password the user does not have.
- Needs KVM, the network and `limactl` and `jq` on the host. No CI runner
  has them, so it runs by hand before every release, and
  `scripts/release.sh` asks for it.
- Logs per check land in a new `$TMPDIR/garuda-toolbox-it.XXXXXX/` per
  run, or in `$LIMA_IT_ROOT/garuda-toolbox-it/` when that is set.
- Checks live in `scripts/lima-it/checks.sh`.

### Suites

`--suite basic,upgrade,faults,accounts,sdboot` picks the suites, `basic` (the
checks above) by default. One file per suite in `scripts/lima-it/`
defines `suite_<name>`. After the setup a `limactl snapshot` (tag `base`)
is taken, and every suite after the first, and every part of a suite that
breaks the system, starts from it again.

| Suite      | What it checks                                                                                            |
| ---------- | --------------------------------------------------------------------------------------------------------- |
| `upgrade`  | `update`, a package in and out, maintenance tasks, `linux-lts` installed and removed, on the real mirrors |
| `faults`   | Cuts the power while pacman runs, then blocks the network for an install                                  |
| `accounts` | Adds, changes, locks and removes an account, each proven by `su` on a terminal                            |
| `sdboot`   | `loader.conf` settings, `kernels default` and a new kernel parameter, then a restart into the rebuilt UKI |

- A snapshot of the running VM takes a second, after a `sync` so the
  setup is on the disk. `restore_base` stops the VM, applies the snapshot
  and boots it, because memory loaded into a QEMU started after the
  snapshot lost SSH or name lookups. It waits until `archlinux.org`
  resolves.
- Lima's SSH master socket dies with a restore or a forced stop, and every
  connection through it hangs until it times out, so `wait_ssh` deletes it
  first.
- A stopped QEMU holds its disk a moment longer, so `wait_qemu_gone` waits
  for it before the next start.
- Never take or apply a snapshot while a check runs in the VM: the
  restore rolls the check's system back under it.
- A secret goes in on standard input through `tp`, without `script(1)`,
  which would echo it into the log.

## Dev loop

`scripts/dev.sh` (`start-dev` in the dev shell) restarts the app on
changes. Debug builds read catalogs from the checkout. On NixOS the
package, pacman and Garuda probes report "not available": test real
behaviour on Arch.
