# The Boot Repair page: reinstall the boot loader of an installed system or
# build its boot files again, from the live system. Which boot loader the
# system has shows only once it is mounted, so the texts name both.

page-boot-repair-subtitle = Bootloader oder Bootdateien eines Systems reparieren, das nicht mehr startet.
repair-refresh = Neu einlesen
repair-asks-first = Vor jeder Reparatur werden die Partition und der Datenträger genannt, auf die geschrieben wird. GRUB-Systeme mit eigener /boot-Partition oder einer EFI-Systempartition außerhalb von /boot/efi werden hier nicht repariert.
repair-test-mode = Testmodus: Diese Seite ist für das Live-System gedacht und erscheint hier nur, weil Garuda Toolbox mit --boot-repair gestartet wurde. Datenträger des laufenden Systems bleiben unberührt.
repair-dismiss = Ausblenden

repair-reading = Datenträger werden gelesen…
repair-failed = Datenträger konnten nicht gelesen werden
repair-failed-detail = Den Grund nennt das Log. Mit „Neu einlesen“ wird es erneut versucht.
repair-missing = Die Boot-Reparatur ist auf diesem System nicht möglich
repair-missing-detail = Diese Programme fehlen: { $tools }. Die Programme installieren und danach „Neu einlesen“ verwenden.
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
repair-locked-detail = Vor der Reparatur wird nach dem Passwort gefragt. Danach wird die Partition wieder gesperrt.
repair-disk = Datenträger
repair-layout = Speicherort des Systems
repair-layout-detail = Garuda wird ins Subvolumen @ installiert, ein Btrfs-Verzeichnis, das das System enthält. Die oberste Ebene sollte nur gewählt werden, wenn das System ohne Subvolumen installiert wurde.
repair-layout-subvolume = Im Subvolumen @ (Garudas Aufteilung, ein Btrfs-Verzeichnis, das das System enthält)
repair-layout-plain = Auf der obersten Ebene des Dateisystems
# The same, short enough for the choice on the page.
repair-layout-subvolume-short = Im Subvolumen @ (Garudas Aufteilung)
repair-layout-plain-short = Auf der obersten Ebene
repair-in-use = Das laufende System nutzt den Datenträger dieser Partition, darum kann sie von hier aus nicht repariert werden. Das Live-System von einem anderen Medium starten oder alle Partitionen dieses Datenträgers aushängen und danach „Neu einlesen“ wählen.

repair-work = Reparatur
repair-esp = EFI-Systempartition
repair-esp-detail = Der Bootloader wird auf diese Partition geschrieben. Ein System mit systemd-boot hat dort auch seine Kernel-Images.
repair-target-disk = Datenträger für den Bootloader
repair-target-disk-detail = GRUB wird in die ersten Sektoren dieses Datenträgers geschrieben. Den Datenträger wählen, von dem der Computer startet.
repair-no-esp = Auf keinem freien Datenträger wurde eine EFI-Systempartition gefunden, daher kann kein Bootloader installiert werden. Das Bootmenü eines GRUB-Systems kann trotzdem neu geschrieben werden.
repair-no-disk = Es wurde kein freier Datenträger gefunden, daher kann GRUB nicht installiert werden.
repair-reinstall = Bootloader neu installieren…
repair-regenerate = Bootdateien neu bauen…

repair-running = Die Reparatur läuft. Die Ausgabe erscheint in der Konsole.
repair-succeeded = Die Reparatur wurde fehlerfrei abgeschlossen. Den Computer ohne das Live-Medium neu starten, um das installierte System zu testen.
repair-failed-run = Die Reparatur ist fehlgeschlagen. Alle Partitionen wurden wieder ausgehängt und gesperrt. Die Konsole zeigt, welcher Befehl fehlschlug.
repair-busy = Die Reparatur wurde nicht gestartet, weil noch eine andere Aufgabe lief. Erneut versuchen, sobald sie abgeschlossen ist.
repair-show-console = Konsole anzeigen

# The confirm dialog.
repair-reinstall-title = Bootloader neu installieren?
repair-regenerate-title = Bootdateien neu bauen?
repair-reinstall-confirm = Bootloader neu installieren
repair-regenerate-confirm = Bootdateien neu bauen
repair-cancel = Abbrechen
# The labels of the dialog's list of what the repair writes where.
repair-fact-writes = Wird geschrieben
repair-fact-disk = Datenträger
repair-fact-system = System
repair-fact-layout = Speicherort
# What the list says the repair writes. $esp and $target are EFI system
# partitions.
repair-writes-regenerate = Das Bootmenü /boot/grub/grub.cfg von GRUB
repair-writes-regenerate-uefi = Das Bootmenü /boot/grub/grub.cfg von GRUB oder Initramfs und Kernel-Images von systemd-boot auf { $esp }
repair-writes-uefi = Der Bootloader auf die EFI-Systempartition { $target }, danach die Bootdateien des Systems
repair-writes-bios = GRUB in die ersten Sektoren des Datenträgers, danach das Bootmenü /boot/grub/grub.cfg
# The one sentence below the list.
repair-then-regenerate = Sonst ändert sich nichts auf dem Datenträger, und ein System mit systemd-boot wird ohne UEFI so nicht repariert.
repair-then-regenerate-uefi = Sonst ändert sich nichts auf den Datenträgern.
repair-then-uefi = Der Bootloader wird in das Bootmenü der Firmware eingetragen, GRUB als „Garuda“ und systemd-boot als „Linux Boot Manager“.
repair-then-bios = GRUB ersetzt den bisherigen Bootloader, und ein System mit systemd-boot wird so nicht repariert.
repair-runs-now = Die Reparatur startet sofort und fragt nach dem Passwort eines Administrators. Jeder Befehl erscheint in der Konsole.
# In the question on an installed system, not on the live ISO.
repair-trust-disk = Dies ist nicht die Live-ISO. grub-install, mkinitcpio und bootctl des gewählten Systems laufen als root. Es sollte nur ein Datenträger repariert werden, auf den niemand sonst schreiben konnte.
repair-encrypted = { $system } ist verschlüsselt. Für die Reparatur wird es entsperrt und danach wieder gesperrt. Das Passwort wird in keiner Datei von Garuda Toolbox gespeichert.
repair-passphrase = Passwort der verschlüsselten Partition

repair-refused-not-offered = Die Boot-Reparatur ist nur im Live-System verfügbar.
repair-refused-unavailable = Die Datenträger ließen sich nicht lesen, daher kann nichts repariert werden.
repair-refused-unknown-system = Dieses System ist nicht mehr in der Liste. Die Datenträger neu einlesen.
repair-refused-in-use = Das laufende System nutzt diesen Datenträger, daher wird nichts darauf geschrieben.
repair-refused-unknown-target = Das Ziel für den Bootloader muss zuerst gewählt werden.
repair-refused-unknown-layout = Der Speicherort des Systems muss zuerst gewählt werden.
repair-refused-empty = Das Passwort der verschlüsselten Partition eingeben.
repair-refused-too-long = Das Passwort ist zu lang.
repair-refused-control-character = Das Passwort enthält einen Zeilenumbruch oder ein anderes Zeichen, das sich nicht tippen lässt.
repair-refused-other = Die Reparatur konnte nicht gestartet werden.
repair-why-passphrase = Das Passwort muss eingegeben werden, um fortzufahren.
repair-why-busy = Gerade läuft noch eine andere Aufgabe.
