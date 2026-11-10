
changes-title = Vorgemerkte Änderungen
# The floating pill at the bottom right. $count is how many.
changes-pill =
    { $count ->
        [one] Änderung vorgemerkt
       *[other] Änderungen vorgemerkt
    }
changes-review = Ansehen
changes-hide = Ausblenden
changes-together = Eine Passwortabfrage für alle Änderungen
changes-apply-all =
    { $count ->
        [one] Änderung übernehmen
       *[other] { $count } Änderungen übernehmen
    }
# The button that drops one change, named after it for screen readers.
changes-remove = „{ $change }“ entfernen
# Asks first, hence the ellipsis.
changes-remove-all-ask = Alle entfernen…
changes-remove-all = Alle entfernen
changes-remove-all-question =
    { $count ->
        [one] Vorgemerkte Änderung entfernen?
       *[other] Alle { $count } vorgemerkten Änderungen entfernen?
    }
changes-apply = Übernehmen
changes-keep = Behalten
# Tag of a change whose last apply stopped before all its steps ended well.
change-unfinished = Nicht abgeschlossen
changes-wait-read = Übernehmen wartet, bis das System gelesen ist.
changes-wait-locks = Übernehmen wartet auf den Sperrstatus der Konten. Die Seite Benutzer und Gruppen meldet, wenn er nicht gelesen werden kann.
changes-wait-aur = Übernehmen wartet auf die Paketliste des AUR. Die Seite Pakete meldet, wenn sie nicht gelesen werden kann.

# $name is what the change is about.

change-account-lock = Konto { $name } sperren
change-account-unlock = Konto { $name } entsperren
change-clean-package-cache = Paket-Cache leeren
change-dkms-build = Modul { $module } für Kernel { $kernel } bauen
change-dns = Namen über { $name } auflösen
change-driver-allow = Treiber { $name } von der Blacklist nehmen
change-driver-blacklist = Treiber { $name } auf die Blacklist setzen
change-full-name-clear = Vollen Namen von { $name } entfernen
change-full-name-set = Vollen Namen von { $user } auf { $name } setzen
change-garuda-update-flag-on = { $name } in Garuda Update einschalten
change-garuda-update-flag-off = { $name } in Garuda Update ausschalten
change-group-add = Gruppe { $name } anlegen
change-group-join = { $user } zur Gruppe { $group } hinzufügen
change-group-leave = { $user } aus der Gruppe { $group } entfernen
change-group-remove = Gruppe { $name } löschen
change-grub-background = { $name } als Hintergrund des Bootmenüs verwenden
change-grub-background-none = Hintergrund des Bootmenüs entfernen
change-grub-default = { $name } standardmäßig starten
change-grub-default-saved = Standardmäßig den zuletzt gewählten Eintrag starten
change-grub-os-prober-on = Nach anderen Betriebssystemen für das Bootmenü suchen
change-grub-os-prober-off = Nicht mehr nach anderen Betriebssystemen für das Bootmenü suchen
change-grub-parameters = Kernel mit { $name } starten
change-grub-parameters-none = Kernel ohne zusätzliche Parameter starten
change-sdboot-parameters = Kernel-Images mit { $name } bauen
change-sdboot-parameters-none = Kernel-Images ohne Parameter bauen
change-sdboot-plymouth-theme = Kernel-Images mit dem Splash-Screen-Theme { $name } bauen
change-grub-save-default-on = Zuletzt gewählten Eintrag im Bootmenü merken
change-grub-save-default-off = Zuletzt gewählten Eintrag im Bootmenü nicht mehr merken
change-grub-submenus-on = Ältere Kernel in einem Untermenü gruppieren
change-grub-submenus-off = Alle Kernel direkt im Bootmenü auflisten
change-grub-theme = GRUB-Theme { $name } verwenden
change-grub-theme-none = GRUB-Theme entfernen
change-grub-timeout =
    { $name ->
        [one] Bootmenü { $name } Sekunde lang anzeigen
       *[other] Bootmenü { $name } Sekunden lang anzeigen
    }
change-hblock-on = Werbe- und Trackerblocker einschalten (hBlock)
change-hblock-off = Werbe- und Trackerblocker ausschalten (hBlock)
change-home-folder = Home-Verzeichnis von { $user } nach { $folder } verschieben
change-loader-timeout =
    { $name ->
        [one] systemd-boot-Menü { $name } Sekunde lang anzeigen
       *[other] systemd-boot-Menü { $name } Sekunden lang anzeigen
    }
change-loader-timeout-menu =
    { $name ->
        [menu-force] systemd-boot-Menü zeigen, bis eine Taste gedrückt wird
       *[other] systemd-boot-Menü verbergen, außer beim Start wird eine Taste gehalten
    }
change-loader-timeout-unset = Die eigene Wartezeit von systemd-boot verwenden
change-loader-default = { $name } mit systemd-boot standardmäßig starten
change-loader-default-saved = Mit systemd-boot standardmäßig den zuletzt gewählten Eintrag starten
change-loader-default-unset = Standardmäßig den ersten Eintrag des systemd-boot-Menüs starten
change-loader-editor-on = Bearbeiten von systemd-boot-Einträgen beim Start erlauben
change-loader-editor-off = Bearbeiten von systemd-boot-Einträgen beim Start verbieten
change-loader-editor-unset = Die eigene Einstellung von systemd-boot zum Bearbeiten verwenden
change-loader-auto-entries-on = systemd-boot die gefundenen Einträge hinzufügen lassen
change-loader-auto-entries-off = systemd-boot keine gefundenen Einträge mehr hinzufügen lassen
change-loader-auto-entries-unset = Die eigene Einstellung von systemd-boot für gefundene Einträge verwenden
change-loader-auto-firmware-on = systemd-boot einen Eintrag für die Firmware-Einrichtung hinzufügen
change-loader-auto-firmware-off = Den Eintrag für die Firmware-Einrichtung aus systemd-boot entfernen
change-loader-auto-firmware-unset = Die eigene Einstellung von systemd-boot für den Firmware-Eintrag verwenden
change-locale-enable = Sprache { $name } hinzufügen
change-locale-disable = Sprache { $name } entfernen
change-login-shell = Login-Shell von { $user } auf { $shell } ändern
change-package-install = { $name } installieren
change-package-remove = { $name } entfernen
change-aur-install = { $name } aus dem AUR bauen und installieren
change-plymouth-theme = Splash-Screen-Theme { $name } verwenden
change-refresh-keyring = Schlüsselbund aktualisieren
change-remove-database-lock = Sperre der Paketdatenbank entfernen
change-remove-orphans = Verwaiste Pakete entfernen
change-system-locale = { $name } als Systemsprache verwenden
change-unit-enable = { $name } beim Systemstart starten
change-unit-disable = { $name } nicht mehr beim Systemstart starten
change-unit-mask = { $name } dauerhaft am Start hindern
change-unit-unmask = { $name } wieder starten lassen
change-upgrade-system = System aktualisieren
change-user-config-on = { $name } einschalten
change-user-config-off = { $name } ausschalten
change-user-remove-keep-home = Benutzer { $name } löschen, Home-Verzeichnis behalten
change-user-remove-with-home = Benutzer { $name } samt Home-Verzeichnis löschen
change-user-unit-enable = { $name } bei der Anmeldung starten
change-user-unit-disable = { $name } nicht mehr bei der Anmeldung starten
change-user-unit-mask = Benutzerdienst { $name } am Start hindern
change-user-unit-unmask = Benutzerdienst { $name } wieder starten lassen
change-wifi-backend = { $name } als WLAN-Backend verwenden
