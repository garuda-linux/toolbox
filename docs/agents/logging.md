# Logging

## Facade and sinks

- Every crate logs with `tracing` macros, default target. The root helper
  keeps a `log` stderr logger. Tracing's `log` feature lets shared crates
  reach it (`app/helper/tests/log_fallback.rs`). The app forwards the
  helper's stderr into its log as `helper:`.
- The app installs `toolbox_log::subscriber`, with `LogTracer` for the
  `log` records of egui, wgpu and other dependencies.
- The full log always goes to the rotating file
  (`crates/services/src/log_folder.rs`) at the configured level.
- A CLI command writes no log line to the terminal
  (`toolbox_log::quiet_terminal`) and states problems as its own `error:`
  and `warning:` lines. Only the window also logs to the terminal.

## Levels

- Error: a user-requested operation failed.
- Warn: fallback, degraded or swallowed error.
- Info (default): one line when a user action starts or finishes, a
  setting changes or the snapshot reloads. Never one per item, step or
  probe.
- Debug: branches taken, paths, counts, timings.
- Trace: profiler spans and per-row detail.
- `[general] log_level` in `settings.toml` sets the workspace crates.
  Dependencies stay at warn, at info when the level is Trace.
  `GARUDA_TOOLBOX_LOG` (`EnvFilter` syntax) replaces the filter.

## Spans

- `apply{steps}` (info) around a plan, `step{index program}` (debug) per
  step: the program name, never argv.
- `read{what}` (debug) around background reads, `command{command from}`
  (debug) around window commands. `Command::for_log` hides Console input.
- Never hold an entered span across `.await`, use `Instrument`.
- `--features profiling` on `toolbox-gui` sends trace spans to Tracy on
  localhost, one frame per egui frame.

## Rules

- A failure shown in the Console is logged once, by the service that
  reports it.
- A program is logged once where it starts (argv, exit status, duration).
  Expected probe failures log at debug.
- Never log secrets, whole files from `/etc`, paste links or the public
  address. Console input is logged by its length.
