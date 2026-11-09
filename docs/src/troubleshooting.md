# Troubleshooting

These are the messages Garuda Toolbox shows when something does not work,
with why they appear and what to do. Each page's own guide lists the
messages that only that page shows.

## No password dialog could be shown

**Symptoms**

- After **Apply**, the Console says: "No password dialog could be shown.
  Log in to a desktop session with a polkit agent."

**Cause**

Changes that need administrator rights ask for a password through polkit.
Plasma, GNOME and most desktops start the program that shows this dialog.
Some window managers and remote sessions do not.

**Solution**

1. Install an authentication agent, for example `polkit-kde-agent` or
   `polkit-gnome`.
2. Start it with your session, for example from your window manager's
   autostart file.
3. Apply the changes again. They stay queued until then.

On the [command line](cli.md) the app asks through `sudo` instead when no
password dialog can be shown.

## The password dialog was closed or the password was not accepted

**Symptoms**

- After **Apply**, the Console says: "The password dialog was closed or the
  password was not accepted. Nothing was changed."

**Cause**

One of:

- **Cancel** was pressed in the password dialog of KDE Plasma, which
  reports a cancel the same way as a wrong password.
- The password was wrong or does not belong to an administrator.
- Your account is in neither the `wheel` nor the `sudo` group.

**Solution**

1. Apply the changes again and enter the password.
2. If it is still refused, ask an administrator to enter their password in
   the dialog, or to add your account to `wheel` on the
   [Users and Groups](users.md) page.

## The password dialog was closed

**Symptoms**

- The Console says: "The password dialog was closed. Nothing was changed."

**Cause**

The dialog was cancelled or closed.

**Solution**

1. Click **Apply** on the changes pill again and enter the password.

## A step failed

**Symptoms**

- The Console says **Stopped before all changes were applied**, and the
  failed step says **Failed** with its exit code.
- A desktop notification says: "A step failed. The Console shows what
  happened."

**Cause**

A program of that step reported an error. Its own message is in the
Console's terminal. A common one from pacman is `unable to lock database`:
another package manager is running, or an earlier run was stopped while it
held the lock.

**Solution**

1. Read the program's message in the [Console](console.md).
2. For `unable to lock database`, wait until any other package manager or
   update has finished. When none runs and the message stays, use **Remove
   the pacman lock** on [Maintenance](maintenance.md). It refuses while
   pacman runs.
3. Apply the remaining changes again. Steps that already ran stay applied.

## Something could not be read

**Symptoms**

- A message at the top of a page says a part could not be read, for
  example "The GRUB settings could not be read".
- The status bar at the bottom of the window says **Some parts of the
  system could not be read** and names them.

**Cause**

A program or file the page reads failed. The page shows that part as not
known. In a session without a user session bus, for example over plain
SSH, user services cannot be read at all.

**Solution**

1. Point at or focus the **i** button after the message in the status bar.
   It lists each part with the error the system gave, for example
   `/etc/kernel/install.conf: Permission denied`.
2. Fix the cause, then click **Read again** in the message, or press `F5`.
3. The round arrow in the status bar reads the whole system again.

When the boot loader itself cannot be read, the Boot Options page shows
neither GRUB nor systemd-boot settings until the read works.

## Something is not available on this system

**Symptoms**

- A page or a section says **not available on this system**, for example
  "Kernels are not available on this system".

**Cause**

The system lacks what the page works with: pacman's package databases,
GRUB, NetworkManager or a Garuda tool on a system that is not Garuda Linux.
This is no error.

**Solution**

1. Nothing to do when you do not need that page. Each page's guide names
   what it needs.

## Could not check on Home

**Symptoms**

- On [Home](home.md), a figure says **Could not check**, and a message says
  how many checks could not run. **Show details** names each check, for
  example the repository updates, and the reason.

**Cause**

One of the status checks failed, for example `checkupdates` without an
internet connection.

**Solution**

1. Click **Read again** beside **System status** once the network works.
2. When it keeps failing, open [Diagnostics](diagnostics.md) and collect
   the system information for a forum post.

## A button does nothing or the app closed

**Symptoms**

- A click has no effect and a page shows old data, or the window closed on
  its own.

**Cause**

A bug in Garuda Toolbox. The app writes every such error with the place in
the code to `~/.local/state/garuda-toolbox/crash.log`.

**Solution**

1. Close and reopen the page or the app.
2. Report the problem in the Garuda Linux forum and attach `crash.log`.
   Only you can read the file, as it may contain paths of your home folder.

## Garuda Toolbox is not installed completely

**Symptoms**

- The window shows only **Garuda Toolbox is not installed completely** and
  **Quit**.
- `garuda-toolbox doctor` reports `/usr/share/garuda-toolbox/catalog is
missing`.

**Cause**

The folder with the catalogs of apps, components and settings is missing.
Without it every page would be empty.

**Solution**

1. Reinstall the package: `sudo pacman -S garuda-toolbox`.
2. Start the app again.

Commands that need no catalog, such as `garuda-toolbox status`, keep
working meanwhile.

## See also

- [Review and apply changes](applying.md)
- [The Console](console.md)
- [Diagnostics](diagnostics.md)
