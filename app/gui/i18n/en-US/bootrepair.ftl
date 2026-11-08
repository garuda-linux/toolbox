# The Boot Repair page: reinstall GRUB or write the boot menu again for an
# installed system, from the live system.

page-boot-repair-subtitle = Install the boot loader of a system that does not start again, or write its boot menu again.
repair-refresh = Read again
repair-intro = Use this page when an installed system does not start anymore. It installs the GRUB boot loader of that system again or writes its boot menu again.
repair-asks-first = Each repair asks first and names the partition and the disk it writes to. A system with /boot on a partition of its own, or with its EFI system partition somewhere else than /boot/efi, is not repaired here.
repair-test-mode = Test mode: this page is meant for the live system and shows here only because Garuda Toolbox was started with --boot-repair. Nothing is written to a disk the running system uses.
repair-dismiss = Dismiss

repair-reading = Reading the disks…
repair-failed = The disks could not be read
repair-failed-detail = The log says why. Use Refresh to try once more.
repair-missing = Boot repair cannot work on this system
repair-missing-detail = These programs are missing: { $tools }. Install them and use Refresh.
repair-none = No installed system was found
repair-none-detail = No disk holds a partition with a Linux file system or an encrypted partition.
repair-hidden = Boot repair is only offered on the live system

repair-system = Installed system
repair-root = Root partition
repair-root-detail = The partition that holds the system that does not start. Every partition with a Linux file system and every encrypted one is listed.
repair-firmware = Started through
repair-uefi = UEFI
repair-bios = BIOS (legacy)
repair-uefi-detail = The live system was started through UEFI, so GRUB is installed for UEFI.
repair-bios-detail = The live system was started the legacy way, so GRUB is installed into the boot sector of a disk. For a UEFI installation start the live system through UEFI.
repair-encryption = Encryption
repair-encryption-none = Not encrypted
repair-encryption-locked = Encrypted (LUKS), locked
repair-encryption-unlocked = Encrypted (LUKS), unlocked
repair-locked-detail = You are asked for the passphrase before a repair. The partition is locked again afterwards.
repair-disk = Disk
repair-layout = Where the system lies
repair-layout-detail = Garuda installs into the subvolume @, a Btrfs folder that holds the system. Choose the top of the file system only for a system that was installed without subvolumes.
repair-layout-subvolume = In the subvolume @ (Garuda's layout, a Btrfs folder that holds the system)
repair-layout-plain = At the top of the file system
# The same, short enough for the choice on the page.
repair-layout-subvolume-short = In the subvolume @ (Garuda's layout)
repair-layout-plain-short = At the top of the file system
repair-in-use = The running system uses the disk of this partition, so it cannot be repaired from here. Start the live system from another medium, or unmount what is mounted from that disk and use Refresh.

repair-work = Repair
repair-esp = EFI system partition
repair-esp-detail = GRUB is written to this partition.
repair-target-disk = Disk for the boot loader
repair-target-disk-detail = GRUB is written to the first sectors of this disk. Choose the disk the computer starts from.
repair-no-esp = No EFI system partition was found on a disk that is free, so GRUB cannot be installed. The boot menu can still be written again.
repair-no-disk = No free disk was found, so GRUB cannot be installed.
repair-reinstall = Reinstall GRUB…
repair-regenerate = Generate boot menu…

repair-running = The repair is running. The Console shows its output.
repair-succeeded = The repair finished without an error. Restart the computer without the live medium to try the installed system.
repair-failed-run = The repair failed. Nothing stays mounted or unlocked. The Console shows which command failed.
repair-busy = Another task was running, so the repair did not start. Try again when it has finished.
repair-show-console = Show Console

# The confirm dialog. $system and $target are devices, $disks and $disk
# the disks they lie on.
repair-reinstall-title = Reinstall GRUB?
repair-regenerate-title = Generate the boot menu again?
repair-reinstall-confirm = Reinstall GRUB
repair-regenerate-confirm = Generate boot menu
repair-cancel = Cancel
repair-does-regenerate = The boot menu /boot/grub/grub.cfg of the system on { $system } (disk { $disks }) is written again. Nothing else on the disk changes.
repair-does-uefi = GRUB is written to the EFI system partition { $target } on the disk { $disk } and entered into the firmware's boot menu as "Garuda". Then the boot menu /boot/grub/grub.cfg of the system on { $system } (disk { $disks }) is written again.
repair-does-bios = GRUB is written to the first sectors of the disk { $target }, in place of the boot loader that is there now. Then the boot menu /boot/grub/grub.cfg of the system on { $system } (disk { $disks }) is written again.
# $layout is one of the layouts above.
repair-looked-for = The system is looked for here: { $layout }
repair-runs-now = It runs right away and asks for an administrator's password. The Console shows every command.
repair-encrypted = { $system } is encrypted. It is unlocked for the repair and locked again afterwards. The passphrase is not kept in any file of Garuda Toolbox.
repair-passphrase = Passphrase of the encrypted partition

repair-refused-not-offered = Boot repair is only offered on the live system.
repair-refused-unavailable = The disks could not be read, so nothing can be repaired.
repair-refused-unknown-system = This system is not in the list anymore. Read the disks again.
repair-refused-in-use = The running system uses this disk, so nothing is written to it.
repair-refused-unknown-target = Choose where the boot loader goes first.
repair-refused-unknown-layout = Choose where the system lies first.
repair-refused-empty = Enter the passphrase of the encrypted partition.
repair-refused-too-long = The passphrase is too long.
repair-refused-control-character = The passphrase holds a line break or another character that cannot be typed.
repair-refused-other = The repair could not be started.
repair-why-passphrase = Enter the passphrase to go on.
repair-why-busy = Another task is running.
