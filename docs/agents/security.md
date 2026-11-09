# Security

The root and argv rules of [AGENTS.md](../../AGENTS.md#rules-for-every-change)
apply everywhere.

## Who runs what

| Runs as             | What                                                                                                                                                                        |
| ------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| User                | the app, probes, user units and journals, user configuration files, `paru` (asks for root itself with `--sudo pkexec`), the editor, `rfkill`, `ping`, `traceroute`, `lsblk` |
| Root, through D-Bus | login1 `Reboot`, systemd1. Both check polkit themselves                                                                                                                     |
| Root, helper        | everything else, as typed `RootAction`s, one `pkexec` per apply                                                                                                             |

The app refuses to start as root.

## The helper

- `/usr/lib/garuda-toolbox/garuda-toolbox-helper`, off `PATH`, pinned by
  the polkit actions.
- It refuses to run when it is not root, when stdin is a terminal, or when
  the protocol version differs (read before the rest is decoded).
- `apply` reads one `Request` from stdin and validates every step before
  the first runs. One invalid step rejects the request. A refused name is
  quoted with at most 64 characters, control characters spelled out.
- Programs run as argv at their fixed path (`Program::path`) with a clean
  environment: `PATH` is `/usr/local/sbin:/usr/local/bin:/usr/bin`
  (`SYSTEM_PATH`, `crates/exec/src/pty_runner.rs`), the locale fixed.
- Only a development build looks for a missing program elsewhere, in
  `/run/current-system/sw/bin` and `/run/wrappers/bin`, never in the
  caller's `PATH` (`crates/exec/src/located.rs`).
- A closed stdin interrupts the program and skips the rest. Every client
  keeps stdin open until the `done` event.
- SIGTERM and SIGHUP interrupt the program, skip the rest and still run
  the `Invocation::Always` part. Programs run in their own session and
  never get the helper's signal. SIGINT is ignored, because Ctrl+C reaches
  the command line too, which answers `Answer::Finish`, then
  `Answer::Cancel` on the second press.
- A panic fails its step. A panic in an `Always` command ends the helper,
  so cleanup commands stay simple. The release profile keeps unwinding.
- `Answer::Input` above 64 KiB (`MAX_INPUT_BYTES`) is refused. The app
  splits pastes and strips control characters but tab and line breaks
  (`toolbox_terminal::encode_paste`). Input and output queues are bounded,
  so a stuck or flooding program never stalls the helper.
- The helper links no runtime, GUI, D-Bus, HTTP or `parking_lot`
  (`app/helper/tests/dependencies.rs`, `cargo tree --target all`).

### Rules `scripts/lint.sh` enforces

- `no-shell-command`: no `Command::new` of `sh`, `bash` or `/bin/sh`.
- `program-path`: `Command::new` takes an absolute path or a
  `toolbox_plan::procedure::Program`. The exceptions run as the user:
  `xdg-open` in `app/gui/src/links.rs`, `curl` in the xtask.
- `sudo-once`: only `crates/services/src/apply/launcher.rs` names `sudo`.
  The doctor takes `SUDO_NAME` from there.
- `tests-in-file`: no inline `mod tests {` below `src/`.

## Value objects

Every value of a request is a `toolbox-plan` type, checked again when the
request is decoded. Names follow a `--` in argv wherever the program takes
one. The rules with a reason the code does not show:

- No name starts with `-`, so no value reads as an option.
- `FullName`, `HomeDirectory` and `ShellPath` have no `:`, so they cannot
  add a field to `/etc/passwd`.
- `HomeDirectory` lies below `/home/`, `/srv/`, `/var/lib/` or `/opt/`
  (`HOME_ROOTS`). Elsewhere `usermod -d -m` would hand the user files in a
  tree root trusts, such as `/etc/systemd/system`.
- GRUB values (`MenuEntryId`, `GrubPicture`, `GrubThemeFile`) have no `"`,
  `\`, `$` or backtick and are escaped by `toolbox_plan::quote`, because
  `grub-mkconfig` sources `/etc/default/grub` as root. `GrubPicture` and
  `GrubThemeFile` take only ASCII letters, digits and `_ . - / + @ , = :`,
  because `/etc/grub.d/00_header` writes them unquoted into `grub.cfg`.

The helper repeats what only the wire shows: `DeleteUser`, locking and
`SetHomeFolder` refuse a user id, `DeleteGroup` a group id outside 1000 to
60000, `CreateUser` and `SetShell` a shell not in `/etc/shells`,
`SetSystemLocale` a locale not enabled in `/etc/locale.gen`,
`SetHomeFolder -m` a taken target, and `SetHomeFolder -m` and `CreateUser`
with a chosen home one below any folder everyone may write
(the sticky bit too), because another user could place a link there
between the check and the move.

## Protected packages

`toolbox_plan::PROTECTED` (`crates/plan/src/protected.rs`) lists the
packages no page, no command and no helper request removes. It is
compiled in, so no missing file can empty it.

- Every removal asks `features::packages::queue::may_remove` in
  `toolbox-core`: the package itself, then
  `protected::may_remove_with_cascade` over the whole queued batch, as
  one `pacman -Rns` takes it. The switches go through
  `features/toggles/guards.rs`, which checks the entry's `also` packages
  too. The CLI goes through the same functions. A new way to remove a
  package must ask them too.
- The cascade check needs the `PackageGraph` part. Without it no removal
  passes.
- The helper refuses a `RemovePackages` step that names a protected
  package (`Request::validate`), and `RemoveOrphans` keeps protected
  orphans out of `-Rns` and names them in the Console.
- `cargo xtask catalog-validate` refuses a protected `also` package. A
  protected main package is allowed: its switch stays on.
- A pacman group name expands in the helper. The app blocks it only
  because it is no installed package.

## Files the helper writes

- Through `toolbox-exec`'s `secure` module only: folders opened with
  `openat2` and `RESOLVE_NO_SYMLINKS | RESOLVE_BENEATH`, atomic replace,
  mode 0644. No fallback for kernels without `openat2`.
- A file that is not UTF-8 fails the step untouched. A missing
  `/etc/default/grub` is never created.
- `SetWifiBackend` and module load or unload restart NetworkManager as
  `Invocation::Always`, so an apply never leaves the machine without
  network management.
- `dkms install` runs the module's own build scripts as root, as
  installing its package did. `ChrootInstallation` gives a root shell in
  the installed system, behind the same password as every apply.

## Secrets

Account passwords and LUKS passphrases.

1. The dialog checks the text when a field changes and builds the
   `toolbox_plan::Password` it sends on confirm, never once per frame. On
   close it empties its fields and drops egui's state of them, undo
   history included (`widgets::forget_text`). The CLI reads it on
   `/dev/tty` without echo or with `--password-stdin`, never from
   arguments.
2. `Password` has no `Display` or `Serialize`, `Debug` redacts it, its
   bytes are overwritten on drop. At most 512 bytes, no control character.
3. It sits behind `#[serde(skip)]` and never enters the `ChangeSet`, the
   saved queue or the Changes popover.
4. After the request line the app writes `secret <step> <password>` per
   step that needs one. The helper reads them after the root check,
   overwrites its buffer, and rejects a missing or misplaced line without
   quoting it.
5. `chpasswd` and `cryptsetup open --key-file -` get it as
   `Invocation::Feed`: pipes, no PTY, which would echo. Output is scrubbed
   of it. A fed program gets no cancel and dies after 120 s.
6. Logs carry the step count, never argv of a secret step or a secret
   line.

Limits: the dialog's text field, the copies egui makes while it edits
and the helper's buffered stdin are freed, not overwritten. Each lives
only as long as its process.

## Reads

`garuda-toolbox-helper read <kind>` takes one
`toolbox_plan::protocol::ReadKind`, never a path. Polkit lets it run
without a password.

- `shadow-locks`: user names and locked flags, never a hash. The buffer is
  overwritten before the answer. Only accounts of uid 1000 to 60000 and
  the caller's own. The caller is the peer uid of stdin, a socket pair the
  app makes (`Policy::identified`). Any other stdin, root, a system
  account and `nobody` are refused. `!` and `*` count as locked.
- `grub-menu`: titles and ids of at most 1000 entries, never the password
  hash or the kernel command line. Only the branches of the running
  firmware count (`/sys/firmware/efi`, as the app's own read).
- `sdboot-menu`: the settings of `loader.conf` and the ids, titles and
  kernel releases of at most 256 entries on the ESP. The helper finds the ESP
  in the mount table and reads only a UKI's header and its `.osrel` and
  `.uname` sections, and runs no program.

`garuda-toolbox-helper logs <kind>` (`dmesg`, `journal`) is a separate
first argument with a password, so no log is ever reachable through
`read`. A test fails when a program-running kind belongs to `read`.
Program, arguments and environment are constants. The app asks the helper
only after its own read was denied.

## Config files

Root-owned files are edited as a private copy and written back by the
helper. No editor runs as root.

- A path is a `ConfigFilePath`: absolute, no control character, no empty,
  `.` or `..` part, below `CONFIG_ROOTS`, never below
  `WITHHELD_CONFIG_FILES` and never ending in `WITHHELD_CONFIG_SUFFIXES`
  (`*.netdev`, WireGuard keys in `/etc/systemd/network`)
  (`crates/plan/src/config_file/path.rs`). A new file that holds a secret
  goes on one of those lists.
- A write must also pass `ConfigFilePath::is_writable`
  (`crates/plan/src/config_file/writable.rs`). `toolbox-catalog` refuses a
  writable entry missing there, `catalog-validate` a line no entry needs.
- A fetch must also pass `ConfigFilePath::is_readable`: the write table or
  `READABLE_CONFIG_FILES` and `READABLE_CONFIG_FOLDERS`
  (`crates/plan/src/config_file/readable.rs`). That file is generated by
  `cargo xtask catalog-readable` from the `read_only` entries, and its
  `--check` fails when it is stale. The asked path is checked before any
  look at the file (exit 2), the real path after the links (`outside`).
  The deny list stays the second check.
- A `read_only = true` entry is a view only. The helper refuses a write.
- `toolbox_exec::config_file::resolve` follows symlinks by hand (at most
  40). The asked path, every followed link and the result must each be a
  `ConfigFilePath`. A dangling link or a circle is refused.
- The real path's folder opens with `openat2`, the file with `O_NOFOLLOW`,
  so a link swapped in meanwhile ends the request. Only a regular UTF-8
  file of at most 1 MiB without NUL.
- `garuda-toolbox-helper fetch <path>` is the one command that takes a
  path, so it has its own polkit action. The app asks it only after its
  own read was denied.

The edit:

1. The copy goes into `$XDG_RUNTIME_DIR/garuda-toolbox/edit/` (folder
   0700, file 0600, `O_EXCL`, a symlinked folder refused) and is removed
   by a drop guard.
2. The editor is one `toolbox_plan::Editor`, run as the user from its
   fixed path, else from the user's `$PATH`. That lookup is safe only
   because it runs as the user. `$VISUAL` and `$EDITOR` pick an `Editor`
   by name and are never run.
3. `WriteConfigFile` resolves again and refuses unless the real path
   passes `is_writable` too (`WriteError::NotWritable`). Decoding checked
   only the asked path, and a link in a writable folder may lead to a
   `read_only` file.
4. It fails unless the file still has the original `Fingerprint`, backs
   it up to `/var/lib/garuda-toolbox/backups/` (0600) and replaces it with
   its old owner, group and mode. A missing file is never created.

Limits: no syntax check. Extended attributes and ACLs are not kept.
A change by another writer between the last read and the rename is lost.

A catalog `HomePath` file is edited the same way, but nothing runs as
root: `UserAction::WriteHomeFile` is a builtin in the app, never in the
helper, which has no home. It follows symlinks, checks the `Fingerprint`,
backs up to `~/.local/state/garuda-toolbox/backups/` (0700 and 0600) and
keeps the mode. A missing file is never created.

## User config edits

Catalog `user-*` checks (`UserAction::EditUserConfig`) run as the user in
the app, never through the helper.

- The file is a `HomePath`: relative to the home, no `..`, `.` or empty
  part, no control character.
- INI groups, keys, values and JSON keys are `ConfigWord`s: no `[`, `]`,
  `=`, `#` or line break. Paths and words come from the catalog only.
- The edit is computed in `toolbox-plan` and written atomically, the mode
  kept. A file the edit does not understand fails the step untouched.
- The only follow-up is a restart of catalog user units or a `reload` from
  a closed list.

## Boot repair

One action, `RepairBoot`, offered on the live ISO or with `--boot-repair`.
Devices are `toolbox-plan` values. The mount root and the mapper name are
constants (`crates/plan/src/repair.rs`). `toolbox_exec::preflight` runs
before the first step and again before `grub-install`. It refuses unless:

- every device is a block device of the expected kind (from `/sys` and
  udev data), the ESP is `vfat`, a locked root is `crypto_LUKS`,
- UEFI has `/sys/firmware/efi`,
- no target lies on a disk the running system has mounted (LVM and LUKS
  count),
- no mount is left below the repair folder.

The mount root is made without following a symlink. `grub-install` runs
without `--force`. Unmounting and `cryptsetup close` are
`Invocation::Always`.

## Polkit

| Action                          | Active session    | For                   |
| ------------------------------- | ----------------- | --------------------- |
| `org.garudalinux.toolbox.apply` | `auth_admin_keep` | `helper apply`        |
| `org.garudalinux.toolbox.read`  | `yes`             | `helper read <kind>`  |
| `org.garudalinux.toolbox.logs`  | `auth_admin_keep` | `helper logs <kind>`  |
| `org.garudalinux.toolbox.fetch` | `auth_admin_keep` | `helper fetch <path>` |

- Policy: `data/org.garudalinux.toolbox.policy`, tested by
  `app/helper/tests/policy.rs`. Each action matches the helper path and
  `exec.argv1`. Inactive and remote sessions get `auth_admin`.
- A new root action never gets a password-less rule.
- A development build runs the helper from `target/`, which no action
  names, so pkexec asks every time.
- pkexec is `/usr/bin/pkexec`. A development build also takes a root-owned
  setuid `pkexec` from `PATH`. `toolbox_system::pkexec` classifies every
  refusal.

### Sudo on the command line

The command line may start the same helper with `sudo -- <helper> apply`
(or `read`, `logs`, `fetch`): argv, `--`, no shell.

- Only on a terminal (stdin and stdout), never in the window.
- Only when pkexec cannot ask: missing, not setuid root, no polkit agent,
  or broken.
- Never after a closed dialog or a wrong password.
- The helper validates as always. sudo only replaces the password
  question.

Code: `crates/commands/src/root.rs`, `crates/services/src/apply/launcher.rs`.

## Text from outside

Package descriptions, wiki text, unit descriptions, feeds, program output
and server answers are untrusted.

- egui draws them as plain text. Nothing becomes a link, image or style.
  Banners show the app's own words, the outside value beside them.
- Hidden characters (controls, bidi overrides and marks, zero-width
  characters) go through `toolbox_plan::text::is_hidden` and `plain`
  wherever outside text is kept or printed. New outside text does too.
- URLs open only from the catalog or a fixed list (`app/gui/src/links.rs`)
  and `file://` links of catalog home files. A package's website opens
  only with `http://` or `https://`, checked again by `links::open`.
- `toolbox_pty::styled_lines` reads only SGR colour and bold of program
  output and drops every other sequence. Copy, share and upload take
  `toolbox_pty::strip_ansi`.

### The session bus

Any process of the user can call the window's `org.garudalinux.toolbox`
name. It gains nothing the user's own files do not give: the queue is the
user's, and an apply still asks for the password.

- `org.freedesktop.Application`: names are `PageName`s, tokens visible
  ASCII of at most 1024 bytes. `Open` is refused.
- `org.garudalinux.toolbox.Queue`: `Edit` takes JSON of at most 256 KiB
  and decodes it as a `QueueEdit` (`crates/services/src/queue/edit.rs`).
  Anything that does not decode into its domain's value objects refuses
  the whole edit. Each wish then meets every page guard, as a click.
  At most 64 questions wait for the app (`instance::WAITING_ASKS`), the
  next is refused as `too-many`. Every edit that lands shows a toast.
- Nothing on the bus applies the queue. Refusals cross as fixed codes.

## Network

The app contacts the network only for:

- the Home status check (`checkupdates`, `paru -Qua`),
- the Home news: the Garuda forum announcements feed and the Arch news
  feed (`toolbox_news::Source`), at most every 6 hours,
- the AUR metadata dump, only while the AUR setting is on,
- a PrivateBin upload after its confirm dialog,
- the public address after **Show public address**,
- links the user opens, `ping` and `traceroute` to a host the user typed,
- NetworkManager's own connectivity check.

No telemetry, no update check of the app. A dry run sends nothing to
PrivateBin and asks no address service. A self-test build's dry run
reaches none of the servers.

Every HTTPS request goes through `toolbox_https::Https` (`crates/https`):
rustls with system certificates, HTTPS only, no redirects, a timeout and
a body cap per caller. Each agent is pinned to one host in its `Rules`
and port 443 and refuses a URL of another (`RequestError::OtherHost`)
before connecting. Any status but 2xx is
`RequestError::Status`, a redirect and `304` too. Addresses are
constants, never built from input or an answer.

Proxies come from the environment, as ureq 3 reads them by default:
the first of `ALL_PROXY`, `all_proxy`, `HTTPS_PROXY`, `https_proxy`,
`HTTP_PROXY`, `http_proxy` that parses, minus the hosts in `NO_PROXY` or
`no_proxy`. An `http://` or `https://` proxy only sees a `CONNECT` to the
host. TLS still runs end to end and the certificate is checked against
the host, so a proxy cannot read or change an answer. A `socks` proxy is
not built in (no `socks-proxy` feature): ureq logs a warning and
connects directly. A `User-Agent` says `garuda-toolbox/<version>`, nothing of the
user. The news and the AUR dump store each attempt, failed or not, and
wait the full interval after it instead of retrying.

- News: a post's link is kept only when it starts with its source's post
  prefix.
- Public address (`toolbox_state::network::ADDRESS_SERVICE`): only the
  `<Ip>` element is read, and only when it parses as an IP address.
  Nothing is stored or logged.

### AUR metadata

One GET of `toolbox_aur::DUMP_URL`, the only AUR address. No RPC call.

- At most one attempt per 24 hours. The attempt time is stored before the
  request, so a crash waits a day too. An attempt whose time cannot be
  stored is not made. A stored time in the future counts as now.
- Conditional with the stored `ETag` and `Last-Modified`. One download at
  a time.
- At most 64 MiB gzipped, 256 MiB unpacked and 400 000 packages, each
  name once while reading. Only the kept fields are
  read, each cut to a fixed length. A name that is no `PackageName` drops
  its entry.
- No download while the AUR setting is off, also for an AUR change a
  command queued.
- The copy is replaced whole with mode 600. A failed download keeps the
  older one, which the page then marks as possibly out of date. A copy
  this version cannot read is removed and downloaded again at once.

### PrivateBin upload

One POST to `toolbox_privatebin::SERVER`, only after **Upload** in a
dialog that shows the exact text.

- Encrypted with a random paste key, which stays in the link's fragment
  and never reaches the server.
- Only `status`, `id`, `deletetoken` and `message` of the answer are read.
  Links are built from the constant and an alphanumeric `id`.
- The delete link deletes on opening. Neither link is logged, `Debug`
  hides both.
