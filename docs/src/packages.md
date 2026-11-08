# Packages

**Packages** lists every package of the repositories in `/etc/pacman.conf`
and every installed package, with a page of details for each one. Use it
to install or remove a package by name. For a curated pick of
apps, see [App Catalog](discover.md).

## Before you start

The page reads pacman's package databases as they are. It never syncs
them, so it is as fresh as the last system update. When the databases
cannot be read, the page says so and offers **Read again**.

## Find a package

The chips above the list pick what it shows:

| Chip          | What it lists                                                              |
| ------------- | -------------------------------------------------------------------------- |
| **Installed** | Every installed package. This is how the page opens                        |
| **Updates**   | Installed packages with a pending update, the same ones **Home** counts    |
| **Orphans**   | Packages installed as a dependency that no installed package needs anymore |
| **All**       | Every package of the repositories, plus the installed ones they lack       |
| **AUR**       | The packages of the AUR, only while the AUR is turned on in **Settings**   |

**Updates** holds the repository updates and the AUR updates that **Home**
found, so both show the same number.

To narrow the list, type in **Search packages** at the top. The search
looks at names and descriptions under the chip that is picked. While you
search, the number on each chip counts its matches. When nothing matches,
the page says so and offers **Clear search**.

Every row shows the package's icon, its name, one line of its description,
then its repository, version and installed size. Next to the name:

- **Dependency** marks a package pulled in by another one.
- **Not from a repository** marks an installed package no configured
  repository offers, such as one built from the AUR.
- **Installed**, **Installs with Apply** or **Removes with Apply** tells its
  state.

## Install a package

1. Find the package, then click **Install** in its row.
2. The row says **Installs with Apply**. Click **Take back** to take it out
   of the queue.
3. Apply the queue, see [Review and apply changes](applying.md).

When it worked, the row says **Installed** and the queue is empty. pacman
installs the package with `pacman -S --needed`.

## Remove a package

1. Click **Remove** in the package's row or on its details.
2. A question names the dependencies that go along: the ones the package
   pulled in that nothing else needs. Click **Remove** to queue the
   removal, or **Cancel**.
3. Apply the queue.

> **Note:** pacman removes the package with `pacman -Rns`, which also
> removes those dependencies and the package's saved configuration files. The
> question is an estimate. The Console lists what pacman really removes
> before it does. Install the package again to get it back.

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

Each AUR row shows the name with an **AUR** tag, one line of the
description, the version and the votes. **Out of date** marks a package
someone flagged. The search looks at names, descriptions and keywords. The
exact name comes first, then names that start with what you typed, then the
rest, each part by popularity.

The details of an AUR package show the maintainer, votes, popularity,
dates, licenses, the website and a link to the package's **AUR page** at
the left, and what it needs to run and to build at the right. An installed
AUR package also shows its files and history.

To install a package from the AUR:

1. Click **Install** in its row or on its details.
2. A question says the package is built from a community script on your
   computer. Click **Install** to queue it, or **Cancel**.
3. The row says **Builds with Apply**. Apply the queue.

paru builds the package in the Console as your own user, never as root.
It shows the build script (the PKGBUILD) first and asks before it
builds, so read it and answer in the Console. It asks for your password
itself when it installs the result.

An installed AUR package is removed like any other package, see
[Remove a package](#remove-a-package).

## See a package's details

Click a row, or select it and press `Space`, to open its details on one
page. **← Packages** (or `Esc`) goes back to the list.

- At the left: the description, then the repository, installed size,
  licenses, groups, install date and install reason, the website, and
  what it provides, conflicts with and replaces. Empty lines are left
  out.
- At the right: what it depends on, the installed packages that require it
  and its optional dependencies with what each adds, each with its count.
  A long list shows its first 12 names. **+N more** shows the rest.
- **Files:** folded at first, with the number of files. **Copy list**
  puts every path on the clipboard, one per line. Installed packages only.
- **History:** the newest 5 changes pacman made to the package. **Show
  all** lists every one the log still holds.

In a narrow window the right column moves under the left one.

Click a name in a dependency list to open that package. The back button
then names the package you came from, so **←** walks back through the
chain to the list. A name that no repository offers and no installed
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
shows a warning under the steps with **Update the system and try again**.
It runs one plan: a full system update, which syncs the databases, then
the same install. You type your password once.

## Use the keyboard

| Key             | What it does                                                                                       |
| --------------- | -------------------------------------------------------------------------------------------------- |
| `Tab`           | Moves between the search, the chips and the list, and through the links and buttons of the details |
| `Up` and `Down` | Move through the list                                                                              |
| `Space`         | Opens the details of the selected package                                                          |
| `Enter`         | Opens the package of a focused name, or presses a focused button                                   |
| `Esc`           | Goes back one package, then to the list                                                            |
| `F5`            | Reads the packages again (**Read again**)                                                          |

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
