# The command line

`garuda-toolbox` is also a command line tool. Without a command it opens the
window. With one it answers in the terminal and never opens a window.

The following commands open the window, open the window on a page, print
what needs attention and update the system without asking:

```console
$ garuda-toolbox
$ garuda-toolbox --page maintenance
$ garuda-toolbox status
$ garuda-toolbox update --yes
```

Every command explains itself with `--help`, and `man garuda-toolbox` shows
the whole reference. Each command has a page of its own too, such as
`man garuda-toolbox-services-list`. Your shell completes commands, options
and the names of the system once the package is installed (bash, fish and
zsh): services (yours with `--user`), switches, apps, kernels, locales,
users, groups, configuration files, maintenance tasks, boot entries, network
drivers and pages. `packages install` completes the packages the
repositories offer, `packages remove` and `packages show` the installed
ones. With `--aur`, `packages install` and `packages show` complete the
AUR's packages from the copy of its package list that `packages search
--aur` keeps. A `Tab` does not refresh that copy. Without a copy the first
`Tab` downloads it, about 14 MB, and gives up after two minutes. A failed
download is not tried again for a day, as with `packages search --aur`.
Language packs, guide pages, the Setup Assistant's choices and the kernel
parameters of `boot param remove` complete as well.
Completion never asks for a password and offers nothing for a part it
cannot read. Without the package, load it by hand:
`source <(COMPLETE=bash garuda-toolbox)` in bash,
`COMPLETE=fish garuda-toolbox | source` in fish, or
`source <(COMPLETE=zsh garuda-toolbox)` in zsh.

A name the system does not have ends with a hint when a close one exists:
``hint: did you mean `sshd.service`?``

## Opening the window

| Option              | What it does                                                           |
| ------------------- | ---------------------------------------------------------------------- |
| `--page <page>`     | Opens the window on a page. `garuda-toolbox pages` lists the page ids. |
| `--action <action>` | Runs an action of that page once it is open, see below.                |
| `--setup-assistant` | Opens the window on the Setup Assistant.                               |
| `--boot-repair`     | Shows Boot Repair on an installed system too, for testing it.          |

If the window is open already, the running window opens the page and asks to
come to the front ([Introduction](introduction.md) says when it does). No
second window opens.

The diagnostics page takes a report as its action:

```console
$ garuda-toolbox --page diagnostics --action inxi
```

## Reading

These commands only read. Each takes `--json` for scripts. The other
`list`, `show` and `entries` commands of the sections below read
too and take `--json` as well (`configs show` prints the file itself).

| Command                                                           | What it prints                                                        |
| ----------------------------------------------------------------- | --------------------------------------------------------------------- |
| `status`                                                          | What Home shows: updates, files to merge, health, restart and facts.  |
| `doctor`                                                          | What the toolbox needs, whether each part works, and the fix.         |
| `pages`                                                           | The page ids for `--page`, with their names.                          |
| `guide list`                                                      | The pages of the [user guide](guide.md), in the order of its parts.   |
| `guide show <page>`                                               | One page of the user guide as text, for example `guide show kernels`. |
| `services list [--user]`                                          | The system's services, or your own with `--user`.                     |
| `toggle list [--page components\|system-settings\|gaming\|setup]` | The switches and whether each is on.                                  |
| `apps list [--category <category>]`                               | The App Catalog's apps and whether each is installed.                 |
| `packages search [<text>] [--installed\|--orphans]`               | Repository and installed packages whose name or description matches.  |
| `packages show <package>`                                         | One package: repository, version, size, dependencies, files.          |
| `kernels list`                                                    | The installed and available kernels, the running one marked.          |
| `locales list`                                                    | The locales, the enabled ones and the system's marked.                |
| `users list [--all]`                                              | Root and the people accounts, every account with `--all`.             |
| `groups list`                                                     | The groups with their members.                                        |

A switch shows `on`, `off`, or `unknown` when its state could not be read.
`doctor` ends with exit code 9 when a part the toolbox needs is missing
or broken (see [Exit codes](#exit-codes)).
Old names still work: `dkms status`, `network status` and `bootrepair
status` are the same as `dkms list`, `network show` and `bootrepair show`.
Some commands have a second name: `packages list` is `packages search`,
`network drivers block` is `network drivers blacklist`,
`network drivers unblock` is `network drivers allow`, `kernels
set-default` is `kernels default`, `locales set-system` is `locales
system`, and `apps uninstall`, `kernels uninstall` and `packages
uninstall` are the `remove` commands.

Every `list` takes `--search <text>`, which keeps the rows whose name or
shown name holds the text, upper or lower case alike. Some lists take
filters of their own: `services list --running`, `--failed` or
`--enabled`, `toggle list --on` or `--off`, `apps list --installed`,
`kernels list --installed` and `locales list --enabled`. Filters apply to
`--json` too. The title still counts the whole list and says how many
are shown.

On a terminal, lists and `show` commands open in a pager:
`$GARUDA_TOOLBOX_PAGER`, else `$PAGER`, else `less -FRX`, which quits at
once when everything fits on the screen. `--no-pager`, or `cat` or an
empty value in the variable, prints directly. The value is split into
words and no shell reads it. The pager gets `LESS=FRX` and `LV=-c` when
these are unset, as git does, so `PAGER=less` keeps the colours too. Long
cells are cut to the terminal's width and end in `…`, the name never. In a
pipe nothing pages and nothing is cut.

Slow reads (`status`, `services list`, `kernels list`, `locales list`,
`dkms list`, `network show`) show a spinner on standard error while they
run, never with `--json` and never on a `TERM=dumb` terminal, so standard
output holds the result alone.

`status --brief` prints one line for a login message or a status bar,
such as `2 updates, 1 warning, disk space low` or `all good`. Parts that
could not be read are left out. `status` ends with exit code 9 when
something needs attention (see [Exit codes](#exit-codes)).

Besides the update check, `status` names the installed packages (with
the foreign ones), the running kernel, the uptime, the free space on
`/`, failed system services, orphans and the size of the package cache.
In `--json` they are `packages`, `kernel`, `uptime_hours`, `disk`,
`failed_services`, `orphans` and `package_cache_bytes`. `--brief` adds
`disk space low`, `1 failed service` and `clean-up due` as Home does.

`status --watch` reads the facts again every minute and prints them only
when they changed, each time under the time they were read, such as
`14:02`. `--interval` sets the wait, such as `30s`, `5m` or `1h30m`, at
least one second. It goes with `--brief` (one line per change, behind the
time) and `--json` (one JSON object per line and change, the shape
`status --json` has). `Ctrl+c`, `SIGTERM` and `SIGHUP` end it with exit
code 0, the normal way to stop a watch.

## Diagnostics

For example:

```console
$ garuda-toolbox diagnostics inxi
$ garuda-toolbox diagnostics all --upload
```

The report is one of `inxi`, `garuda-health`, `systemd-analyze`,
`journalctl`, `pacman`, `dmesg` or `all`, the same as on the
[Diagnostics](diagnostics.md) page. It is printed in full.

With `--upload` it goes to `bin.garudalinux.org` as an encrypted paste
after a question, and the link and the delete link are printed. Whoever has
the link can read the report. The uploaded text has your user name, host
name, home folder, serials, UUIDs, hardware and public addresses replaced
([list](diagnostics.md#what-the-upload-replaces)), and the command says how
many before it asks. The printed report keeps them.

## Changing the system

For example:

```console
$ garuda-toolbox update
$ garuda-toolbox update --aur --yes
```

`update` updates the system now, through garuda-update where it is
installed. `--aur` also updates the AUR packages with paru, as you.

Before anything changes, the command lists the steps under **Planned** and
asks `Continue? [y/N]`. Only `y` or `yes` goes on. Then the password dialog
asks for your password, as in the window, and **Running** shows each step
as it starts, its program's output indented under it, and how it ended. A
closing line tells how it went:

```text
Planned  (1 step)

  systemctl restart -- sshd.service

Continue? [y/N] y

Running

  systemctl restart -- sshd.service
  ● systemctl restart -- sshd.service done

Done
```

A failed step ends with `✗ … failed`, and the run ends with
`error: the change did not finish`. A refused password says why and what
helps. Questions of the programs, such as paru's, are answered by typing.

Each step starts with its place in the plan, such as `[2/5]`, and its
last line says how long it took. `--dry-run` on any command that changes
the system prints the plan and ends with 0, without asking or changing
anything. It cannot go with `--yes`. `diagnostics --dry-run` collects the
report but does not upload it.

On a terminal, output is in colour and every row of a list starts with a
mark: ● on or running, ○ off or stopped, ? not read, ✗ failed. In a pipe
or file, and always with `NO_COLOR` set, the marks and colours are left
out and the state stays as a word. Each list has a title with its count,
and a `hint:` line names the command that acts on it. `--json` gives the
same data for scripts.

When a program fails, one `error:` line says what could not be done. Add
`-v` or `--verbose` after the command to see the program's own words
under it. The whole log goes to the log file only, so the terminal shows
just the command's own output.

When no password dialog can appear (no polkit agent, for example over SSH
or on a plain console), the command says `warning: pkexec is not
available, asking through sudo` and sudo asks for your password in the
terminal instead. It never does that after you closed the dialog or typed
a wrong password.

What you type reaches a program only while one of its steps runs. While
the password dialog asks, no step runs, so your password never reaches a
program. When a program itself asks for a password, the answer is typed
without echo. Typed answers are read from a terminal only, never from a
pipe.

- `--yes` (or `-y`) skips the question.
- Without a terminal, for example in a script or a pipe, a change needs
  `--yes`. Without it nothing changes and the exit code is 4.
- A change that cannot be undone wants its name typed instead of `y`.
  `--yes` covers that too.

`Ctrl+c` while a change runs lets the running step end and skips the
rest: `warning: stopping after this step, press Ctrl+C again to stop it
now`. A second `Ctrl+c` stops the running step at once. Its cleanup still
runs, such as an unmount. Either way the command waits for the helper,
shows the steps that did not run as cancelled, ends with `Cancelled` and
exit code 130. After `queue apply`, what did not run stays in the queue.

`queue apply` checks a queue an earlier run left as the pages do. For a
queued lock, account removal or group change it reads the lock state of
the accounts through the helper, for a queued AUR build the package list
of the AUR. When one of them cannot be read, nothing runs and the exit
code is 5.

`SIGTERM` (as `kill` sends it) and `SIGHUP` (a closed terminal) count as
`Ctrl+c` does: the first lets the running step end, a second stops it.
The command then ends with 143 for `SIGTERM` or 129 for `SIGHUP`, 128 and
the number of the first signal that came, as a shell reports it.

### Services

For example:

```console
$ garuda-toolbox services restart sshd.service
$ garuda-toolbox services enable sshd.service --yes
$ garuda-toolbox services mask cups.service --user
```

| Command                   | What it does                                       |
| ------------------------- | -------------------------------------------------- |
| `services start <unit>`   | Starts the service now.                            |
| `services stop <unit>`    | Stops it now.                                      |
| `services restart <unit>` | Restarts it now.                                   |
| `services reload <unit>`  | Reloads its configuration now.                     |
| `services enable <unit>`  | Starts it at boot, as the switch on the page does. |
| `services disable <unit>` | Stops starting it at boot.                         |
| `services mask <unit>`    | Masks it, so nothing can start it.                 |
| `services unmask <unit>`  | Unmasks it.                                        |

`--user` changes one of your own services instead of the system's. As on
the Services page, starting a service that runs already or stopping one
that is not running changes nothing: the command says `Nothing to do`
and ends with 0, as a switch already as asked does.
Stopping, restarting or masking a service the session lives on (the login
screen, NetworkManager, polkit, the system bus, systemd-logind) wants its
name typed instead of `y`, or `--yes`, as the page asks first.

### Switches and apps

For example:

```console
$ garuda-toolbox toggle on pipewire-support
$ garuda-toolbox toggle off bluetooth-support --yes
$ garuda-toolbox apps install firefox
$ garuda-toolbox apps remove firefox
```

`toggle on <id>` and `toggle off <id>` work for every switch of
Components, System Tweaks, Gaming and the Setup Assistant. The id is the
one `toggle list` prints. A switch brings what it needs and turns off what
conflicts with it, as a click on the page does, and the page's rules
apply: a switch that needs another one that is off is refused with 1.
`apps install <id>` and `apps remove <id>` do the same for the App
Catalog, with the ids `apps list` prints. A switch that is already as
asked plans no step, says `Nothing to do.` and ends with 0.

### Packages

For example:

```console
$ garuda-toolbox packages search editor --json
$ garuda-toolbox packages show gimp
$ garuda-toolbox packages install gimp
$ garuda-toolbox packages remove gimp
```

`packages search` lists every repository and installed package whose name
or description holds the text, `--installed` the installed ones only,
`--orphans` the dependencies nothing needs anymore. `--json` prints an
array, `packages show --json` one object, with sizes in bytes and the
install date in seconds since 1970. `install` runs `pacman -S --needed`
for a package a repository offers, `remove` runs `pacman -Rns`, which
takes the dependencies nothing else needs along. A package that is
already installed, or not installed for `remove`, says `Nothing to do.`

### Kernels

For example:

```console
$ garuda-toolbox kernels install linux-lts
$ garuda-toolbox kernels headers linux
$ garuda-toolbox kernels remove linux-zen
```

`install` adds a kernel, used after the next restart. `remove` removes an
installed one, but never the running kernel. `headers` installs the
headers of an installed kernel, which DKMS needs to build modules for it.

### Maintenance

For example:

```console
$ garuda-toolbox maintenance list
$ garuda-toolbox maintenance run clean-package-cache
$ garuda-toolbox maintenance run remote-full-fix
```

`maintenance list` prints every task and repair of the Maintenance page
with its id, and marks those that are not available here and those that
ask first. `maintenance run <id>` runs one now, as its **Run now** button
does. The repairs that reinstall packages or throw settings away (marked
"asks first") want their id typed instead of `y`, or `--yes`.

### Users and groups

For example:

```console
$ garuda-toolbox users add grace --full-name "Grace Hopper" --admin
$ garuda-toolbox users edit ada --shell /usr/bin/fish --add-group docker
$ garuda-toolbox users edit ada --home /home/lovelace --move-files
$ garuda-toolbox users password ada
$ garuda-toolbox users lock guest
$ garuda-toolbox users remove guest --delete-home
$ garuda-toolbox groups add media
```

`users add` creates the account and sets its password at once. The
password is never an argument: it is asked twice on the terminal without
echo, or read from the first line of standard input with
`--password-stdin`. `--admin` puts the account into `wheel`, `--group`
into further groups. `users edit` changes the full name, the login shell
(one of `/etc/shells`), the groups and the home folder. `lock` and
`unlock` lock and unlock the password. `remove` keeps the home folder
unless `--delete-home`, which wants the account's name typed. The account
running the command can be neither locked nor removed, as on the page.

### Languages

For example:

```console
$ garuda-toolbox locales enable de_DE.UTF-8
$ garuda-toolbox locales system de_DE.UTF-8
$ garuda-toolbox locales packs list
$ garuda-toolbox locales packs install firefox-i18n-de
```

`enable` and `disable` switch a locale in `/etc/locale.gen`. The system's
own locale stays enabled. `system` makes a locale the system's language,
enabling it too. `packs` lists, installs and removes the translations of
installed programs for the enabled locales.

### Boot options and DKMS

For example:

```console
$ garuda-toolbox boot show
$ garuda-toolbox boot entries
$ garuda-toolbox boot set default gnulinux-simple-1
$ garuda-toolbox boot set timeout 3
$ garuda-toolbox boot set quiet off
$ garuda-toolbox boot set background none
$ garuda-toolbox boot set plymouth bgrt
$ garuda-toolbox boot param add nowatchdog
$ garuda-toolbox kernels default linux-lts
$ garuda-toolbox dkms list
$ garuda-toolbox dkms build --kernel linux-lts
```

`boot set` takes the page's settings: `default` (an id of `boot entries`),
`timeout` (0 to 300 seconds), `remember`, `submenus`, `os-prober`,
`quiet`, `splash`, `mitigations-off` and `sysrq` (`on` or `off`),
`background` and `theme` (a full path or `none`) and `plymouth` (a theme).
Each change writes `/etc/default/grub` and generates the boot menu again.
On systemd-boot, `boot set` writes `loader.conf` instead and takes
`default` (an id of `boot entries`, `saved` or `none`), `timeout` (seconds,
`menu-force`, `menu-hidden` or `none`), `remember`, and `editor`,
`auto-entries` and `auto-firmware` (`on`, `off` or `none`). The GRUB settings
are refused there, and the systemd-boot ones on GRUB. On systemd-boot,
`quiet`, `splash`, `mitigations-off`, `sysrq` and `boot param` change
`/etc/kernel/cmdline`, `plymouth` sets the theme, and both build the kernel
images again. `boot param remove` refuses `root=`, `rootflags=` and `rw`, and
`boot show` also names the Plymouth theme.
`kernels default` makes an installed kernel's entry the default. On systemd-boot
it takes the kernel's image, saved by its kind (`garuda-*-garuda.efi`), and
refuses a kernel without an image.
`dkms build` builds every module an installed kernel with headers lacks,
or only those of `--kernel`.

### Config files

For example:

```console
$ garuda-toolbox configs list
$ garuda-toolbox configs show pacman-config
$ EDITOR=nano garuda-toolbox configs edit pacman-config
```

`configs edit` opens the file in your editor (`$VISUAL`, else `$EDITOR`,
else micro, nano or vi), right in the terminal, as a private copy. After the
editor ends, the changes are printed and saved once you agree: a file of the
system through the password dialog, a file of your home folder with your own
rights and its previous version kept in
`~/.local/state/garuda-toolbox/backups`. A file that changed meanwhile is
not overwritten. Files the page only shows are never edited.

### Network

For example:

```console
$ garuda-toolbox network show
$ garuda-toolbox network radio wifi off
$ garuda-toolbox network drivers blacklist r8169
$ garuda-toolbox network public-address
$ garuda-toolbox network ping garudalinux.org --count 4
$ garuda-toolbox network traceroute garudalinux.org
```

`radio` switches Wi-Fi, Bluetooth or flight mode at once.
`drivers blacklist` keeps a driver from loading at the next start,
`drivers allow` lets it load again. `public-address` asks geoip.kde.org,
as the page's **Show public address** does, and nothing is kept. `ping` and
`traceroute` run in the terminal and change nothing.

### Boot Repair

For example:

```console
$ garuda-toolbox bootrepair show
$ garuda-toolbox bootrepair regenerate /dev/nvme0n1p2
$ garuda-toolbox bootrepair reinstall /dev/nvme0n1p2 --target /dev/nvme0n1p1
```

On the live ISO, `show` lists the installed systems and boot targets
found. `regenerate` generates a system's boot menu again, `reinstall`
installs GRUB again onto the EFI system partition (or the disk on a BIOS
machine), the usual one without `--target`. A locked system's passphrase
is asked without echo, or read from standard input with
`--passphrase-stdin`. A disk the running system uses is never touched.
`--boot-repair` lets it run on an installed system, for testing.

### Setup Assistant and settings

For example:

```console
$ garuda-toolbox setup list
$ garuda-toolbox setup apply firefox strawberry-player
$ garuda-toolbox settings list
$ garuda-toolbox settings set update-check 30
$ garuda-toolbox settings set start-at-login on
$ garuda-toolbox settings set dark-scheme nord
```

`setup apply` turns several of the assistant's choices on in one plan, as
its summary does. `settings` reads and writes the app's own settings:
`language` (a code or `system`), `start-at-login`, `update-check` (minutes
or `never`), `user-services`, `copy-reports`, `log-level`, `dark-scheme`
and `light-scheme` (a colour scheme of that mode, such as `nord` or
`catppuccin-latte`, or `system` for the desktop's colours).
`settings get <setting>` prints one value, with `--json` as the object
`settings list --json` holds for it, such as
`{"schema": 1, "id": "language", "value": "system"}`. The window and the
command line share one settings file. The window takes a change made here
at its next start. Close a running window first: a change made in its
Settings page writes its own values over the file. Theme and motion are
kept with the window and only changed there.

## Output options

These go after the command, as `-v` does:

| Option                        | What it does                                                                                  |
| ----------------------------- | --------------------------------------------------------------------------------------------- |
| `--color auto\|always\|never` | Colour. `auto` colours a terminal, `always` colours a pipe too and `never` leaves colour out. |
| `--width <columns>`           | Cuts long cells to this width, even in a pipe.                                                |
| `-q`, `--quiet`               | Leaves out the spinner and the `hint:` lines. Errors stay.                                    |

Without `--color`, `CLICOLOR_FORCE` or `FORCE_COLOR` set to a value other
than `0` colours a pipe. `NO_COLOR`, `CLICOLOR=0` and `TERM=dumb` turn
colour off on a terminal, unless one of the two force variables asks for
it. A terminal's width comes from `COLUMNS` when it is set, else from the
window size. A pipe gets long cells whole, as before.

A change ends with `Done`, or `Test mode, nothing was changed`, and the time
the whole run took, such as `Done  (12.6 s)`.

`garuda-toolbox completions <shell>` prints the completion hook for bash,
elvish, fish, powershell or zsh, the same one the package installs. The hook
asks `garuda-toolbox` for the names at each `Tab`, so a saved copy stays
current:

```console
$ garuda-toolbox completions bash > ~/.local/share/bash-completion/completions/garuda-toolbox
$ garuda-toolbox completions zsh > ~/.zfunc/_garuda-toolbox
```

`garuda-toolbox help <command>` prints the long help of one command, such
as `help users add`, coloured as `--help` is. Completion of the system's
names keeps them for ten seconds in `$XDG_CACHE_HOME`, so a `Tab` right
after another one is quick. The AUR's names stay there until its copy changes.

## Environment variables

The man page lists these too:

| Variable                        | What it does                                                                                     |
| ------------------------------- | ------------------------------------------------------------------------------------------------ |
| `GARUDA_TOOLBOX_PAGER`          | The pager for lists and shows, ahead of `PAGER`. Empty or `cat` prints directly.                 |
| `PAGER`                         | The pager when `GARUDA_TOOLBOX_PAGER` is unset. Without either, `less -FRX`.                     |
| `LESS`, `LV`                    | Given to the pager as `FRX` and `-c` when unset, so the colours stay.                            |
| `NO_COLOR`                      | Any value but an empty one leaves colour out.                                                    |
| `CLICOLOR`                      | `0` leaves colour out.                                                                           |
| `CLICOLOR_FORCE`, `FORCE_COLOR` | A value other than `0` colours a pipe too, and wins over `NO_COLOR`, `CLICOLOR` and `TERM`.      |
| `TERM`                          | `dumb` leaves colour and the spinner out.                                                        |
| `COLUMNS`                       | The terminal's width, which long cells are cut to. `--width` comes first.                        |
| `VISUAL`, `EDITOR`              | The editor of `configs edit`, see [Config files](config-files.md).                               |
| `XDG_CACHE_HOME`                | Where completion keeps the system's names, `~/.cache` without it.                                |
| `COMPLETE`                      | Set by the completion hooks: the program answers the completion request of that shell and exits. |
| `GARUDA_TOOLBOX_LOG`            | Replaces the log level of the settings, in the filter syntax of tracing, such as `debug`.        |

`--color always` and `--color never` win over every colour variable.

## For scripts

Command line output is English, whatever the language of the session.
`--json` carries ids, numbers and English words only, so a script reads
the same values on every system.

`--json` prints the same facts as the text, as JSON. Every object starts
with `"schema": 1`, the version of its field names. A later version that
renames a field raises it. The field names follow one pattern:

- `id` is what a row is known by, the name the commands take: a unit, a
  switch, an app, a package, a kernel, a locale, an account, a group, a
  setting, a page, a guide page, a DKMS module, a driver, a configuration
  file, a disk of Boot Repair, a change in the queue or a check of
  `doctor`, such as `polkit-agent`.
- `name` is the English display name of a row that has one besides its
  id, such as a switch, a locale, a page, a configuration file, an
  account (its full name, empty when it has none) or a check of `doctor`.
- `state` names a row's current condition in one word, such as
  `installed` for a DKMS module (dkms's own words). A fact of `status`
  has a `state` too, which tells whether it could be read: `ok` with its
  `value`, `unavailable` when this system does not have it, or `failed`
  with its `cause`, the words of what failed.
- States that need more than one word, or a yes or no, have names of
  their own: `on` for switches and apps (`true`, `false`, or `null` when
  it could not be read), `active` and `sub_state` for services
  (systemd's own words, such as `active` and `running`) and `result` for
  the checks of `doctor` (`ok`, `problem`, `optional` or `unknown`).

For example, `status --json`:

```json
{
  "schema": 1,
  "updates": {
    "state": "ok",
    "value": [{ "id": "firefox", "from": "143.0-1", "to": "144.0-1" }]
  },
  "aur_updates": { "state": "unavailable" },
  "files_to_merge": { "state": "ok", "value": [] },
  "last_update_days": { "state": "ok", "value": 3 },
  "health": { "state": "ok", "value": [] },
  "restart_needed": { "state": "ok", "value": false }
}
```

A list prints one object with its rows in `rows`, an array, also with one
row or none. `doctor` is a list of its checks. Everything else, such as
`status`, the `show` commands and `settings get`, prints one object with
its fields. `network public-address` prints the address in `address`. For
example `services list --json`:

```json
{
  "schema": 1,
  "rows": [
    {
      "id": "sshd.service",
      "active": "active",
      "sub_state": "running",
      "at_boot": true,
      "description": "OpenSSH Daemon"
    }
  ]
}
```

A check of `doctor --json`:

```json
{
  "id": "polkit-agent",
  "name": "polkit agent",
  "result": "unknown",
  "text": "none found: the window may not be able to ask for the password, the command line asks in the terminal",
  "hint": "start your desktop's polkit agent, such as polkit-kde-agent or polkit-gnome"
}
```

### Errors

An error is one line on standard error that starts with `error:`, and a
`hint:` line under it when something helps. Every command, a wrong option
or page included, says it this way, except under `--json`. `--quiet`
leaves the hint out. For example:

```text
error: `nowhere` is no page
hint: `garuda-toolbox pages` lists the page ids
```

A failed read under `--json` keeps the `error:` and `hint:` lines out.
Standard output holds the JSON alone or nothing, and standard error one
JSON object per error, each on a line of its own, once the exit code is
known:

```json
{
  "schema": 1,
  "error": {
    "message": "not available: the accounts could not be read",
    "code": 5,
    "reason": "unavailable"
  }
}
```

`message` says what went wrong, `hint` what helps and `detail` holds the
program's own words. The last two are left out when there are none.
`code` is the exit code the command ends with and `reason` its name (see
[Exit codes](#exit-codes)). A change under `--json` prints its errors as
JSON Lines on standard output instead, as the next section describes.

The same object stands for an error that comes before any command runs,
when `--json` is among the arguments: a wrong option or argument (`code`
2, `reason` `usage`, clap's tip as the `hint`) or a wrong start option of
the window. When the arguments cannot be parsed, `--json` is looked for
in them as written, up to a `--`. Lost output, such as a full disk behind
a redirect, gives the object too, also for a change, since its standard
output is gone. `--help` and `--version` print as always.

### JSON Lines

A change under `--json` wants `--yes` or `--dry-run`, since it cannot
ask. It prints one JSON object per line on standard output and nothing
else there. Its notes, such as the one after `Ctrl+c`, go to standard
error. Each line has `"schema": 1` and an `event`:

| `event`    | Fields                                            | When                                                         |
| ---------- | ------------------------------------------------- | ------------------------------------------------------------ |
| `plan`     | `steps`: an array of `{"step", "command"}`        | First, the whole plan. `steps` is empty when nothing is due. |
| `started`  | `step`, `of`, `command`                           | A step starts.                                               |
| `output`   | `step`, `line`                                    | A line of the program's output, without its line break.      |
| `finished` | `step`, `outcome`, `seconds`                      | A step ended.                                                |
| `error`    | `message`, `hint` and `detail` when there are any | Something went wrong, such as a closed password dialog.      |
| `done`     | `ok`: `true` when every step ended well           | Last.                                                        |

`step` counts from 1 in the order of the plan, `of` is the number of
steps and `command` the step's command line as text. `outcome` is
`done`, `failed`, `cancelled` or `skipped` (a dry run of the helper, or
a step with nothing to do). `seconds` is how long the step took, to the
millisecond. `hint` says what helps and `detail` holds the program's own
words, as in the errors of a read.

- `--dry-run` prints the `plan` line alone.
- A change that would do nothing prints a `plan` without steps and
  `done` with `ok` `true`, and ends with 0.
- A change refused before its plan, such as an unknown name or a click
  the page refuses, prints one `error` line and no `done`.
- A step that asks questions while it runs, such as paru's, prints an
  `error` line and ends with 4: leave out `--json` and answer them in a
  terminal.

For example, a restart of `sshd.service`:

```json
{"schema":1,"event":"plan","steps":[{"step":1,"command":"systemctl restart -- sshd.service"}]}
{"schema":1,"event":"started","step":1,"of":1,"command":"systemctl restart -- sshd.service"}
{"schema":1,"event":"finished","step":1,"outcome":"done","seconds":0.412}
{"schema":1,"event":"done","ok":true}
```

`status --watch --json` prints JSON Lines too: each line is one object of
the shape `status --json` has, `schema` included.

### Exit codes

The codes and their reasons keep their meaning in later versions. The
reason is the `reason` of a read's JSON error.

| Code | Reason        | Meaning                                                                                                                                                                           |
| ---- | ------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 0    | `success`     | Done, also when there was nothing to do, such as starting a running service or a switch already as asked. `status` and `doctor`: nothing needs attention.                         |
| 1    | `failure`     | A change or a read failed.                                                                                                                                                        |
| 2    | `usage`       | The arguments were wrong, such as a name that is not well formed.                                                                                                                 |
| 3    | `declined`    | You answered no.                                                                                                                                                                  |
| 4    | `needs-yes`   | No terminal to ask on: pass `--yes`.                                                                                                                                              |
| 5    | `unavailable` | Not available on this system, such as a service, kernel or locale this system does not have.                                                                                      |
| 6    | `denied`      | The password dialog or the helper turned it down.                                                                                                                                 |
| 8    | `page-rule`   | Refused by the page's rules, such as removing root or a password that does not match. The page refuses the same click.                                                            |
| 9    | `attention`   | `status` found something that needs attention: updates, files to merge, health, a restart, low disk space, a failed service, a clean-up. `doctor` found a part missing or broken. |
| 129  | `hung-up`     | `SIGHUP` stopped it, such as a closed terminal. A change stops as with `Ctrl+c`.                                                                                                  |
| 130  | `interrupted` | `Ctrl+c` stopped it.                                                                                                                                                              |
| 143  | `terminated`  | `SIGTERM` stopped it, such as `kill`. A change stops as with `Ctrl+c`.                                                                                                            |

`status` ends with 0 also when some facts could not be read, and `status
--watch` when `Ctrl+c`, `SIGTERM` or `SIGHUP` ends it. 7 and 10
are no longer given: 7 meant a change that would do nothing, which ends
with 0, and 10 a cancelled change, which ends with 130, 143 or 129.
