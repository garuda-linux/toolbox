# The status bar at the bottom of the window.

status-reading = System wird eingelesen…
status-dry-run = Testmodus: Es wird nichts geändert.
# $parts lists the parts by their names below, joined by commas.
status-probes-failed = Einige Teile des Systems konnten nicht gelesen werden: { $parts }
# The info button after it. $reasons is one line per part with its error.
status-probes-reasons = Warum sie nicht gelesen werden konnten:
    { $reasons }

# The parts of the system the app reads.
probe-packages = installierte Pakete
probe-sync-packages = Pakete der Repositorys
probe-package-graph = Paketabhängigkeiten
probe-units = Systemdienste
probe-user-units = Benutzerdienste
probe-masked-units = maskierte Systemdienste
probe-masked-user-units = maskierte Benutzerdienste
probe-accounts = Benutzer und Gruppen
probe-login-shells = Login-Shells
probe-available-shells = verfügbare Shells
probe-dns = DNS-Einstellungen
probe-hblock = Werbeblocker (hBlock)
probe-wifi-backend = WLAN-Backend
probe-garuda-update = Einstellungen von garuda-update
probe-module-blacklist = Blacklist der Kernelmodule
probe-merges = offene .pacnew-Dateien
probe-boot-loader = Bootloader
probe-grub = Bootmenü (GRUB)
probe-kernel-cmdline = Kernel-Befehlszeile (systemd-boot)
probe-plymouth = Splash-Screen (Plymouth)
probe-virtual-machine = Prüfung auf virtuelle Maschine
probe-user-configs = Konfigurationsdateien des Benutzers
probe-linux-packages = Kernelpakete
probe-running-kernel = laufender Kernel
probe-kernel-releases = installierte Kernel
probe-supported-locales = unterstützte Sprachen
probe-enabled-locales = aktivierte Sprachen
probe-system-locale = Systemsprache
probe-user = Benutzerkonto
probe-live-iso = Prüfung auf Live-System
probe-desktops = installierte Desktops
probe-edition = Garuda-Edition
probe-reboot-pending = ausstehender Neustart
# Why a probe failed. The parts that the system printed stay as they are.
probe-cause-command = { $command } endete mit { $status }: { $stderr }
probe-cause-exit-code = Exitcode { $code }
probe-cause-signal = einem Signal
probe-cause-timed-out = { $command }: keine Antwort innerhalb von { $seconds } s, angehalten
probe-cause-cancelled = { $command }: abgebrochen
probe-cause-too-large = { $command }: das Programm hat zu viel ausgegeben
probe-cause-what = { $what }: { $error }
probe-cause-not-utf8 = { $what } ist kein UTF-8: { $raw }
probe-cause-user-id = { $path } nennt keine Benutzerkennung
probe-cause-root = { $path } nennt kein Wurzeldateisystem
probe-cause-boot-time = keine Startzeit in /proc/stat
probe-cause-kernel-release = der Kernel nennt keine Version
probe-cause-not-a-file = { $path } ist keine Datei
probe-cause-nul-in-path = { $path }: NUL im Pfad
probe-cause-panicked = die Abfrage ist abgestürzt
# $kind is a kind of name, $reason says why the value breaks its rule.
probe-cause-name = { $kind } `{ $value }` ist ungültig: { $reason }
probe-cause-listing = lsblk hat keine Geräteliste ausgegeben: { $words }
probe-cause-order-line = paru hat eine unvollständige Zeile ausgegeben: { $line }
# Die Angabe, was ein fehlgeschlagener Lesevorgang betraf, wenn es kein Pfad und kein Befehl ist.
probe-subject-local-database = die lokale Datenbank
probe-subject-disks = die Datenträger
probe-subject-running-kernel = der laufende Kernel
probe-subject-route-list = ip hat keine Routenliste ausgegeben
probe-subject-build-order = die Baureihenfolge von { $package }
probe-subject-sample-build-order = die Baureihenfolge aus der Beispielliste von { $package }
name-reason-empty = der Wert ist leer
name-reason-too-long = der Wert ist länger als { $max } Zeichen
name-reason-invalid-start = der Wert darf nicht mit `{ $character }` beginnen
name-reason-invalid-character = der Wert enthält `{ $character }`
name-reason-unknown-unit-type = der Wert braucht eine Typendung wie `.service`
name-reason-invalid-component = der Wert ist kein absoluter Pfad ohne `.`- und `..`-Teile
name-reason-wrong-file-type = der Wert hat den falschen Dateityp
name-reason-unknown-device = der Wert ist kein Datenträger, keine Partition und kein Gerät mit Zuordnung, wie der Kernel sie benennt
name-kind-package = Paketname
name-kind-unit = Unit-Name
name-kind-user = Benutzername
name-kind-group = Gruppenname
name-kind-locale = Gebietsschemaname
name-kind-kernel-module = Kernelmodulname
name-kind-charset = Zeichensatzname
name-kind-dkms-module = DKMS-Modulname
name-kind-dkms-version = DKMS-Modulversion
name-kind-kernel-release = Kernelversion
name-kind-host = Hostname oder Adresse
name-kind-kernel-parameter = Kernelparameter
name-kind-plymouth-theme = Plymouth-Themename
name-kind-menu-entry = Startmenü-Eintrag
name-kind-grub-picture = GRUB-Hintergrundbild
name-kind-grub-theme = GRUB-Themedatei
name-kind-disk = Datenträger
name-kind-partition = Partition
name-kind-volume = Partition oder Gerät mit Zuordnung
name-kind-mapper = Device-Mapper-Name
name-kind-bootloader-id = Bootloader-Name
name-kind-loader-entry = systemd-boot-Eintrag
name-kind-loader-default = systemd-boot-Standardeintrag
name-kind-build-run = AUR-Bauvorgang
