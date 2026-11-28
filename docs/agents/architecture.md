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
  applies them before the next frame. `PageContext` holds what every page
  reads (snapshot, queue, catalogs, plans, commands), and a page's own
  data comes from its service (`context.service::<T>()`): the dkms
  reading from `Kernels`, the unit lists from `Units`, the app pictures
  from `Artwork`. The design system is
  `crates/gui-kit`. The window's background is translucent over the blur
  the compositor draws behind it, where one is offered
  ([`backdrop` in toolbox-appmenu](#crates)). Cards and controls stay opaque, and blur
  inside the window is [Frost](#frost).
- Console: a full terminal, so full-screen programs such as paru's
  editor work. A plain-text transcript beside it serves screen readers,
  copy and share. Output is never clickable. A queue is applied as a whole.
- Reading state: libraries and D-Bus first (`alpm`, systemd1 and login1
  over zbus), plain files for locales and kernels. `dkms status` is the one
  parsed program output.
- Translations: the core sends codes and data, never text. See
  [translations.md](translations.md).
- Dates: `jiff::Timestamp` is the instant and `jiff::civil` the civil date,
  parsed, computed and formatted with jiff. `icu` only lays out the text the
  user reads (`app/gui/src/dates.rs`). `SystemTime` is kept for clock reads
  that make a name or an age and hold no date: `fs` and `exec`, where jiff
  may not link, and the run ids of `commands` and `services`. Whole Unix
  seconds stay where the stored form is a number (`https::Clock`, the news
  and AUR caches).
- Packages: `PackageSource` (local) and `SyncSource` (the sync
  databases `/etc/pacman.conf` names) are read only, never synced. alpm's
  build script refuses another libalpm major, so bump `alpm` when Arch
  moves to one.
- AUR: read from a trimmed copy of the daily metadata dump (see
  [security.md](security.md#aur-metadata)). A build is the `aur-packages`
  domain (paru as the user), a removal the `packages` domain's
  `pacman -Rns`. Its wish is `AurWish`: `Paru` plans paru's install as
  before, `Waiting` (order not read yet) plans nothing, and `Built` holds
  the parsed `paru -Po` order, its `BuildRun` and the names of the order
  installed when it was read. Any `Waiting` wish holds back the `Built`
  wishes, and `Built` wishes of two runs plan no split actions. Only
  `paru` and `waiting` are saved or sent. A `built` form from a file or
  the bus comes back as `Waiting`, so the app's own order read is the only
  way an order becomes a `Built` wish.
- The review builtin `ReviewAur` (`crates/exec/src/aur/review.rs`) fetches
  into the app's own clone `~/.cache/garuda-toolbox/aur-clone` and asks
  once before any build ([The review](security.md#the-review)).
- The split plan (`crates/core/src/features/packages/aur/build.rs`) in order:
  1. `ReviewAur` with every base in build order, when there is a base.
  2. `InstallPackages` for repository targets, when there are any.
  3. `InstallDependencies` for the repository lines that are not targets.
  4. Per base: `BuildAur`, then `InstallBuilt` for each reason that has names
     (Dependency, Keep, Explicit).
  5. `RemoveMakeDependencies` for the MAKE names no installed package keeps,
     without protected names, when there are any.

  The domain puts the `InstallAur` of `Paru` wishes after the review and
  before the rest, so the review asks before any root step of the AUR plan. `BuildPlan`
  holds `actions` and `bases`.

- Which path a click takes (`features::packages::aur::toggle`). The snapshot
  part `AurRoute` (`crates/state/src/aur.rs`) is probed in `probe_snapshot`
  by `toolbox_system::probe_route`, from the helper's own check of the
  calling user's build folder (`toolbox_exec::route`). `Accepted` queues
  `Waiting` (the split path). `Refused { kind, detail }` queues `Paru` (the
  old path). A missing part refuses as `Refusal::Unread`. `RouteKind` names
  the refusals a text tells apart (`aur-route-*`, see
  `toolbox_services::aur_orders::route_text_id`).
- Conflicts are checked on the split path only. `AurNames` (the window's
  copy part) lists each package's `conflicts` by name, without version
  constraints. A conflict with an installed package refuses with
  `Refusal::Conflicts`, its subject the installed package. The check fails
  closed: no `AurNames` on the split path is `Unread`.
- The command line overrides the route part with `RouteKind::ThroughSudo`
  when it starts the helper through sudo (`root::sudo_route`, see
  [security.md](security.md#aur-installs)).
- Orders are filled in memory, in one place for both front ends.
  `toolbox_services::aur_orders::read_into` (a `ChangeSet`) and `fill` (a
  `Queue`) read the orders of every queued `Waiting` or `Built` build with
  `read_orders`, then `give_order` sets `Built` for each order that came back
  and keeps or sets `Waiting` for each that did not. A `Paru` wish is never
  touched. `read_orders` blocks on paru, so a caller runs it in
  `spawn_blocking`. Filling writes nothing when only orders changed: a
  `Built` wish saves as `waiting`, and `Queue::edit` writes only a changed
  saved form (pinned by `an_order_alone_changes_nothing_on_disk`).
- A `Waiting` wish holds Apply. `may_plan` refuses with `Refusal::OrderWaits`
  while `waiting_orders` names one, and `Queue::ready` holds on it. One such
  wish holds the whole queue, Paru wishes included.
- Review rows: `BuildOrder::outline` gives the pkgbases one queued build
  builds and the repository packages it installs as dependencies
  (`build::Outline`).
- The command line's split flow. `queue apply` (`crates/commands/src/queue.rs`,
  `queue/run.rs`) restores and checks the queue (`ready::checked`), refuses
  waiting builds under sudo (`ready::sudo_waits`), then reads and fills the
  orders (`ready::filled`) and only then plans. `packages install --aur`
  (`packages/aur/install.rs`, `plan_install`) queues, fills and plans in the
  same order. An order that cannot be read stops the command with exit 5,
  an installed conflict with exit 8 (`Refused::Rule`), and a setup on the old
  path is told why in a warning line first (`warn_old_path`).
- The window's background read of the orders (`Packages`, its `OrderReads`
  in `toolbox_services::aur_orders::reads`). `Packages::ask_orders` starts
  one blocking read at a time for a `Waiting` build no read named yet, and
  every queued build when forced. `App::take_orders` lands it with
  `land_orders`, which fills the queue, so a build left out of the queue is
  not put back. A missing order keeps its reason in `order_missing` and is
  not asked again on its own. The route check runs again on that thread
  (`RouteSource`), and the samples of a self-test build are
  `SampleAurOrders` and `SampleRoute` in `toolbox_system`.
- The window's Apply (`app/gui/src/app/services/orders.rs`): with AUR builds
  queued it sets the pending Apply to the queue's revision and forces a
  read of every order. A landed read fills the queue, then the plan is sent
  in that same pass, unless the revision changed (a queue edit drops the
  Apply). A second click while it waits does nothing. A queue without
  builds plans at once, in the click's pass.
- The Console's review: `features::console::asks_review` matches the text
  of `ReviewAur` (pinned by a test against the plan). While that step runs,
  a line above the header shows the hint and **Continue** (sends `y` and
  Enter as input), since the header has no room for them. Continue never
  takes the focus, so Enter does not click it.
- The Changes rows of AUR builds (`shell/changes/build.rs`) show the notes
  of `App::holds`. The hold of Apply names the build (`Hold`, text built with
  its arguments), and Read again sends `PackagesCommand::ReadOrders`.

## Names

| Thing          | Name                                                                                                    |
| -------------- | ------------------------------------------------------------------------------------------------------- |
| App            | Garuda Toolbox, binary, Wayland app_id and icon `garuda-toolbox`                                        |
| Root helper    | `/usr/lib/garuda-toolbox/garuda-toolbox-helper` (off `PATH`)                                            |
| Polkit actions | `org.garudalinux.toolbox.{apply,read,logs,fetch}`                                                       |
| D-Bus name     | `org.garudalinux.toolbox` (session bus, `org.freedesktop.Application`, `org.garudalinux.toolbox.Queue`) |
| Global menu    | `com.canonical.dbusmenu` at `/MenuBar`, on its own session connection under its unique name             |
| Desktop file   | `garuda-toolbox.desktop`, autostart links point to `/usr/share/garuda-toolbox/autostart.desktop`        |
| XDG dirs       | `garuda-toolbox` (config, state, cache)                                                                 |
| Environment    | `GARUDA_TOOLBOX_*`                                                                                      |
| Installed data | `/usr/share/garuda-toolbox/catalog`                                                                     |

## Crates

```
toolbox-gui (bin garuda-toolbox)
  ├─ toolbox-commands (the CLI) ─ toolbox-services, toolbox-cli, toolbox-guide, toolbox-aur
  ├─ toolbox-services ─ toolbox-core, toolbox-system, toolbox-exec, ...
  ├─ toolbox-core ─ toolbox-catalog ─ toolbox-state ─ toolbox-plan ─ toolbox-text
  ├─ toolbox-system ─ toolbox-exec ─ toolbox-pty, toolbox-fs
  ├─ toolbox-gui-kit ─ toolbox-terminal ─ toolbox-pty
  ├─ toolbox-privatebin, toolbox-news, toolbox-aur ─ toolbox-https
  └─ toolbox-config, toolbox-log, toolbox-instance, toolbox-cli, toolbox-appmenu (with its `backdrop` module)
toolbox-helper (bin, root) ─ toolbox-plan, toolbox-exec
dev only: toolbox-lint, toolbox-xtask
```

Link rules (check with `cargo tree -p <crate> --edges normal`):

- Only `toolbox-gui` and `toolbox-gui-kit` link egui, so
  `cargo nextest run -p toolbox-commands` builds without it.
- `toolbox-helper` links `toolbox-plan` and `toolbox-exec` only among the
  workspace crates. Its dev-dependencies turn on the `testing` features of
  `toolbox-fs`, `toolbox-plan`, `toolbox-state` and `toolbox-system`, which
  no release build links.
- `toolbox-pty`, `toolbox-fs`, `toolbox-https`, `toolbox-appmenu` and
  `toolbox-text` link nothing of the workspace. Every outside request goes
  through `toolbox-https`, so the HTTPS rules live in one place.
- `toolbox-text` sits below `toolbox-plan`, so every crate that keeps
  outside text can hold it as `OutsideText`.
- `toolbox-system` links neither `toolbox-https`, `toolbox-privatebin` nor
  `toolbox-log`. The upload, the public address and the pages' states are
  `toolbox-services`'s.

| Crate                | Holds                                                                                                                                                                                                                                                                  | Must not                                                                       |
| -------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| `toolbox-plan`       | value objects, `RootAction`, `UserAction`, `Plan`, helper protocol, validation                                                                                                                                                                                         | do I/O                                                                         |
| `toolbox-text`       | `OutsideText`, made only by `plain`, `plain_up_to` and `keep_lines`, `is_hidden`                                                                                                                                                                                       | link anything but serde, do I/O                                                |
| `toolbox-state`      | snapshot parts, lsblk and dkms parsing, reports                                                                                                                                                                                                                        | do I/O, be linked by the helper                                                |
| `toolbox-pty`        | programs in a PTY, process group cancel, `strip_ansi`, `styled_lines`                                                                                                                                                                                                  | know actions or egui                                                           |
| `toolbox-terminal`   | the Console's emulator: bytes in, styled rows and cursor out, keys to bytes                                                                                                                                                                                            | know egui or the helper                                                        |
| `toolbox-guide`      | `docs/src` built in by `build.rs`, search, link and picture tests                                                                                                                                                                                                      | link egui, read files at run time                                              |
| `toolbox-exec`       | plan steps in PTYs, `capture`, `secure` (`openat2`), `preflight`, builtins                                                                                                                                                                                             | link a derive crate other than `thiserror`                                     |
| `toolbox-fs`         | atomic replace (`replace_at`, `replace`)                                                                                                                                                                                                                               | link anything but rustix                                                       |
| `toolbox-helper`     | `apply`, `read <kind>`, `logs <kind>`, `fetch <path>`                                                                                                                                                                                                                  | run free-form commands                                                         |
| `toolbox-system`     | ports to the running system as traits plus adapters, status probes                                                                                                                                                                                                     | decide, hold UI state, send a request outside the machine                      |
| `toolbox-catalog`    | TOML schemas, loader and validation for `data/catalog/`                                                                                                                                                                                                                | probe the system                                                               |
| `toolbox-core`       | `ChangeSet`, `Snapshot`, the planner, `features/<name>`                                                                                                                                                                                                                | do I/O                                                                         |
| `toolbox-config`     | settings file, XDG paths, autostart link                                                                                                                                                                                                                               |                                                                                |
| `toolbox-log`        | rotating log file, see [logging.md](logging.md)                                                                                                                                                                                                                        | look up XDG folders                                                            |
| `toolbox-https`      | the one HTTPS agent (`Https`, `Rules` per caller, `Clock`)                                                                                                                                                                                                             | follow a redirect, read a body past its cap, know a server                     |
| `toolbox-privatebin` | PrivateBin v2 paste format and upload                                                                                                                                                                                                                                  | take a server from input, log a link                                           |
| `toolbox-aur`        | the AUR from its daily dump only (`Cache`, `parse`)                                                                                                                                                                                                                    | call the RPC, build a URL from input, ask twice a day                          |
| `toolbox-news`       | Garuda's and Arch's RSS feeds (`Cache`, `Feeds`)                                                                                                                                                                                                                       | build a URL from input, ask more than every six hours                          |
| `toolbox-instance`   | single instance: `activation`, `bus`                                                                                                                                                                                                                                   | ask more than window, page and queue, decode a queue edit                      |
| `toolbox-appmenu`    | the global menu: `com.canonical.dbusmenu`, the X11 Registrar, KWin's Wayland appmenu; `backdrop`, the blur the compositor draws behind the window (`ext-background-effect-v1`, KWin's `org_kde_kwin_blur`, KWin's X11 property, `choice` is pure); window only, no CLI | link egui, hold an item that applies the queue, decide the look, read settings |
| `toolbox-cli`        | clap arguments, exit codes, `output::Paint`, `Prompt`                                                                                                                                                                                                                  | run a command or touch the system                                              |
| `toolbox-gui-kit`    | tokens, theme, icons, fonts, widgets, toasts, `terminal_view`, `frost`                                                                                                                                                                                                 | pick a colour outside the tokens                                               |
| `toolbox-services`   | what the window and the CLI share (`apply`, `queue`, `diagnostics`, `share`, `address`, `locks`, ...)                                                                                                                                                                  | link egui, `toolbox-gui-kit` or a root helper crate                            |
| `toolbox-gui`        | `app/*`, `shell/*`, `pages/*`, GUI services, `ports.rs`, `selftest/*`                                                                                                                                                                                                  | hold feature logic                                                             |
| `toolbox-commands`   | every subcommand, its output, Ctrl+C handling                                                                                                                                                                                                                          | link egui or `toolbox-gui-kit`                                                 |
| `toolbox-lint`       | comment and shape rules for Rust                                                                                                                                                                                                                                       | ship                                                                           |
| `toolbox-xtask`      | the `cargo xtask` commands                                                                                                                                                                                                                                             | ship                                                                           |

### Frost

Menus, lists, toasts, the Changes popover, the status note, dialogs and
the Console show a live blur of what lies under them plus a tint. The
Changes footer bar under the page is window coloured, not frosted
(`the_footer_is_window_coloured_and_the_popover_frosted` in
`app/gui/tests/integration/changes.rs`). The header band shows the blur
under a neutral scrim.

- `frost::install` turns it on. Without it, or when the GPU refuses a
  pipeline, a surface shows its tint.
- `frost::reserve` takes a slot, filled at `ctx.on_end_pass`.
  `live/gather.rs` leaves text out, as atlas sampling is unstable at a
  quarter scale. A result is kept once the same shapes rendered twice.
- Every tint and scrim is as thin as its texts keep 4.5:1 over the app's
  own surfaces. `header_scrim` and `terminal_glass` are neutral black or
  white, so nothing under them casts a tint, and never below `SCRIM_LEAST`
  (`header_scrim` never below `HEADER_SCRIM_LEAST`, so an accent under the
  band does not smear) (`tokens/frost.rs`, `tokens/terminal.rs`). The
  dialog `scrim` is black at a fixed alpha per theme. Without the blur the band is
  the opaque `window` colour and the terminal box the solid `terminal`.
- The band uses `frost::backdrop_within`, which leaves out shapes outside
  it and shadows, so the sidebar and controls below do not smear in.
- `terminal_edge` adds a hairline, as thin white glass on the light sheet
  has no edge.
- `terminal_ansi` moves ANSI 1 to 6 and 9 to 14 in OKLCH lightness only,
  until they reach 4.5:1 on the solid `terminal` and the thinnest glass.
- `cover_page_bottom` ends the scroll area at the Console's top while the
  page paints on under the sheet, so the blur shows the page.
- `sidebar` is `window` with `header_scrim` on it, opaque, so every text
  that reads on `window` reads there (`tokens/tests/sidebar.rs`). The
  status bar and the Console sheet are `pane`, which stands apart from the
  `terminal` and from `sidebar`.

## Errors

A function that can fail returns its own error type, derived with
`thiserror`, the helper's crates included. A wrapped error stays its
`#[source]`, behind an `Arc` where the type must stay `Clone`. A probe
fails with a typed `ProbeError` variant per cause, never free text. A fake
fails with `ProbeError::fake` (feature `testing`). `Result<_, String>`
only in `toolbox-xtask`, `toolbox-lint` and clap value parsers (`crates/cli/src/args*`,
`crates/commands/src/settings.rs`), because clap
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
  only the newest answer of each kind lands. A read a page asks for again
  and again (open, Refresh, after a plan) is an `Asked`: one runs at a
  time, and asks meanwhile become one more read.
- `apply` runs root segments through the helper, user segments in the app.
- Every background result wakes the frame with `ctx.request_repaint()`.
- A panic in a task reaches the log and the crash file (`panics.rs`).
- Tests: pages in `egui_kittest` with `PageFixture` (`testing.rs`), timers
  with tokio's paused time.

## Extending

| To add                  | Implement                                                                                                                                                                                                                                                                                                                                                                                                                                            | Untouched                        |
| ----------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------- |
| A kind of queued change | a `toolbox_core::Domain` (`Item`, `Wish`, `State`, `ID`, a `STAGE` no other domain uses, `CODES`, `is_satisfied`, `plan`, `key`, `summary`), a `SummaryCode` variant per sentence with its `change-<code>` text in `changes.ftl`, its `.with::<D>()` in `domains::all()` (toolbox-lint rule `domain-registered`), its CLI command and its line in `COMMANDS` of `crates/commands/src/parity.rs`, its saved file in `crates/core/src/saved/fixtures/` | `ChangeSet`, `Snapshot`, planner |
| Its probe               | a function that `Snapshot::insert`s the domain's `State`, a `Probe` variant, `probe-<id>` in `status.ftl`                                                                                                                                                                                                                                                                                                                                            | the snapshot service             |
| A root step             | a `RootAction` variant (see [security.md](security.md))                                                                                                                                                                                                                                                                                                                                                                                              | apply service, Console           |
| A catalog entry         | TOML only                                                                                                                                                                                                                                                                                                                                                                                                                                            | everything                       |
| A page                  | [A page](#a-page)                                                                                                                                                                                                                                                                                                                                                                                                                                    | shell, palette                   |

### A page

1. `crates/core/src/features/<name>`: pure logic with tests.
2. `crates/services/src/page.rs`: a `PageId` variant with its `slug`
   (`--page`, palette, D-Bus activation) and sidebar `Group`. Icon and hue
   in `app/gui/src/page_ui.rs` ([icons.md](icons.md)).
3. `app/gui/src/pages/<name>.rs`: a `PageView`, tests in
   `pages/<name>/tests.rs`.
4. `app/gui/src/pages.rs`: its field and its line in `Pages::view`.
   `PageId::ALL` drives the sidebar, the palette and the start options.
5. A service in `app/gui/src/<name>.rs` for reads outside the snapshot and
   for the page's clicks, and its dry-run stand-in in `ports.rs` when the
   read changes or leaves the machine. It implements `PageService`
   (`app/gui/src/page_service.rs`) and answers with `AppCommand`s. Its
   field in `Services`, its line in `Services::all_mut` and its `Provides`
   impl are all `App` needs. The page reads it with
   `context.service::<Name>()`.
   - `PageService::handle` gets every command with the queue and the
     snapshot. It returns `Handled { taken, edit, answers }`. `taken` says
     the service ran the command, so `App` leaves it, also for a refused
     click. The edit is a copy of the queue the click changed
     (`queue::clicked::on_copy`), which `App` takes in the same frame
     through `Queue::take_click`. That keeps the met guards, unlike
     `Queue::take` of the bus. Work that needs `App` goes back as an answer (`AppCommand::ReadSystem`, `Answer::ShowPage`).
   - The service owns the page's last refused click as a core
     `features::Refused<R, S = String>`, most through `queue_or_refuse`, or
     `queue_or_keep` (`app/gui/src/refused.rs`) for a rule that names its
     own subject, such as a protected dependency. `R` implements
     `features::RefusalText`, so the text id is `R::TEXT_PREFIX` plus the
     strum code, and every refusal text names its subject `$name`. A page
     shows it with `Shown::text`, or `Shown::text_with` for more
     placeables such as the `$why` of a protected package. Its banner
     sends `AppCommand::DismissRefusal(PageId)`, which
     `PageService::receive` hands to the service's `dismiss_refusal`. A
     toggle page names its catalog's page (`CatalogId::page`).
   - A click that needs `App` state (a run at once, a dialog, the
     preferences) stays in `app/gui/src/app/<name>.rs` and sets the
     refusal with the service's `set_refusal`. A page whose part is a
     single short match keeps it in `app/commands.rs` instead, an answer
     in `app/answers.rs`. Its command enum lives in
     `app/gui/src/command/<name>.rs`, or in `command.rs` with its `From`
     in `command/conversions.rs` when it is a few lines. Its match keeps an empty
     arm for each command a service takes, as no match on an enum may
     use a wildcard.
6. Texts in `crates/services/i18n/en-US/<name>.ftl` and `de-DE/<name>.ftl`.
7. CLI commands (`crates/cli/src/args/*`, `crates/commands/src/*`).
   `crates/commands/src/parity.rs` fails for a queue domain without one.
8. Integration tests and snapshots (`app/gui/tests/integration/<name>.rs`)
   in dark and light.
9. Self-test steps in `app/gui/src/selftest/<name>.rs`, listed in `SUITES`.
10. A page under `docs/src/` and its line in `SUMMARY.md`.

### Dry run

`GARUDA_TOOLBOX_HELPER=dry-run` makes the helper only check plans. Every
other effect is a port that `app/gui/src/ports.rs` picks at start (`DryRunTransport`, `DryRunAddress`, `DryRunActions`,
`DryRunLauncher`).
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
  `crates/core/src/saved/fixtures/` holds one saved file per domain. A
  changed shape fails their test until the version is bumped and the
  fixture written again.
- While a window holds the bus name, `garuda-toolbox queue` never writes
  the file. It sends a `QueueEdit` over `org.garudalinux.toolbox.Queue`,
  and the window runs every guard again (`Queue::answer`). The edit names
  its own `QUEUE_EDIT_VERSION` and the `SAVED_VERSION` of the changes it
  carries, and a mismatch of either is refused. The window
  refuses an edit while its Console runs a plan. `queue apply` runs only
  without a window, so a change never runs twice.
- The item is the key of a change, the wish its value. A second wish
  replaces the first. Work to do has the wish `Queued`. `ChangeSet::want`
  ignores a wish that `Domain::fits` refuses.
- `ChangeSet::want` and `remove` are `pub(crate)`. The window and the
  command line queue through one `features::<page>` function per click,
  such as `tweaks::queue` or `services::queue`, so both keep the page's
  rules. Tests of other crates import `toolbox_core::testing::Queue`.
- A restored queue was queued before the system was known.
  `features::Unchecked` runs each `features::Guard` once over it, as soon
  as the snapshot holds the part the guard needs. A page that queues
  changes adds its `Guard` variant, the same function the page calls.
- A guard that waits fails closed: `Queue::plan` refuses with
  `Refusal::GuardsWaiting` while `Unchecked::holding` names a guard with
  a queued change of one of its `Guard::domains`. Apply stays disabled
  and says why (`changes-wait-*`). `Queue::plan` also refuses with
  `Refusal::OrderWaits` while an AUR build waits for its order (the
  `changes-wait-orders` hold). Both come from `features::may_plan`, so
  `Queue::ready` and the CLI's `ready::checked` share one rule.
- No probe reads `AccountLockFlags`, the AUR names or `CurrentLoader`.
  `toolbox_services::unprobed` joins them: `Needs::of` names what a queue
  waits for (a held guard's part, the loader while a loader change is
  queued), `joined` adds what an `UnprobedReads` knows. The app joins its
  pages' cached reads into every snapshot (`combine` in
  `app/gui/src/app/services.rs`) and asks the Users, Packages or Boot read
  itself once the queue needs it (`read_held_parts`). `queue apply` reads
  them through the helper (`crates/commands/src/queue/ready.rs`), exits 5
  when a guard still waits and settles against `ready::after`, so a loader
  change that applied leaves the queue.

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
2. Apply builds the `Plan`. User steps run in the app. The root steps of
   the whole plan run in one `pkexec garuda-toolbox-helper apply`
   (`auth_admin`), so one password per apply. The app starts it at the
   plan's first root step, keeps it across the user steps, passes each later
   root segment's gate and ends it when the plan stops
   ([security.md](security.md#the-helper)). A protocol change bumps
   `PROTOCOL_VERSION`.
3. A failed step stops the plan. Steps that did not run stay queued. The
   snapshot is read again.
4. Every step a domain planned belongs to all of that domain's queued
   changes. When one of them failed or never ran, each of those changes
   stays queued as `Outcome::Failed` with the first such step, even when
   the snapshot already shows its wish: a first step that changes the
   probed state (a `locale.gen` edit) can succeed while a later one (`locale-gen`) fails. A step that found nothing to
   do ended well. A
   dry run records no outcome. The Changes popover tags a failed change (`change-unfinished`). It leaves after an apply
   that finished its
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
  `SyncPackages`, unless a change of it is queued (`features::toggles::is_offered`).

## Accessibility

- Every interactive item is reachable with Tab and arrow keys and shows
  focus. Tab order follows [ui-rules.md](ui-rules.md).
- Icon-only buttons carry an AccessKit name and a tooltip (`icon_button`).
- State is never colour alone: an icon and a word go with every status.
- Sizes come from tokens, fonts from the desktop. Animations use `motion`
  durations and stop with reduced motion.
- Banners, Console status and toasts are live regions. The terminal is a
  live region only while its last line asks something.
