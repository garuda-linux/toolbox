# Self-test

`scripts/selftest.sh [suite]` (default `all`) builds the app with the
`selftest` feature and runs it on headless weston with the software Vulkan
driver. It prints `SELFTEST PASS` or `FAIL` and writes `result` for
`scripts/selftest-junit.sh`. Shots land in the run's `shots/`, the app's
output in `out.log`.

- Suites live in `app/gui/src/selftest/`, one file per page, listed in
  `SUITES` in `app/gui/src/selftest.rs`.
- The runner (`selftest/runner.rs`) is an egui plugin. After each frame it
  reads the AccessKit tree, before the next it feeds the step's events.
  Steps find controls by accessible name. It waits for animations to
  settle before a shot.

## Rules

- The self-test never changes the machine. The helper runs with
  `GARUDA_TOOLBOX_HELPER=dry-run`, every other effect is a dry-run port
  (see [Dry run](architecture.md#dry-run)).
- Probes read the real system. Where Garuda tools may be missing, a step
  waits for either state (`Step::WaitEither`).
- The script gives the run its own environment:
  - `GARUDA_TOOLBOX_SELFTEST_CONFIGS` points Config Files at a copy of
    `tests/selftest/configs`, plus a link out of the config folders, a
    non-text file and a `.bashrc`
  - its own `settings.toml` with `show_aur = true`, so `packages-aur`
    runs offline against `SampleAur`
  - `--boot-repair` for the `bootrepair` and `all` suites
- `ARCH_SUITES` (`packages-arch`) need pacman's databases. `all` leaves
  them out, `arch-selftest` runs each by name after `all`.
- `scripts/coverage.sh` runs the suites of `COVERAGE_SELFTESTS` (default
  `all`) instrumented.
- Real root actions are tested by `scripts/arch-it.sh` (see
  [toolchain.md](toolchain.md#arch-integration-tests)).
