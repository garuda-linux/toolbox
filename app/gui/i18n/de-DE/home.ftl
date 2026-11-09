home-hello = Hallo, { $user }!
home-welcome = Willkommen bei Garuda Linux
home-subtitle = Pflege dein System und richte es ein.

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
home-updates-detail = Aktualisiere jetzt in der Konsole oder merk das Update zusammen mit deinen anderen Änderungen vor.
home-update-now = Jetzt aktualisieren
home-queue-update = Update vormerken
home-unqueue-update = Nicht aktualisieren
home-show-packages = Pakete anzeigen
home-aur-updates =
    { $count ->
        [one] Ein AUR-Update verfügbar
       *[other] { $count } AUR-Updates verfügbar
    }
home-aur-detail = paru baut Pakete aus dem AUR (Arch User Repository) in der Konsole. Prüfe dort die Build-Dateien und beantworte die Rückfragen.
home-aur-update = In der Konsole aktualisieren
home-stale = Ein Update ist überfällig
home-stale-detail = Arch-basierte Systeme sollten mindestens alle zwei Wochen aktualisiert werden.
home-merges = Offene .pacnew-Dateien
home-health-errors = garuda-health hat Probleme gefunden
home-health-warnings = garuda-health hat Warnungen gefunden
home-health-fix = Beheben
home-health-check = In der Konsole anzeigen
home-restart = Neustart nötig, um das Update abzuschließen
home-restart-detail = Der Kernel oder Systembibliotheken wurden im laufenden Betrieb ersetzt.
home-restart-button = Neu starten…

home-restart-question = Jetzt neu starten?
home-restart-question-detail = Speichere zuerst deine Arbeit. Offene Programme werden geschlossen.
home-restart-confirm = Neu starten
home-cancel = Abbrechen
home-restart-denied = Der Neustart wurde nicht genehmigt.
home-restart-failed = Der Computer ließ sich nicht neu starten. Starte ihn über das Menü deines Desktops neu.

home-pending = Ausstehende Updates
home-pending-none = Es stehen keine Updates aus.
home-pending-aur = AUR
home-pending-versions = { $from } → { $to }

home-live = Live-System
home-install = Garuda Linux installieren
home-install-detail = Startet das Installationsprogramm. Es fragt nach deinem Passwort.
home-install-button = Installation starten
# $reason is the system's own words, in English.
home-install-failed = Das Installationsprogramm ließ sich nicht starten: { $reason }
home-programs-failed = Die Suche nach den Werkzeugen des Live-Systems ist fehlgeschlagen.
home-chroot = Ein installiertes System reparieren
home-chroot-detail = Öffnet in der Konsole eine Root-Shell im installierten Garuda Linux auf diesem Computer. Mit exit verlässt du sie.
home-chroot-button = In der Konsole öffnen

home-next = Nächste Schritte
home-next-setup = Apps auswählen und installieren
home-next-maintenance = Dein System pflegen
home-next-review =
    { $count ->
        [one] Eine Änderung wurde noch nicht übernommen
       *[other] { $count } Änderungen wurden noch nicht übernommen
    }


# $days counts whole days since the last update.
home-summary-fine = Alles in Ordnung
home-summary-unknown = Einige Prüfungen konnten nicht ausgeführt werden
home-summary-attention =
    { $count ->
        [one] Ein Punkt braucht deine Aufmerksamkeit
       *[other] { $count } Punkte brauchen deine Aufmerksamkeit
    }
home-summary-updated-today = · heute aktualisiert
home-summary-updated =
    { $days ->
        [one] · letztes Update gestern
       *[other] · letztes Update vor { $days } Tagen
    }

# $kernel is as uname prints it, $free a size such as "38.2 GiB".
home-fact-packages =
    { $count ->
        [one] Ein Paket
       *[other] { $count } Pakete
    }
home-fact-packages-foreign = { $count } Pakete · { $foreign } aus dem AUR
home-fact-kernel-days =
    { $amount ->
        [one] { $kernel } · läuft seit einem Tag
       *[other] { $kernel } · läuft seit { $amount } Tagen
    }
home-fact-kernel-new = { $kernel } · läuft seit weniger als einer Stunde
home-fact-kernel-hours =
    { $amount ->
        [one] { $kernel } · läuft seit einer Stunde
       *[other] { $kernel } · läuft seit { $amount } Stunden
    }
home-fact-disk = { $free } frei auf /

home-disk = Wenig Speicherplatz
home-disk-detail = Entferne verwaiste Pakete, leere den Paket-Cache oder lösche alte Snapshots unter „Wartung“.
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
