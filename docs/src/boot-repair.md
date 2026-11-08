# Boot Repair

**Boot Repair** helps when an installed system no longer starts. From the
live system it installs the boot loader of that system again, GRUB or
systemd-boot, or builds its boot files again: GRUB's boot menu, or
systemd-boot's kernel images. Which of the two the system has, the app sees
once it has mounted the system, so the page and its dialogs name both.

> **Warning:** A repair writes a boot loader to a disk. It replaces the boot
> loader that is there now, which cannot be undone from this page. Read the
> dialog before you confirm: it names the partition and the disk that are
> written. Use the page only for a system that does not start, and keep a
> backup of what you cannot lose.

![The Boot Repair page with its target pickers](images/boot-repair.png)

## Before you start

- Start the computer from the Garuda live medium. **Boot Repair** shows in the
  sidebar of the live system only. On an installed system it shows only
  with `--boot-repair`, see [Test mode](#test-mode).
- Start the live medium the same way the installed system starts. A system
  that was installed through UEFI needs the live medium started through UEFI,
  and a system installed the legacy way needs it started the legacy way. The
  page shows which one it found under **Started through**.
- The live system needs `lsblk`, `grub-install` and `cryptsetup`. The Garuda
  live medium has them. Without one of them the page shows **Boot repair
  cannot work on this system**, names the missing programs and offers nothing.
- For an encrypted system you need its passphrase.

The page repairs systems that boot with GRUB and keep `/boot` on the root
partition, and systems that boot with systemd-boot. These are refused before
anything is written:

- a GRUB system with `/boot` on a partition of its own,
- a GRUB system that mounts its EFI system partition somewhere else than
  `/boot/efi`,
- a systemd-boot system whose `/etc/fstab` mounts no EFI system partition at
  `/efi`, `/boot` or `/boot/efi`,
- a systemd-boot system from a live medium started the legacy way:
  systemd-boot only starts through UEFI.

## Choose the system

1. Open **Boot Repair** in the sidebar.
2. Pick the partition of the system under **Root partition**. The list holds
   every partition with a Linux file system and every encrypted partition,
   each with its label, size and file system.

Below the list the page shows what it found:

| Field                     | Meaning                                                                                                 |
| ------------------------- | ------------------------------------------------------------------------------------------------------- |
| **Started through**       | **UEFI** or **BIOS (legacy)**: how the live system was started, and so how the boot loader is installed |
| **Encryption**            | **Not encrypted**, **Encrypted (LUKS), locked** or **Encrypted (LUKS), unlocked**                       |
| **Disk**                  | The disk the partition lies on                                                                          |
| **Where the system lies** | Only for btrfs and for locked partitions, see below                                                     |

Garuda installs into the btrfs subvolume `@`, so **Where the system lies** is
set to **In the subvolume @ (Garuda's layout)**. Choose **At the top of the
file system** only for a system that was installed on btrfs without
subvolumes, or for an encrypted partition that holds another file system.

**Read again** (the round arrow at the top, or `F5`) reads the disks again, for
example after you plugged in a disk or unmounted a partition.

## Reinstall the boot loader

Use this when the computer does not find a boot loader at all, shows a
`grub rescue>` prompt, or lost its boot entry after a firmware update.

1. Choose where the boot loader goes:
   - started through UEFI: the partition under **EFI system partition**,
   - started the legacy way: the disk under **Disk for the boot loader**.
     Choose the disk the computer starts from.
2. Click **Reinstall the boot loader…**.
3. Read the dialog. It names the partition or disk the boot loader is written
   to and the system it is installed for.
4. For a locked system, enter its passphrase.
5. Click **Reinstall the boot loader** and enter an administrator's password.

On a GRUB system the repair installs GRUB and then writes the boot menu
again. Its boot entry in the firmware's boot menu is called `Garuda`. On a
systemd-boot system it runs `bootctl install`, which adds the entry
`Linux Boot Manager`, then builds the initramfs and the kernel images again.
A legacy start installs GRUB only.

## Build the boot files again

Use this when the boot loader starts but its menu is empty, lacks a kernel
you installed or still lists one you removed.

1. Click **Build the boot files again…**.
2. Read the dialog and, for a locked system, enter its passphrase.
3. Click **Build the boot files again** and enter an administrator's
   password.

On a GRUB system only `/boot/grub/grub.cfg` of the chosen system is written.
On a systemd-boot system the initramfs and the kernel images are built again
onto the EFI system partition on the system's own disk, which the dialog
names. The boot loader itself stays as it is.

## What a repair does

Every repair runs at once in the [Console](console.md), which shows each
command. Nothing is added to the queue.

1. The app checks the devices: each must be a real disk or partition of the
   kind the repair needs, and none may lie on a disk the live system uses.
2. An encrypted partition is unlocked with your passphrase.
3. The system is mounted below `/run/garuda-toolbox/boot-repair/root`,
   together with `/dev`, `/proc`, `/sys` and `/run`.
4. The app checks which boot loader the system has: systemd-boot when its
   `/etc/kernel/install.conf` says `layout=uki`, GRUB otherwise. It checks
   that the system has that boot loader's programs, and for GRUB a kernel in
   `/boot`. A data partition or a wrong layout stops here.
5. The EFI system partition is mounted for a GRUB reinstall on UEFI, at
   `/boot/efi`, and for every systemd-boot repair, where the system's own
   `/etc/fstab` mounts it.
6. The installed system's own programs run inside it:
   - GRUB: `grub-install` when it is reinstalled, then `dracut-rebuild` (or
     `mkinitcpio -P` where it is missing) to build every initramfs again,
     then `grub-mkconfig`.
   - systemd-boot: `bootctl install` when it is reinstalled, then
     `dracut-rebuild` (or `mkinitcpio -P`), then Garuda's
     `/usr/lib/garuda/kernel-install-uki add` (or `kernel-install add-all`)
     to build the kernel images again. They run with
     `SYSTEMD_RELAX_ESP_CHECKS=1`, since the live system cannot always tell
     them the type of the partition.
7. Everything is unmounted again and the encrypted partition is locked, also
   when a step failed or you cancelled.

The passphrase goes to `cryptsetup` directly. It is not shown in the Console,
not written to the log and not kept in any file.

Afterwards the page says how it went:

| Message                                                                                                             | Meaning                                             |
| ------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------- |
| **The repair finished without an error. Restart the computer without the live medium to try the installed system.** | Every command succeeded                             |
| **The repair failed. Nothing stays mounted or unlocked. The Console shows which command failed.**                   | A command failed. **Show Console** opens the output |
| **Another task was running, so the repair did not start. Try again when it has finished.**                          | The Console was busy with another run               |

## When a system cannot be repaired here

- **The running system uses the disk of this partition, so it cannot be
  repaired from here.** Something from that disk is mounted, for example
  because you opened it in the file manager. Unmount it and click
  **Read again**. The disk the live system itself runs from is never written
  to.
- **No EFI system partition was found on a disk that is free, so no boot
  loader can be installed. The boot menu of a GRUB system can still be
  written again.** The live system was started through UEFI, but no free
  disk has an EFI system partition. A system that was installed the legacy
  way needs the live medium started the legacy way. A systemd-boot system
  needs its EFI system partition for every repair.
- **No installed system was found**: no disk holds a partition with a Linux
  file system or an encrypted partition.
- The Console says the partition has no `/usr/bin/grub-install` or
  `/usr/bin/bootctl`: the partition holds no installed system with that boot
  loader, or **Where the system lies** names the wrong place.
- A volume inside an encrypted partition (LVM on LUKS) only shows once the
  partition is unlocked. Unlock it in the file manager or with
  `cryptsetup open`, then click **Read again**.

For anything this page does not cover, **Repair an installed system** on
**Home** opens a root shell inside the installed system, see
[Home and system status](home.md#on-the-live-iso).

## Test mode

Starting the app with `garuda-toolbox --boot-repair` shows the page on an
installed system too, for testing. A banner says so: **Test mode: this page is
meant for the live system and shows here only because Garuda Toolbox was
started with --boot-repair. Nothing is written to a disk the running system
uses.** A system on a disk that is in use cannot be repaired there either.

## See also

- [Boot Options](boot-options.md)
- [The Console](console.md)
- [Home and system status](home.md)
- [The command line](cli.md)
