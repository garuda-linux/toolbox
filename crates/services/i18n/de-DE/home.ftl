home-hello = Hallo, { $user }!
home-welcome = Willkommen bei Garuda Linux
home-subtitle = System pflegen und einrichten.

home-status = Systemstatus
home-refresh = Neu einlesen
home-checking = System wird geprüft…
home-all-clear =
    { $count ->
        [one] Prüfung bestanden
       *[other] Alle { $count } Prüfungen bestanden
    }
home-status-failed = Die Suche nach Updates ist fehlgeschlagen.
home-failed =
    { $count ->
        [one] Eine Prüfung konnte nicht ausgeführt werden. Ihr Ergebnis fehlt.
       *[other] { $count } Prüfungen konnten nicht ausgeführt werden. Ihre Ergebnisse fehlen.
    }
home-show-details = Details anzeigen
home-hide-details = Details ausblenden
# $reason is the program's own words, in English.
home-failed-updates = Updates der Repositorys: { $reason }
home-failed-aur = AUR-Updates: { $reason }
home-failed-merges = Offene .pacnew-Dateien: { $reason }
home-failed-last-update = Letztes Update: { $reason }
home-failed-disk = Speicherplatz: { $reason }
home-failed-services = Dienste: { $reason }
home-failed-cleanup = Verwaiste Pakete und Paket-Cache: { $reason }
home-failed-health = garuda-health: { $reason }

home-updates =
    { $count ->
        [one] Ein Update verfügbar
       *[other] { $count } Updates verfügbar
    }
home-updates-detail = Jetzt in der Konsole aktualisieren oder das Update zusammen mit den übrigen Änderungen vormerken.
home-update-now = Jetzt aktualisieren
home-queue-update = Update vormerken
home-unqueue-update = Nicht aktualisieren
home-show-packages = Pakete anzeigen
home-aur-updates =
    { $count ->
        [one] Ein AUR-Update verfügbar
       *[other] { $count } AUR-Updates verfügbar
    }
home-aur-detail = paru baut Pakete aus dem AUR (Arch User Repository) in der Konsole. Dort die Build-Dateien prüfen und die Rückfragen beantworten.
home-aur-update = In der Konsole aktualisieren
home-stale = Ein Update ist überfällig
home-stale-detail = Arch-basierte Systeme sollten mindestens alle zwei Wochen aktualisiert werden.
home-merges = Offene .pacnew-Dateien
home-merges-review = Prüfen…
home-merges-meld = In meld öffnen
home-merges-title = Konfigurationsdateien zusammenführen
home-merges-reading = Dateien werden gelesen…
home-merges-failed = Die Dateien konnten nicht gelesen werden.
home-merges-none = Es gibt nichts mehr zusammenzuführen.
home-merges-files = Dateien
home-merges-pair = { $copy } zu { $file }
home-merges-queued = vorgemerkt
home-merges-take-back = Zurücknehmen
home-merges-unreadable = Eine der Dateien konnte nicht gelesen werden. Sie kann stattdessen mit meld zusammengeführt werden.
home-merges-gone = Die zugehörige Datei existiert nicht mehr. Die Kopie kann an ihre Stelle treten.
home-merges-same = Die Kopie ist mit der aktuellen Datei identisch.
home-merges-older = Ältere Sicherung, die nur gelöscht werden kann.
home-merges-remove = Kopie löschen
home-merges-keep = Beibehalten
home-merges-take-new = Übernehmen
home-merges-take-saved = Sicherung übernehmen
home-merges-change = Änderung { $number }
home-merges-unchanged = { $count ->
        [one] Eine unveränderte Zeile
       *[other] { $count } unveränderte Zeilen
    }
home-merges-keep-file = Aktuelle Datei behalten
home-merges-use-file = Neue Datei übernehmen
home-merges-restore-file = Sicherung wiederherstellen
home-merges-queue-picks = Zusammenführung vormerken
home-merges-edit = Ergebnis bearbeiten…
home-merges-edit-name = Zusammengeführter Text
home-merges-edit-invalid = Der Text ist zu groß oder enthält ein NUL-Zeichen.
home-merges-edit-back = Zurück
home-merges-edit-queue = Bearbeiteten Text vormerken
home-merges-applied = Die Auswahl wird zusammen mit den übrigen Änderungen angewendet. Ist snapper eingerichtet, wird vorher ein Snapshot erstellt. Die bisherige Datei wird unter /var/lib/garuda-toolbox/backups gesichert.
home-health-errors = garuda-health hat Probleme gefunden
home-health-warnings = garuda-health hat Warnungen gefunden
home-health-fix = Beheben
home-health-check = In der Konsole anzeigen
home-restart = Neustart nötig, um das Update abzuschließen
home-restart-detail = Der Kernel oder Systembibliotheken wurden im laufenden Betrieb ersetzt.
home-restart-button = Neu starten…

home-restart-question = Jetzt neu starten?
home-restart-question-detail = Die Arbeit zuerst speichern. Offene Programme werden geschlossen.
home-restart-confirm = Neu starten
home-cancel = Abbrechen
home-restart-denied = Der Neustart wurde nicht genehmigt.
home-restart-failed = Der Computer ließ sich nicht neu starten. Über das Menü des Desktops neu starten.

home-pending = Ausstehende Updates
home-pending-none = Es stehen keine Updates aus.
home-pending-aur = AUR
home-pending-versions = { $from } → { $to }

home-live = Live-System
home-install = Garuda Linux installieren
home-install-detail = Das Installationsprogramm wird gestartet. Es fragt nach dem Passwort.
home-install-button = Installation starten
# $reason is the system's own words, in English.
home-install-failed = Das Installationsprogramm ließ sich nicht starten: { $reason }
home-programs-failed = Die Suche nach den Werkzeugen des Live-Systems ist fehlgeschlagen.
home-chroot = Ein installiertes System reparieren
home-chroot-detail = In der Konsole wird eine Root-Shell im installierten Garuda Linux auf diesem Computer geöffnet. Mit exit wird sie wieder verlassen.
home-chroot-button = In der Konsole öffnen

home-next = Nächste Schritte
home-next-setup = Apps auswählen und installieren
home-next-maintenance = System pflegen
home-next-review =
    { $count ->
        [one] Eine Änderung wurde noch nicht übernommen
       *[other] { $count } Änderungen wurden noch nicht übernommen
    }


# $days counts whole days since the last update.
home-summary-fine = Alles in Ordnung
home-summary-attention =
    { $count ->
        [one] Ein Punkt erfordert Aufmerksamkeit
       *[other] { $count } Punkte erfordern Aufmerksamkeit
    }
home-summary-updated-today = · heute aktualisiert
home-summary-updated =
    { $days ->
        [one] · letztes Update gestern
       *[other] · letztes Update vor { $days } Tagen
    }

# The stat tiles under the greeting: a name, then its figure.
home-stat-updates = Updates
home-stat-updates-none = Aktuell
home-stat-updates-count =
    { $count ->
        [one] Eines verfügbar
       *[other] { $count } verfügbar
    }
home-stat-disk = Speicher
# $free is a size such as "38.2 GiB".
home-stat-disk-free = { $free } frei
home-stat-health = Zustand
home-stat-health-fine = Keine Probleme
home-stat-health-warnings = Warnungen
home-stat-health-errors = Probleme gefunden
home-stat-last-update = Letztes Update
home-stat-last-update-today = Heute
home-stat-last-update-days =
    { $days ->
        [one] Gestern
       *[other] Vor { $days } Tagen
    }
# A figure not read yet, or whose check could not run.
home-stat-unknown = Unbekannt

home-disk = Wenig Speicherplatz
home-disk-detail = Verwaiste Pakete entfernen, den Paket-Cache leeren oder alte Snapshots unter „Wartung“ löschen.
home-disk-button = Wartung öffnen
home-services =
    { $count ->
        [one] Ein Dienst ist fehlgeschlagen
       *[other] { $count } Dienste sind fehlgeschlagen
    }
home-services-button = In „Dienste“ anzeigen
home-cleanup = Verwaiste Pakete und Paket-Cache
# $cache is a size such as "6.4 GiB".
home-cleanup-detail-cache = Es gibt keine verwaisten Pakete. Der Paket-Cache belegt { $cache }.
home-cleanup-detail =
    { $orphans ->
        [one] Ein Paket wird nicht mehr gebraucht. Der Paket-Cache belegt { $cache }.
       *[other] { $orphans } Pakete werden nicht mehr gebraucht. Der Paket-Cache belegt { $cache }.
    }
home-cleanup-queue = Aufräumen vormerken
home-cleanup-unqueue = Nicht aufräumen

home-check-updates = Updates der Repositorys
home-check-aur = AUR-Updates
home-check-merges = Offene .pacnew-Dateien
home-check-disk = Speicherplatz
home-check-services = Dienste
home-check-cleanup = Verwaiste Pakete und Cache
home-check-health = garuda-health

home-next-diagnostics = Mit einem Bericht Hilfe holen
home-recent = Zuletzt verwendet

home-news = Neuigkeiten
home-news-garuda-page = Garuda-Forum
home-news-arch-page = Arch-News
home-news-garuda = Garuda
home-news-arch = Arch
# A post that asks for a manual step around an update.
home-news-manual = Manueller Eingriff
# $title is the post's title, as its site gives it.
home-news-open = { $title } öffnen
# Over Home when saved changes did not come back at start. $count is how many.
home-queue-lost =
    { $count ->
        [one] Eine vorgemerkte Änderung konnte nicht wiederhergestellt werden. Die Logdatei nennt den Grund.
       *[other] { $count } vorgemerkte Änderungen konnten nicht wiederhergestellt werden. Die Logdatei nennt den Grund.
    }
home-queue-lost-dismiss = Ausblenden
# A run from Home did not start, as other changes are being applied.
home-busy = Es werden gerade andere Änderungen übernommen, daher wurde der Vorgang nicht gestartet. Erneut starten, sobald sie abgeschlossen sind.
