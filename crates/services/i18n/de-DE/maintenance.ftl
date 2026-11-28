
page-maintenance-subtitle = Updates, Aufräumen, Werkzeuge und Reparaturen für ein gepflegtes System.
maintenance-refresh = Neu einlesen

maintenance-tabs = Wartungsbereiche
maintenance-tab-common = Allgemein
maintenance-tab-reset = Einstellungen zurücksetzen
maintenance-tab-repairs = Reparaturen

maintenance-operations = Paketpflege
maintenance-operations-detail = Aufgaben vormerken, um sie zusammen mit den übrigen Änderungen zu übernehmen, oder eine davon sofort in der Konsole ausführen.
maintenance-run-now = Jetzt ausführen
maintenance-unqueue = Vormerkung entfernen
maintenance-queued = Läuft beim Übernehmen
maintenance-op-upgrade-system = System aktualisieren
maintenance-op-upgrade-system-detail = Alle Pakete aktualisieren, mit garuda-update samt Snapshots und Prüfungen.
maintenance-op-upgrade-system-queue = Update vormerken
maintenance-op-remove-orphans = Verwaiste Pakete entfernen
maintenance-op-remove-orphans-detail = Pakete entfernen, die als Abhängigkeiten installiert wurden und nicht mehr gebraucht werden.
maintenance-op-remove-orphans-queue = Entfernen vormerken
maintenance-op-remove-orphans-question = Pakete, die nur als Abhängigkeit installiert wurden und nicht mehr gebraucht werden, werden entfernt. Die Konsole zeigt sie vorher an und fragt noch einmal nach.
maintenance-op-clean-package-cache = Paket-Cache leeren
maintenance-op-clean-package-cache-detail = Pakete aus dem Cache löschen, die nicht mehr installiert sind.
maintenance-op-clean-package-cache-queue = Leeren vormerken
maintenance-op-clean-package-cache-question = Alle Pakete im Cache, die nicht mehr installiert sind, werden gelöscht. Ein Downgrade auf diese Versionen ist danach nicht mehr möglich.
maintenance-op-refresh-keyring = Schlüsselbund aktualisieren
maintenance-op-refresh-keyring-detail = Die neuesten Schlüssel herunterladen, mit denen pacman die Pakete der Repositorys auf Echtheit prüft.
maintenance-op-refresh-keyring-queue = Aktualisieren vormerken
maintenance-op-remove-database-lock = pacman-Sperre entfernen
maintenance-op-remove-database-lock-detail = Die Sperre eines abgebrochenen Updates löschen. Solange pacman läuft, bleibt sie erhalten.
maintenance-op-aur-upgrade-detail = Die aus dem AUR gebauten Pakete mit paru aktualisieren, das seine Fragen im Terminal stellt.
maintenance-op-remove-database-lock-queue = Entsperren vormerken

maintenance-tools = Werkzeuge
maintenance-tools-detail = Jedes Werkzeug öffnet sich in einem eigenen Fenster.
maintenance-open = Öffnen
maintenance-install-open = Installieren und öffnen…
# The dot beside the title of a tool that is not installed.
maintenance-not-installed = Nicht installiert

maintenance-tool-missing = Die Installation startet sofort in der Konsole und fragt nach dem Passwort. Danach wird das Werkzeug geöffnet.
maintenance-ranker-missing = rate-mirrors wird benötigt. Die Installation startet sofort in der Konsole und fragt nach dem Passwort. Danach beginnt die Bewertung.
maintenance-ranking = Die Spiegelserver werden getestet. Das dauert bis zu fünf Minuten. Die Konsole öffnet sich, sobald die neue Liste bereit ist.
maintenance-ranking-progress = Spiegelserver werden bewertet, { $time }
maintenance-ranking-cancel = Bewertung abbrechen
maintenance-tool-arch-mirrors = Arch-Spiegelserver aktualisieren
maintenance-tool-arch-mirrors-detail = Mit reflector-simple die schnellsten Arch-Linux-Spiegelserver wählen, von denen Pakete heruntergeladen werden.
maintenance-tool-chaotic-mirrors = Chaotic-AUR-Spiegelserver aktualisieren
maintenance-tool-chaotic-mirrors-detail = Die Spiegelserver von Chaotic-AUR (fertig gebaute Pakete aus dem Arch User Repository) nach Geschwindigkeit sortieren und die schnellsten speichern.
maintenance-tool-btrfs-assistant = Btrfs Assistant
maintenance-tool-btrfs-assistant-detail = Snapshots verwalten und Btrfs-Wartung ausführen.
maintenance-tool-edit-repositories = Repositorys bearbeiten
maintenance-tool-edit-repositories-detail = Repositorys mit pace, dem Editor für pacman.conf, aktivieren und deaktivieren.
maintenance-tool-merge-pacdiff = Konfigurationsupdates zusammenführen
maintenance-tool-merge-pacdiff-detail = .pacnew- und .pacsave-Dateien nebeneinander vergleichen und zusammenführen. Nach einem Update liegen sie neben einer geänderten Konfigurationsdatei, zusammen mit deren neuer Version.

# Why a tool did not start or the mirrors were not ranked. $tool is the
# tool's name, $reason the system's own words.
maintenance-problem-launch-absent = Die Installation ist fertig, aber { $reason } fehlt, daher wurde { $tool } nicht gestartet.
maintenance-problem-mirrors-timeout = Die Bewertung der Chaotic-AUR-Spiegelserver dauerte länger als fünf Minuten und wurde abgebrochen. Die Liste der Spiegelserver ist unverändert.
maintenance-problem-mirrors-start = rate-mirrors konnte nicht gestartet werden, die Chaotic-AUR-Spiegelserver wurden nicht bewertet: { $reason }
maintenance-problem-mirrors-missing = das Programm ist nicht installiert
maintenance-problem-mirrors-panicked = die Bewertung wurde unerwartet beendet
maintenance-problem-mirrors-exit = Die Bewertung der Chaotic-AUR-Spiegelserver ist fehlgeschlagen, ihre Liste ist unverändert. Die Internetverbindung prüfen und erneut versuchen.
maintenance-problem-mirrors-invalid = Die Bewertung ergab keine brauchbare Liste von Spiegelservern. Die Chaotic-AUR-Liste ist unverändert.
maintenance-problem-busy = Es werden gerade andere Änderungen übernommen, daher wurde der Vorgang nicht gestartet. Erneut starten, sobald sie abgeschlossen sind.
maintenance-problem-dismiss = Ausblenden

maintenance-reset = App-Einstellungen
maintenance-reset-detail = Apps auswählen, deren Einstellungen auf Garudas Standardeinstellungen zurückgesetzt werden sollen. Die aktuellen Dateien bleiben als Sicherung erhalten.
maintenance-reset-empty = Keine Standardeinstellungen gefunden
maintenance-reset-empty-detail = In /etc/skel liegen keine Standardeinstellungen von Garuda.
maintenance-reset-button = Auswahl zurücksetzen…
maintenance-reset-busy = Die gewählten Einstellungen werden zurückgesetzt…
maintenance-reset-done = Einstellungen zurückgesetzt. Die alten Dateien tragen die Endung .bak mit Datum.
# $names lists the applications, separated by commas.
maintenance-reset-failed = Einige Einstellungen konnten nicht zurückgesetzt werden: { $names }
maintenance-reset-question = { $count ->
    [one] 1 Einstellung zurücksetzen?
   *[other] { $count } Einstellungen zurücksetzen?
}
# $files is the comma list of the home files a reset replaces.
maintenance-reset-replaces = Ersetzt { $files }
maintenance-reset-files = Diese Dateien werden durch Garudas Standardfassung ersetzt. Die aktuellen bleiben als Sicherung erhalten:
maintenance-reset-cancel = Abbrechen
maintenance-reset-confirm = Zurücksetzen

maintenance-repairs = Reparaturen
maintenance-repairs-detail = Jede Reparatur läuft in der Konsole, damit sie Rückfragen stellen kann.
# $packages lists the missing packages, separated by commas. $count is
# how many they are.
maintenance-repairs-missing = { $count ->
    [one] Die ausgegrauten Reparaturen brauchen { $packages }, das nicht installiert ist. Zum Freischalten muss es installiert werden. Die Installation startet sofort in der Konsole und fragt nach dem Passwort.
   *[other] Die ausgegrauten Reparaturen brauchen { $packages }, die nicht installiert sind. Zum Freischalten müssen sie installiert werden. Die Installation startet sofort in der Konsole und fragt nach dem Passwort.
}
maintenance-install-missing = { $packages } installieren
maintenance-run = Ausführen
maintenance-repair-health-check = Systemprüfung
maintenance-repair-health-check-detail = Mit garuda-health nach häufigen Problemen suchen.
maintenance-repair-health-fix = Systemprobleme beheben
maintenance-repair-health-fix-detail = Häufige Probleme, die garuda-health findet, automatisch beheben.
maintenance-repair-remote-fix = Paketverwaltung zurücksetzen
maintenance-repair-remote-fix-detail = pacman.conf, die Mirrorlisten und den Schlüsselbund auf Garudas Standardfassung zurücksetzen.
maintenance-repair-remote-keyring = Schlüsselbund zurücksetzen
maintenance-repair-remote-keyring-detail = Den Schlüsselbund von pacman auf Garudas Standardfassung zurücksetzen.
maintenance-repair-remote-full-fix = Vollständig zurücksetzen
maintenance-repair-remote-full-fix-detail = pacman vollständig zurücksetzen und alle Pakete neu installieren.
maintenance-repair-remote-reset-audio = Audio zurücksetzen
maintenance-repair-remote-reset-audio-detail = PipeWire neu installieren und die Audiokonfiguration zurücksetzen.
maintenance-repair-remote-reset-snapper = Snapper zurücksetzen
maintenance-repair-remote-reset-snapper-detail = Snapper vollständig zurücksetzen und seine Konfigurationen neu anlegen.
maintenance-repair-remote-reinstall = Pakete neu installieren
maintenance-repair-remote-reinstall-detail = Alle installierten Pakete neu installieren.

# The question before a repair that reinstalls packages or throws away
# what the user had. $repair is the repair's name, such as Full reset.
maintenance-repair-question = { $repair } ausführen?
maintenance-repair-cancel = Abbrechen
maintenance-repair-remote-fix-question = Die pacman.conf und die Mirrorlisten werden durch Garudas Standardfassung ersetzt. Hinzugefügte Repositorys gehen danach verloren.
maintenance-repair-remote-full-fix-question = Die pacman.conf wird durch Garudas Standardfassung ersetzt, und jedes installierte Paket wird neu heruntergeladen und installiert. Das dauert lange und setzt eine funktionierende Internetverbindung voraus.
maintenance-repair-remote-reset-audio-question = PipeWire und WirePlumber werden neu installiert und die Audioeinstellungen auf den Standard zurückgesetzt.
maintenance-repair-remote-reset-snapper-question = Alle Snapper-Snapshots und Backup-Subvolumen werden gelöscht und die Snapper-Konfigurationen neu angelegt. Ältere Snapshots lassen sich danach nicht mehr wiederherstellen.
maintenance-repair-remote-reinstall-question = Alle installierten Pakete werden neu heruntergeladen und installiert. Das dauert lange und setzt eine funktionierende Internetverbindung voraus.
maintenance-repair-needs = Benötigt { $package }
