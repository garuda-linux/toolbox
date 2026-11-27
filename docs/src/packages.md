# Packages

**Packages** lists every package of the repositories in `/etc/pacman.conf`
and every installed package. The details of the package you pick show
beside the list. Use it to install or remove a package by name. For a
curated pick of apps, see [App Catalog](discover.md).

## Before you start

The page reads pacman's package databases as they are. It never syncs
them, so it is as fresh as the last system update. When the databases
cannot be read, the page says so and offers **Read Again**. While the AUR
is turned on, the **AUR** chip is then picked and still works. The other
chips are greyed out and show no number.

## Find a package

The chips above the list pick what it shows:

| Chip          | What it lists                                                                                                                                       |
| ------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Installed** | Every installed package. This is how the page opens                                                                                                 |
| **Updates**   | Installed packages with a pending update, the same ones **Home** counts                                                                             |
| **Orphans**   | Packages installed as a dependency that no installed package needs anymore, not even as an optional dependency, as `pacman -Qdtt` lists them        |
| **Optional**  | Packages installed as a dependency that installed packages only list as an optional dependency. With **Orphans** they make what `pacman -Qdt` lists |
| **All**       | Every package of the repositories, plus the installed ones they lack                                                                                |
| **AUR**       | The packages of the AUR, only while the AUR is turned on in **Settings**                                                                            |

**Updates** holds the repository updates and the AUR updates that **Home**
found, so both show the same number. Until the check for updates has
answered, the chip shows no number and its list says **Checking for
updates…**. When the check failed, the list says **Updates could not be
checked.** with **Read Again**.

To narrow the list, type in **Search packages** at the top. The search
looks at names and descriptions under the chip that is picked. While you
search, the number on each chip counts its matches. When nothing matches,
the page says so and offers **Clear Search**.

Every row shows the package's icon, its name, its tags and one line of its
description. In the full-width list a second small line shows its
repository, version and installed size, such as `extra · 3.0-1 · 12.3 MiB`
(the installed version for an installed package). While the details of a
package are open, the narrow list beside them leaves that line out, and the
details show the same facts. Next to the name:

- **Dependency** marks a package pulled in by another one.
- **Not from a repository** marks an installed package no configured
  repository offers, such as one built from the AUR.
- **Optional for** names the installed package that lists it as an optional
  dependency while none requires it. With several, the tag says how many
  more and its tooltip names them all.
- **Installed**, **Installs with Apply** or **Removes with Apply** tells its
  state.

## Install a package

1. Find the package and click its row. Its details open beside the list.
2. Click **Install** in the details. The row and the details say
   **Installs with Apply**. Click **Take Back** to take it out of the
   queue.
3. Apply the queue, see [Review and apply changes](applying.md).

When it worked, the row says **Installed** and the queue is empty. pacman
installs the package with `pacman -S --needed`.

## Remove a package

1. Open the package and click **Remove** in its details.
2. A question names the dependencies that go along: the ones the package
   pulled in that nothing else needs. Click **Remove** to queue the
   removal, or **Cancel**.
3. Apply the queue.

> **Note:** pacman removes the package with `pacman -Rns`, which also
> removes those dependencies and the package's saved configuration files. The
> question is an estimate. The Console lists what pacman really removes
> before it does. Install the package again to get it back. When pacman
> would also remove a package the system needs, the step fails and
> nothing is removed.

## Packages from the AUR

The Arch User Repository (AUR) holds build scripts that people of the
community share. Its packages are not checked by Arch or Garuda. They are
built on your computer from those scripts.

The AUR is off at first. While it is off, the page says **AUR packages are
hidden.** with a link **Turn them on in Settings**. Turn on **Show AUR
packages** in [Settings](settings.md) and the **AUR** chip appears. On the
command line: `garuda-toolbox settings set aur on`.

The AUR publishes a file of every package once a day. The first time you
pick the **AUR** chip, Garuda Toolbox downloads that file (about 14 MB) and
keeps a copy in `~/.cache/garuda-toolbox/aur/`. It downloads it again at
most once a day: when the chip or an `--aur` command reads the list and the
last try is a day old. A failed download also waits a day. Nothing else is
asked from the AUR, and nothing is downloaded while the AUR is off.

Each AUR row shows the name with an **AUR** tag and one line of the description.
Its version and votes show in the details. **Out of date** marks a package
someone flagged. The search looks at names, descriptions and keywords. The exact
name comes first, then names that start with what you typed, then the rest, each
part by popularity.

The details of an AUR package show the maintainer, votes, popularity,
dates, licenses, the website and a link to the package's **AUR page** at
the left, and what it needs to run and to build at the right. An installed
AUR package also shows its files and history.

To install a package from the AUR:

1. Click the package in the list, then **Install** in its details.
2. A question says the package is built from a community script on your
   computer. Click **Install** to queue it, or **Cancel**.
3. The row says **Builds with Apply**. Apply the queue.

Garuda Toolbox builds the package as your own user, never as root. Before
anything is built, the Console shows the build files of the package, its
PKGBUILD and the other files that are not downloads, and waits. Read them.
Type `y` and press Enter, or click **Continue** in the Console, to go on.
Any other answer, or **Cancel**, stops the run and nothing is built. The
packages are installed with your password, asked once for the whole run.

> **Note:** Only the build files that are kept in git are shown. Sources
> that the build downloads later are not shown.

On a setup that cannot use this path, paru builds the package instead. It
shows the build script itself and asks for your password for each step, so
it can ask several times. If **Apply** waits for a build order, see
[Apply waits for a build order](troubleshooting.md#apply-waits-for-a-build-order).

An installed AUR package is removed like any other package, see
[Remove a package](#remove-a-package).

### Install from the AUR on the command line

Run `garuda-toolbox packages install --aur octopi-git` to build a package from the AUR.

The command reads the build order of each package, so it can show what gets
built and installed. Then it prints the plan. Its first step shows the build
files of each package, the PKGBUILD or what changed since the last build, and
asks `Build octopi-git?`. Type `y` and press Enter to build. Anything else
cancels, nothing is installed and the command ends with exit code 3. `--yes`
does not skip this question, so a script needs a terminal. `--json` refuses
such a plan, as it cannot ask.

A package that conflicts with an installed one is refused before anything is
queued, with a message such as `octopi-git conflicts with the installed
paru-bin. Remove paru-bin first.` Remove the installed package, then install
again.

Some setups cannot use the build folder that the helper checks, such as a
home folder that is a link or an account without an entry in `/etc/passwd`.
Then the command says `this setup uses the old paru path:` and the reason.
paru builds the package and asks for your password itself, as before, and the
review above does not happen.

The command line uses the old paru path when it asks for the password through
`sudo`, because `sudo` does not tell the helper who asked. Use the window or a
terminal where the password dialog of pkexec works for AUR packages.

## See a package's details

Click a row to show its details beside the list. The selected row is
highlighted, and the details change with each step of the arrow keys. The
details hold the package's button: **Install** when it is not installed,
**Remove** when it is, and **Take Back** while a change waits in the
queue.

On a narrow window the details take the whole page, and **← Packages** goes
back to the list.

The details come in cards:

- At the left: the description, then the repository, installed size,
  licenses, groups, install date and install reason, the website, and
  what it provides, conflicts with and replaces. Empty lines are left
  out.
- At the right: what it depends on, the installed packages that require it
  and its optional dependencies with what each adds, each with its count.
  A long list shows its first 12 names. **+N more** shows the rest.
- **Files:** folded at first, with the number of files. **Copy List**
  puts every path on the clipboard, one per line. Installed packages only.
- **History:** the newest 5 changes pacman made to the package. **Show
  all** lists every one the log still holds.

In a narrow window the right card moves under the left one.

Click a name in a dependency list to open that package. The back button above
the details then names the package you came from, so **←** walks back through
the chain. Once the chain is walked, **Esc** closes the details, and on a narrow
window the list shows again. A name that no repository offers and no installed
package provides shows as plain text.

The website opens in your browser, only when it is an `http` or `https`
address. The installed version has a copy button when it differs from the
repository's.

**History** reads the newest few megabytes of `/var/log/pacman.log`. An
older change of the package does not show there.

## When an install fails with a 404

A mirror drops a package file once a newer build replaces it. When your
package databases are older than the mirror, pacman asks for the old file
and the mirror answers 404 (`failed retrieving file`). The Console then
shows a warning under the steps with **Update the System and Try Again**.
It runs one plan: a full system update, which syncs the databases, then
the same install. You type your password once.

## Use the keyboard

| Key             | What it does                                                                                       |
| --------------- | -------------------------------------------------------------------------------------------------- |
| `Tab`           | Moves between the search, the chips and the list, and through the links and buttons of the details |
| `Up` and `Down` | Move through the list, and show the details of the row they reach                                  |
| `Space`         | Opens the details of the selected package                                                          |
| `Enter`         | Opens the package of a focused name, or presses a focused button                                   |
| `Esc`           | Goes back one package, then closes the details (on a narrow window, shows the list)                |
| `F5`            | Reads the packages again (**Read Again**)                                                          |

## On the command line

To search, show, install and remove packages from a terminal, for example:

```console
$ garuda-toolbox packages search editor
$ garuda-toolbox packages search --orphans --json
$ garuda-toolbox packages show gimp
$ garuda-toolbox packages install gimp
$ garuda-toolbox packages remove gimp
$ garuda-toolbox packages search --aur paru
$ garuda-toolbox packages show --aur paru-bin --json
$ garuda-toolbox packages install --aur paru-bin
```

The `--aur` commands work while **Show AUR packages** is off, because the
flag itself asks for the AUR. They read the same list as the **AUR** chip
and download it at most once a day.

See [The command line](cli.md).

## See also

- [App Catalog](discover.md)
- [Review and apply changes](applying.md)
- [The Console](console.md)
- [Maintenance](maintenance.md)
