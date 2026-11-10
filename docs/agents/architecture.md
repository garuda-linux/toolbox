# Architecture

## Constraints

- Arch Linux based distributions only. No `cfg(target_os)`.
- Any desktop. Colours, accent, fonts and icon theme come from the
  freedesktop portal and files (`app/gui/src/desktop.rs`), never from a
  desktop's own libraries. A KDE-only feature needs a desktop-neutral path
  or stays hidden elsewhere.
- Keyboard, screen readers, scaling and contrast work everywhere.
- A new toggle, app or configuration file is a catalog entry. A new page is one
  feature module, one `PageView` and one `PageId` (see [A page](#a-page)).

## Stack

- UI: one `App` (`app/gui/src/app.rs`) owns the state. Pages implement
  `PageView` (`app/gui/src/pages.rs`) and never change shared state. They
  send commands on the bus (`app/gui/src/command.rs`), and `App::logic`
  applies them before the next frame. The design system is
  `crates/gui-kit`. The window is opaque, blur stays inside it
  ([Frost](#frost)).
- Console: a full terminal, so full-screen programs such as paru's
  editor work. A plain-text transcript beside it serves screen readers,
  copy and share. Output is never clickable. A queue is applied as a whole.
- Reading state: libraries and D-Bus first (`alpm`, systemd1 and login1
  over zbus), plain files for locales and kernels. `dkms status` is the one
  parsed program output.
- Translations: the core sends codes and data, never text. See
  [translations.md](translations.md).
- Packages: `PackageSource` (local) and `SyncSource` (the sync
  databases `/etc/pacman.conf` names) are read only, never synced. alpm's
  build script refuses another libalpm major, so bump `alpm` when Arch
  moves to one.
- AUR: read from a trimmed copy of the daily metadata dump (see
  [security.md](security.md#aur-metadata)). A build is the `aur-packages`
  domain (paru as the user), a removal the `packages` domain's
  `pacman -Rns`.

## Names

| Thing          | Name                                                                                                    |
| -------------- | ------------------------------------------------------------------------------------------------------- |
| App            | Garuda Toolbox, binary and Wayland app_id `garuda-toolbox`                                              |
| Root helper    | `/usr/lib/garuda-toolbox/garuda-toolbox-helper` (off `PATH`)                                            |
| Polkit actions | `org.garudalinux.toolbox.{apply,read,logs,fetch}`                                                       |
| D-Bus name     | `org.garudalinux.toolbox` (session bus, `org.freedesktop.Application`, `org.garudalinux.toolbox.Queue`) |
| Desktop file   | `garuda-toolbox.desktop` (autostart links point to it)                                                  |
| XDG dirs       | `garuda-toolbox` (config, state, cache)                                                                 |
| Environment    | `GARUDA_TOOLBOX_*`                                                                                      |
| Installed data | `/usr/share/garuda-toolbox/catalog`                                                                     |

## Crates

```
toolbox-gui (bin garuda-toolbox)
  ├─ toolbox-commands (the CLI) ─ toolbox-services, toolbox-cli, toolbox-guide, toolbox-aur
  ├─ toolbox-services ─ toolbox-core, toolbox-system, toolbox-exec, ...
  ├─ toolbox-core ─ toolbox-catalog ─ toolbox-state ─ toolbox-plan
  ├─ toolbox-system ─ toolbox-exec ─ toolbox-pty, toolbox-fs
  ├─ toolbox-gui-kit ─ toolbox-terminal ─ toolbox-pty
  ├─ toolbox-privatebin, toolbox-news, toolbox-aur ─ toolbox-https
  └─ toolbox-config, toolbox-log, toolbox-instance, toolbox-cli
toolbox-helper (bin, root) ─ toolbox-plan, toolbox-exec
dev only: toolbox-lint, toolbox-xtask
```

Link rules (check with `cargo tree -p <crate> --edges normal`):

- Only `toolbox-gui` and `toolbox-gui-kit` link egui, so
  `cargo nextest run -p toolbox-commands` builds without it.
- `toolbox-helper` links `toolbox-plan` and `toolbox-exec` only.
- `toolbox-pty`, `toolbox-fs` and `toolbox-https` link nothing of the
  workspace. Every outside request goes through `toolbox-https`, so the
  HTTPS rules live in one place.

| Crate                | Holds                                                                          | Must not                                                   |
| -------------------- | ------------------------------------------------------------------------------ | ---------------------------------------------------------- |
| `toolbox-plan`       | value objects, `RootAction`, `UserAction`, `Plan`, helper protocol, validation | do I/O                                                     |
| `toolbox-state`      | snapshot parts, lsblk and dkms parsing, reports                                | do I/O, be linked by the helper                            |
| `toolbox-pty`        | programs in a PTY, process group cancel, `strip_ansi`, `styled_lines`          | know actions or egui                                       |
| `toolbox-terminal`   | the Console's emulator: bytes in, styled rows and cursor out, keys to bytes    | know egui or the helper                                    |
| `toolbox-guide`      | `docs/src` built in by `build.rs`, search, link and picture tests              | link egui, read files at run time                          |
| `toolbox-exec`       | plan steps in PTYs, `capture`, `secure` (`openat2`), `preflight`, builtins     | link a derive crate                                        |
| `toolbox-fs`         | atomic replace (`replace_at`, `replace`)                                       | link anything but rustix                                   |
| `toolbox-helper`     | `apply`, `read <kind>`, `logs <kind>`, `fetch <path>`                          | run free-form commands                                     |
| `toolbox-system`     | ports to the running system as traits plus adapters, status probes             | decide, hold UI state                                      |
| `toolbox-catalog`    | TOML schemas, loader and validation for `data/catalog/`                        | probe the system                                           |
| `toolbox-core`       | `ChangeSet`, `Snapshot`, the planner, `features/<name>`                        | do I/O                                                     |
| `toolbox-config`     | settings file, XDG paths, autostart link                                       |                                                            |
| `toolbox-log`        | rotating log file, see [logging.md](logging.md)                                | look up XDG folders                                        |
| `toolbox-https`      | the one HTTPS agent (`Https`, `Rules` per caller, `Clock`)                     | follow a redirect, read a body past its cap, know a server |
| `toolbox-privatebin` | PrivateBin v2 paste format and upload                                          | take a server from input, log a link                       |
| `toolbox-aur`        | the AUR from its daily dump only (`Cache`, `parse`)                            | call the RPC, build a URL from input, ask twice a day      |
| `toolbox-news`       | Garuda's and Arch's RSS feeds (`Cache`, `Feeds`)                               | build a URL from input, ask more than every six hours      |
| `toolbox-instance`   | single instance: `activation`, `bus`                                           | ask more than window, page and queue, decode a queue edit  |
| `toolbox-cli`        | clap arguments, exit codes, `output::Paint`, `Prompt`                          | run a command or touch the system                          |
| `toolbox-gui-kit`    | tokens, theme, icons, fonts, widgets, toasts, `terminal_view`, `frost`         | pick a colour outside the tokens                           |
| `toolbox-services`   | what the window and the CLI share (`apply`, `queue`, `page`, `i18n`, ...)      | link egui, `toolbox-gui-kit` or a root helper crate        |
| `toolbox-gui`        | `app/*`, `shell/*`, `pages/*`, GUI services, `ports.rs`, `selftest/*`          | hold feature logic                                         |
| `toolbox-commands`   | every subcommand, its output, Ctrl+C handling                                  | link egui or `toolbox-gui-kit`                             |
| `toolbox-lint`       | comment and shape rules for Rust                                               | ship                                                       |
| `toolbox-xtask`      | the `cargo xtask` commands                                                     | ship                                                       |

### Frost

Menus, lists, toasts, the Changes pill and popover, the status note,
dialogs and the Console show a live blur of what lies under them plus a
tint. The header band shows the blur under a neutral scrim.

- `frost::install` turns it on. Without it, or when the GPU refuses a
  pipeline, a surface shows its tint.
- `frost::reserve` takes a slot, filled at `ctx.on_end_pass`.
  `live/gather.rs` leaves text out, as atlas sampling is unstable at a
  quarter scale. A result is kept once the same shapes rendered twice.
- Every tint and scrim is as thin as its texts keep 4.5:1 over the app's
  own surfaces. `header_scrim` and `terminal_glass` are neutral black or
  white, so nothing under them casts a tint, and never below `SCRIM_LEAST`
  (`tokens/frost.rs`, `tokens/terminal.rs`). Without the blur the band is
  the opaque `window` colour and the terminal box the solid `terminal`.
- The band uses `frost::backdrop_within`, which leaves out shapes outside
  it and shadows, so the sidebar and controls below do not smear in.
- `terminal_edge` adds a hairline, as thin white glass on the light sheet
  has no edge.
- `terminal_ansi` moves ANSI 1 to 6 and 9 to 14 in OKLCH lightness only,
  until they reach 4.5:1 on the solid `terminal` and the thinnest glass.
- `cover_page_bottom` ends the scroll area at the Console's top while the
  page paints on under the sheet, so the blur shows the page.
- `sidebar`, `pane`, the status bar and the Console sheet are `window` with
  `header_scrim` on it, opaque, so every text that reads on `window` reads
  there (`tokens/tests/sidebar.rs`).

## Errors

A function that can fail returns its own error type, derived with
`thiserror`, the helper's crates included. A wrapped error stays its
`#[source]`, behind an `Arc` where the type must stay `Clone`. `Result<_, String>`
only in `toolbox-xtask`, `toolbox-lint` and clap value parsers
(`crates/cli/src/args*`, `crates/commands/src/settings.rs`), because clap
prints the `String` as the usage error.

## Concurrency

```
 eframe main thread                    tokio runtime
 widgets ─Command─► bus ─► App::logic ─► reader / units / apply / diagnostics / desktop
                              ▲                         │
                              └── channel + request_repaint ◄┘
```

- `main` builds a multi-thread tokio runtime, claims the single instance,
  then runs eframe on the main thread. A subcommand never starts eframe.
- Slow reads run through `reader` on blocking threads. Requests during a
  read become one more read. The snapshot reads at start, after every plan
  and when the window comes to the front. Page reads go through a `Lane`:
  only the newest answer of each kind lands.
- `apply` runs root segments through the helper, user segments in the app.
- Every background result wakes the frame with `ctx.request_repaint()`.
- A panic in a task reaches the log and the crash file (`panics.rs`).
- Tests: pages in `egui_kittest` with `PageFixture` (`testing.rs`), timers
  with tokio's paused time.

## Extending

| To add                  | Implement                                                                                                                                                                                                       | Untouched                        |
| ----------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------- |
| A kind of queued change | a `toolbox_core::Domain` (`Item`, `Wish`, `State`, `ID`, `STAGE`, `is_satisfied`, `plan`, `key`, `summary`), its `change-<code>` text in `changes.ftl`, its `.with::<D>()` in `domains::all()`, its CLI command | `ChangeSet`, `Snapshot`, planner |
| Its probe               | a function that `Snapshot::insert`s the domain's `State`, a `Probe` variant, `probe-<id>` in `status.ftl`                                                                                                       | the snapshot service             |
| A root step             | a `RootAction` variant (see [security.md](security.md))                                                                                                                                                         | apply service, Console           |
| A catalog entry         | TOML only                                                                                                                                                                                                       | everything                       |
| A page                  | [A page](#a-page)                                                                                                                                                                                               | shell, palette                   |

### A page

1. `crates/core/src/features/<name>`: pure logic with tests.
2. `crates/services/src/page.rs`: a `PageId` variant with its `slug`
   (`--page`, palette, D-Bus activation) and sidebar `Group`. Icon and hue
   in `app/gui/src/page_ui.rs` ([icons.md](icons.md)).
3. `app/gui/src/pages/<name>.rs`: a `PageView`, tests in
   `pages/<name>/tests.rs`.
4. `app/gui/src/pages.rs`: its field and its line in `Pages::view`.
   `PageId::ALL` drives the sidebar, the palette and the start options.
5. A service in `app/gui/src/<name>.rs` for reads outside the snapshot, and
   its dry-run stand-in in `ports.rs` when the read changes or leaves the
   machine. It implements `PageService` (`app/gui/src/page_service.rs`) and
   answers with `AppCommand`s. Its field in `Services`, its line in
   `Services::all_mut` and its `Provides` impl are all `App` needs. The
   page reads it with `context.service::<Name>()`.
6. Texts in `crates/services/i18n/en-US/<name>.ftl` and `de-DE/<name>.ftl`.
7. CLI commands (`crates/cli/src/args/*`, `crates/commands/src/*`).
   `crates/commands/src/parity.rs` fails for a queue domain without one.
8. Integration tests and snapshots (`app/gui/tests/integration/<name>.rs`)
   in dark and light.
9. Self-test steps in `app/gui/src/selftest/<name>.rs`, listed in `SUITES`.
10. A page under `docs/src/` and its line in `SUMMARY.md`.

### Dry run

`GARUDA_TOOLBOX_HELPER=dry-run` makes the helper only check plans. Every
other effect is a port that `app/gui/src/ports.rs` picks at start
(`DryRunTransport`, `DryRunAddress`, `DryRunActions`, `DryRunLauncher`).
Pages never see the mode. The apply service skips user steps.

Sample data never ships. `DryRunRepair`, `SampleMirrors`, `SampleAur` and
`SampleNews` exist only under `cfg(any(test, feature = "selftest"))`
(`toolbox-gui` also under `testing`), picked in `app/gui/src/ports/samples.rs`.
A release build's dry run reads the real disks, mirrors, AUR and news
instead. The `selftest` feature of `toolbox-gui` turns on the one of
`toolbox-services`, `toolbox-system` and `toolbox-state`.

`GARUDA_TOOLBOX_CATALOG_DIR` and `GARUDA_TOOLBOX_SELFTEST_CONFIGS` count
only in a development build, debug or `selftest`
(`toolbox_services::catalogs::DEVELOPMENT`). Without the catalog variable
such a build reads `data/catalog` of the source tree. A release reads
only `/usr/share/garuda-toolbox/catalog` and never the source tree.

### The saved queue

- `$XDG_STATE_HOME/garuda-toolbox/changes.json` is written atomically on
  every change and restored by `domains::all()`. Unknown domains and
  invalid entries are dropped with a log line. A changed shape bumps
  `SAVED_VERSION`, and a file of another version is dropped whole.
- While a window holds the bus name, `garuda-toolbox queue` never writes
  the file. It sends a `QueueEdit` over `org.garudalinux.toolbox.Queue`,
  and the window runs every guard again (`Queue::answer`). The window
  refuses an edit while its Console runs a plan. `queue apply` runs only
  without a window, so a change never runs twice.
- The item is the key of a change, the wish its value. A second wish
  replaces the first. Work to do has the wish `Queued`. `ChangeSet::want`
  ignores a wish that `Domain::fits` refuses.
- A restored queue was queued before the system was known.
  `features::Unchecked` runs each guard of `features::GUARDS` once over it,
  as soon as the snapshot holds the part the guard needs. A page that
  queues changes adds its guard, the same function the page calls.
- A guard that waits fails closed: `Queue::plan` refuses with
  `Refusal::GuardsWaiting` while `Unchecked::holding` names a guard with
  a queued change of one of its `Guard::domains`. Apply stays disabled
  and says why (`changes-wait-*`).
- No probe reads `AccountLockFlags` or the AUR names. The app adds them
  to every snapshot after the probe (`combine` in
  `app/gui/src/app/services.rs`) and asks the Users or Packages read
  itself once a held guard waits for them (`read_held_parts`).
  `queue apply` reads them the same way (`crates/commands/src/queue/ready.rs`)
  and exits 5 when a guard still waits.

### Stages

- `Domain::plan` returns `Staged` actions, sorted by `Stage`, ties by
  domain order. The planner knows no domain. `TEARDOWN` disables and masks
  before packages leave.
- The Changes popover words a change from its `Summary` through
  `change-<code>`. `Domain::key` only identifies a change. Feature code
  reads queued items through `ChangeSet::queued`.
- Lists share `features::search` (`Searchable`, `Needle::finds`), which
  matches in any case without a lowered copy per row.

## How a change flows

1. A page sends an intent. The feature module queues the wish or drops
   what the system already has. A change equal to the `Snapshot` cancels
   out.
2. Apply builds the `Plan`. User steps run in the app, the root steps
   in one `pkexec garuda-toolbox-helper apply` (`auth_admin_keep`), so one
   password per apply. A protocol change bumps `PROTOCOL_VERSION`.
3. A failed step stops the plan. Steps that did not run stay queued. The
   snapshot is read again.
4. Every step a domain planned belongs to all of that domain's queued
   changes. When one of them failed or never ran, each of those changes
   stays queued as `Outcome::Failed` with the first such step, even when
   the snapshot already shows its wish: a first step that changes the
   probed state (a `locale.gen` edit) can succeed while a later one
   (`locale-gen`) fails. A step that found nothing to do ended well. A
   dry run records no outcome. The Changes popover tags a failed change
   (`change-unfinished`). It leaves after an apply that finished its
   domain's steps and a snapshot that shows it, or when the user takes
   it back. A new wish for the item makes it untried again. The outcome
   is saved in `changes.json` and the CLI's `queue apply` keeps it the
   same way.

Operations that are no desired state (update, clean cache, refresh
mirrors) run at once or join the queue, never both.

## Root actions

- `RootAction` and `UserAction` are closed enums of value objects, never a
  path or command line from the app (the one path is `ConfigFilePath`). A
  new root capability is a change in `toolbox-plan` and the helper, never
  something a feature adds alone. Value object rules are in
  [security.md](security.md).
- Some actions run at once and are never queued (`CreateUser`,
  `SetPassword`, `ControlUnits`, `WriteConfigFile`, `LoadModule`,
  `RepairBoot`). See [security.md](security.md#config-files) for configuration
  files.
- `Invocation::Always` runs after an earlier failure or cancel of the same
  step and cannot be cancelled. It undoes a temporary state (starting
  NetworkManager again, unmounting).
- User actions run as the user in the app. AUR packages build through
  `paru --sudo pkexec`.
- The helper's read commands (`read`, `logs`, `fetch`) answer one JSON
  line each. See [security.md](security.md#reads).

## Pages

Rules a page keeps that its code does not show at a glance:

- Home: status from `probe_status`, apart from the snapshot.
- Kernels: the running and the last installed kernel cannot be removed.
  `dkms status` runs on a blocking thread.
- Languages: the system language's locale and the last enabled locale
  stay.
- Users and Groups: regular users are ids 1000 to 60000. Guards in
  `core/src/features/users.rs`: root accounts stay, the own account cannot
  be locked or removed, the last member of `wheel` stays, system and
  primary groups stay.
- Network: read on page open and after each plan, never in the snapshot
  except the app's own blacklist file.
- Boot Options: the kernel command line is one variable, so each
  parameter change queues the whole list.
- Diagnostics: collectors run as the user, the kernel log and journal
  through the helper's `logs` where the user may not read them. Copy and
  upload take the plain form (`strip_ansi`). The upload needs a confirm
  dialog that shows the text.

## Catalog format

- One TOML file per catalog. Every entry has an `id` that never changes,
  unique within the file. Text fields are English and become
  `catalog-<file>-<id>` in `catalog.ftl`.
- Toggle catalogs (`components`, `settings`, `gaming`, `setup`,
  `apps/<category>`) share `toolbox_catalog::ToggleCatalog`.
- `check` is one of `package`, `service`, `user-service`, `group`,
  `user-file`, `user-ini`, `user-json`. A click queues one change in that
  domain. A `group` check never names `wheel` or `sudo`.
- `user-*` checks become one `UserConfigEdit` in the domain `UserConfigs`,
  written by the app as the user. `path` is a `HomePath`, `group`, `key`,
  `on` and `off` are `ConfigWord`s (see
  [security.md](security.md#user-config-edits)).
- `restart-user-units` and `reload` need a `user-*` check. `reload` is a
  closed list, never a command. `desktop` matches `XDG_CURRENT_DESKTOP`.
- `url` is https only (`is_web_url`). Turning an entry off keeps an `also`
  package or unit that another active entry lists.
- `config-files.toml` (`ConfigFileCatalog`): `path` is `~/…` (a
  `HomePath`) or a `ConfigFilePath` below `CONFIG_ROOTS`. A writable
  root-owned entry needs its line in `plan/src/config_file/writable.rs`.
- `data/catalog/apps/` is generated by `cargo xtask wiki-sync` from the
  Arch wiki. Entry ids come from the first package and are unique across
  the files. Only `--write` writes. Without sync databases it fails unless
  `--unfiltered`.
- A toggle page hides a package entry that is neither installed nor in
  `SyncPackages`, unless a change of it is queued
  (`features::toggles::is_offered`).

## Accessibility

- Every interactive item is reachable with Tab and arrow keys and shows
  focus. Tab order follows [ui-rules.md](ui-rules.md).
- Icon-only buttons carry an AccessKit name and a tooltip (`icon_button`).
- State is never colour alone: an icon and a word go with every status.
- Sizes come from tokens, fonts from the desktop. Animations use `motion`
  durations and stop with reduced motion.
- Banners, Console status and toasts are live regions. The terminal is a
  live region only while its last line asks something.
