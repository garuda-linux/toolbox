# Toolchain, lints and CI

## Checks

- Nix (`nix/checks.nix`) runs the checks of
  [AGENTS.md](../../AGENTS.md#checks-all-must-pass) and the self-test.
  Nothing built by Nix ships. Arch (`ci/arch/`) runs the release build on
  stable Rust, the MSRV check, the self-test, the helper integration tests
  and the PKGBUILD.
- prek hooks (`nix/git-hooks.nix`) run clippy, file sizes, `toolbox-lint`,
  treefmt, trufflehog and commitizen on every commit.
- The `tests` check adds `cargo test --doc`, since nextest runs no doc
  tests. Advisories need the network, so the GitLab job `advisories` runs
  them on `Cargo.lock` changes and a pipeline schedule.
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
  `packaging/PKGBUILD` and `nix/package.nix` (`.cz.yaml`), git-cliff
  prepends the release to `CHANGELOG.md` (`cliff.toml`), then one
  `chore(release): <version>` commit and the tag. Prepending keeps earlier
  entries, so hand edits stay. The first release (no tag yet) uses the
  hand-written section as its entry. `CHANGELOG.md` is not committed before
  0.1.0.

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

## Dev loop

`scripts/dev.sh` (`start-dev` in the dev shell) restarts the app on
changes. Debug builds read catalogs from the checkout. On NixOS the
package, pacman and Garuda probes report "not available": test real
behaviour on Arch.
