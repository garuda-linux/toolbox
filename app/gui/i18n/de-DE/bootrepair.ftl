# The Boot Repair page: reinstall the boot loader of an installed system or
# build its boot files again, from the live system. Which boot loader the
# system has shows only once it is mounted, so the texts name both.

page-boot-repair-subtitle = Bootloader oder Bootdateien eines Systems reparieren, das nicht mehr startet.
repair-refresh = Neu einlesen
repair-intro = Startet ein installiertes System nicht mehr, kannst du hier seinen Bootloader neu installieren, GRUB oder systemd-boot, oder seine Bootdateien neu bauen: das Bootmenü von GRUB oder die Kernel-Images von systemd-boot.
repair-asks-first = Vor jeder Reparatur siehst du, auf welche Partition und welchen Datenträger geschrieben wird. GRUB-Systeme mit eigener /boot-Partition oder einer EFI-Systempartition außerhalb von /boot/efi werden hier nicht repariert.
repair-test-mode = Testmodus: Diese Seite ist für das Live-System gedacht und erscheint hier nur, weil Garuda Toolbox mit --boot-repair gestartet wurde. Datenträger des laufenden Systems bleiben unberührt.
repair-dismiss = Ausblenden

repair-reading = Datenträger werden gelesen…
repair-failed = Datenträger konnten nicht gelesen werden
repair-failed-detail = Den Grund findest du im Log. Mit „Neu einlesen“ versuchst du es erneut.
repair-missing = Die Boot-Reparatur ist auf diesem System nicht möglich
repair-missing-detail = Diese Programme fehlen: { $tools }. Installiere sie und wähle dann „Neu einlesen“.
repair-none = Kein installiertes System gefunden
repair-none-detail = Kein Datenträger enthält eine Partition mit einem Linux-Dateisystem oder eine verschlüsselte Partition.
repair-hidden = Die Boot-Reparatur ist nur im Live-System verfügbar

repair-system = Installiertes System
repair-root = Root-Partition
repair-root-detail = Die Partition des Systems, das nicht startet. Aufgeführt sind alle Partitionen mit Linux-Dateisystem und alle verschlüsselten.
repair-firmware = Gestartet über
repair-uefi = UEFI
repair-bios = BIOS (Legacy)
repair-uefi-detail = Das Live-System wurde über UEFI gestartet, daher wird der Bootloader für UEFI installiert.
repair-bios-detail = Das Live-System läuft im Legacy-Modus, deshalb wird GRUB in den Bootsektor eines Datenträgers installiert. systemd-boot und jede UEFI-Installation brauchen ein per UEFI gestartetes Live-System.
repair-encryption = Verschlüsselung
repair-encryption-none = Nicht verschlüsselt
repair-encryption-locked = Verschlüsselt (LUKS), gesperrt
repair-encryption-unlocked = Verschlüsselt (LUKS), entsperrt
repair-locked-detail = Vor der Reparatur wirst du nach dem Passwort gefragt. Danach wird die Partition wieder gesperrt.
repair-disk = Datenträger
repair-layout = Speicherort des Systems
repair-layout-detail = Garuda installiert ins Subvolumen @, ein Btrfs-Verzeichnis mit dem System. Wähle die oberste Ebene nur, wenn das System ohne Subvolumen installiert wurde.
repair-layout-subvolume = Im Subvolumen @ (Garudas Aufteilung, ein Btrfs-Verzeichnis, das das System enthält)
repair-layout-plain = Auf der obersten Ebene des Dateisystems
# The same, short enough for the choice on the page.
repair-layout-subvolume-short = Im Subvolumen @ (Garudas Aufteilung)
repair-layout-plain-short = Auf der obersten Ebene
repair-in-use = Das laufende System nutzt den Datenträger dieser Partition, deshalb kann sie von hier aus nicht repariert werden. Starte das Live-System von einem anderen Medium oder hänge alle Partitionen dieses Datenträgers aus und wähle dann „Neu einlesen“.

repair-work = Reparatur
repair-esp = EFI-Systempartition
repair-esp-detail = Der Bootloader wird auf diese Partition geschrieben. Ein System mit systemd-boot hat dort auch seine Kernel-Images.
repair-target-disk = Datenträger für den Bootloader
repair-target-disk-detail = GRUB wird in die ersten Sektoren dieses Datenträgers geschrieben. Wähle den Datenträger, von dem dein Computer startet.
repair-no-esp = Auf keinem freien Datenträger wurde eine EFI-Systempartition gefunden, daher kann kein Bootloader installiert werden. Das Bootmenü eines GRUB-Systems kann trotzdem neu geschrieben werden.
repair-no-disk = Es wurde kein freier Datenträger gefunden, daher kann GRUB nicht installiert werden.
repair-reinstall = Bootloader neu installieren…
repair-regenerate = Bootdateien neu bauen…

repair-running = Die Reparatur läuft. Ihre Ausgabe siehst du in der Konsole.
repair-succeeded = Die Reparatur wurde fehlerfrei abgeschlossen. Starte den Computer ohne das Live-Medium neu, um das installierte System zu testen.
repair-failed-run = Die Reparatur ist fehlgeschlagen. Alle Partitionen wurden wieder ausgehängt und gesperrt. Welcher Befehl fehlschlug, siehst du in der Konsole.
repair-busy = Die Reparatur wurde nicht gestartet, weil noch eine andere Aufgabe lief. Versuche es erneut, wenn sie abgeschlossen ist.
repair-show-console = Konsole anzeigen

# The confirm dialog. $system and $target are devices, $disks and $disk
# the disks they lie on.
repair-reinstall-title = Bootloader neu installieren?
repair-regenerate-title = Bootdateien neu bauen?
repair-reinstall-confirm = Bootloader neu installieren
repair-regenerate-confirm = Bootdateien neu bauen
repair-cancel = Abbrechen
repair-does-regenerate = Bei einem GRUB-System wird das Bootmenü /boot/grub/grub.cfg des Systems auf { $system } (Datenträger { $disks }) neu geschrieben. Sonst ändert sich nichts auf dem Datenträger. Ein System mit systemd-boot wird ohne UEFI so nicht repariert.
repair-does-regenerate-uefi = Bei einem GRUB-System wird das Bootmenü /boot/grub/grub.cfg des Systems auf { $system } (Datenträger { $disks }) neu geschrieben. Bei einem System mit systemd-boot werden Initramfs und Kernel-Images neu auf die EFI-Systempartition { $esp } gebaut. Sonst ändert sich nichts auf den Datenträgern.
repair-does-uefi = Der Bootloader wird auf die EFI-Systempartition { $target } auf dem Datenträger { $disk } geschrieben und in das Bootmenü der Firmware eingetragen: GRUB als „Garuda“, systemd-boot als „Linux Boot Manager“. Danach werden die Bootdateien des Systems auf { $system } (Datenträger { $disks }) neu gebaut: das Bootmenü /boot/grub/grub.cfg von GRUB oder die Kernel-Images von systemd-boot.
repair-does-bios = GRUB wird in die ersten Sektoren des Datenträgers { $target } geschrieben, anstelle des bisherigen Bootloaders. Danach wird das Bootmenü /boot/grub/grub.cfg des Systems auf { $system } (Datenträger { $disks }) neu geschrieben. Ein System mit systemd-boot wird so nicht repariert.
# $layout is one of the layouts above.
repair-looked-for = Erwarteter Speicherort des Systems: { $layout }
repair-runs-now = Die Reparatur startet sofort und fragt nach dem Passwort eines Administrators. Jeder Befehl erscheint in der Konsole.
repair-encrypted = { $system } ist verschlüsselt. Für die Reparatur wird es entsperrt und danach wieder gesperrt. Garuda Toolbox speichert dein Passwort nirgends.
repair-passphrase = Passwort der verschlüsselten Partition

repair-refused-not-offered = Die Boot-Reparatur ist nur im Live-System verfügbar.
repair-refused-unavailable = Die Datenträger ließen sich nicht lesen, daher kann nichts repariert werden.
repair-refused-unknown-system = Dieses System ist nicht mehr in der Liste. Lies die Datenträger neu ein.
repair-refused-in-use = Das laufende System nutzt diesen Datenträger, daher wird nichts darauf geschrieben.
repair-refused-unknown-target = Wähle zuerst das Ziel für den Bootloader.
repair-refused-unknown-layout = Wähle zuerst den Speicherort des Systems.
repair-refused-empty = Gib das Passwort der verschlüsselten Partition ein.
repair-refused-too-long = Das Passwort ist zu lang.
repair-refused-control-character = Das Passwort enthält einen Zeilenumbruch oder ein anderes Zeichen, das sich nicht tippen lässt.
repair-refused-other = Die Reparatur konnte nicht gestartet werden.
repair-why-passphrase = Gib zuerst das Passwort ein.
repair-why-busy = Gerade läuft noch eine andere Aufgabe.
