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
probe-boot-loader = Bootloader
probe-grub = Bootmenü (GRUB)
probe-kernel-cmdline = Kernel-Befehlszeile (systemd-boot)
probe-plymouth = Splash-Screen (Plymouth)
probe-virtual-machine = Prüfung auf virtuelle Maschine
probe-user-configs = deine Konfigurationsdateien
probe-linux-packages = Kernelpakete
probe-running-kernel = laufender Kernel
probe-kernel-releases = installierte Kernel
probe-supported-locales = unterstützte Sprachen
probe-enabled-locales = aktivierte Sprachen
probe-system-locale = Systemsprache
probe-user = dein Konto
probe-live-iso = Prüfung auf Live-System
probe-desktops = installierte Desktops
probe-edition = Garuda-Edition
probe-reboot-pending = ausstehender Neustart
