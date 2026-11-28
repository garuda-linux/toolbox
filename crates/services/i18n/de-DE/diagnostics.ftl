# The reports themselves stay in English.

page-diagnostics-subtitle = Systemberichte für Forenbeiträge und Fehlermeldungen.

diagnostics-reports = Berichte
# Unfolds the report rows while a report shows.
diagnostics-another = Weiteren Bericht erstellen
diagnostics-collect = Erstellen
# What screen readers call a row's Collect. $name is the report's name.
diagnostics-collect-name = { $name } erstellen
diagnostics-cancel = Abbrechen

diagnostics-inxi = Systeminformationen
diagnostics-inxi-description = Übersicht über Hardware, Treiber und System, erstellt mit garuda-inxi.
diagnostics-garuda-health = Systemzustand
diagnostics-garuda-health-description = Probleme, die garuda-health am System findet.
diagnostics-systemd-analyze = Startzeit
diagnostics-systemd-analyze-description = Startdauer der einzelnen Dienste und worauf der Systemstart gewartet hat, ermittelt mit systemd-analyze.
diagnostics-journalctl = Journal
diagnostics-journalctl-description = Die letzten 1000 Zeilen des Systemjournals. Fragt nach dem Passwort, falls das eigene Konto keinen Lesezugriff hat.
diagnostics-pacman = pacman-Log
diagnostics-pacman-description = Einträge von pacman seit seinem letzten Lauf.
diagnostics-dmesg = Kernel-Log
diagnostics-dmesg-description = Kernelmeldungen aus dmesg. Fragt nach dem Passwort, falls das System sie vor Benutzern verbirgt.
diagnostics-all = Vollständiger Bericht
diagnostics-all-description = Alles oben Genannte in einem Bericht. Kann zweimal nach dem Passwort fragen.

diagnostics-idle-detail = Der Bericht bleibt auf diesem Computer, solange er nicht geteilt wird.
diagnostics-collecting = { $name } wird erstellt…
# $step counts from 1.
diagnostics-collecting-step = { $name } wird erstellt ({ $step } von { $count })…
diagnostics-collecting-detail = Das Starten eines anderen Berichts bricht diesen ab.
diagnostics-empty = { $name } hat keine Ausgabe erzeugt.
diagnostics-unavailable = { $name } ist auf diesem System nicht verfügbar.
diagnostics-unavailable-detail = Das nötige Programm ist nicht installiert.
diagnostics-dismissed = { $name } wurde nicht erstellt: Die Passwortabfrage wurde geschlossen.
diagnostics-denied = { $name } wurde nicht erstellt: Das Konto darf dies nicht lesen.
diagnostics-no-agent = { $name } wurde nicht erstellt: Die Passwortabfrage konnte nicht angezeigt werden, da in dieser Sitzung kein polkit-Agent läuft.
diagnostics-helper-missing = { $name } wurde nicht erstellt: Das Hilfsprogramm von Garuda Toolbox fehlt. Garuda Toolbox neu installieren.
# $reason is the program's own words.
diagnostics-failed = { $name } ist fehlgeschlagen: { $reason }
# Die Gründe einer fehlgeschlagenen Sammlung. Die Worte eines Programms bleiben, wie es sie ausgegeben hat.
diagnostics-failed-no-program = Es wurde kein Programm ausgeführt.
diagnostics-failed-exit-code = Exitcode: { $code }
diagnostics-failed-signal = Ein Signal hat das Programm beendet.
diagnostics-failed-start = { $program } konnte nicht gestartet werden: { $error }
diagnostics-failed-timed-out = { $program } hat länger als { $seconds } Sekunden gedauert
diagnostics-failed-too-large = { $program } hat zu viel ausgegeben
diagnostics-failed-read = { $path }: { $error }
diagnostics-failed-helper-missing = das Hilfsprogramm von Garuda Toolbox oder pkexec ist nicht installiert
diagnostics-failed-helper-start = das Hilfsprogramm kann nicht gestartet werden: { $error }
diagnostics-failed-helper-timed-out = das Hilfsprogramm hat nicht rechtzeitig geantwortet
diagnostics-failed-helper-too-long = die Antwort des Hilfsprogramms ist zu lang
diagnostics-failed-helper-pkexec = pkexec kann nicht ausgeführt werden: { $words }
diagnostics-failed-helper-ended = das Hilfsprogramm endete mit { $ending }: { $words }
diagnostics-failed-ending-code = Exitcode { $code }
diagnostics-failed-ending-signal = einem Signal
diagnostics-failed-helper-unreadable = die Antwort des Hilfsprogramms ist unlesbar
diagnostics-failed-helper-stopped = das Lesen wurde beendet, bevor das Hilfsprogramm antwortete
diagnostics-failed-answer = Die Antwort des Hilfsprogramms von Garuda Toolbox wurde nicht verstanden: { $fault }
diagnostics-failed-answer-too-long = die Zeile hat { $length } Bytes und überschreitet das Limit von { $limit }
diagnostics-failed-not-installed = Das nächste Programm ist nicht installiert.
diagnostics-failed-dismissed = Nicht erstellt: Der Passwortdialog wurde geschlossen.
diagnostics-failed-denied = Nicht erstellt: Root-Rechte wurden nicht erteilt.
diagnostics-failed-no-agent = Nicht erstellt: Es konnte kein Passwortdialog erscheinen.
diagnostics-failed-helper-not-installed = Nicht erstellt: Das Hilfsprogramm ist nicht installiert.
diagnostics-stopped = { $name } wurde vor der Antwort beendet.
diagnostics-report-label = Diagnosebericht

# The report's header line. $time is when it came in, as 14:05.
diagnostics-lines =
    { $count ->
        [one] eine Zeile
       *[other] { $count } Zeilen
    }
diagnostics-collected = · erstellt um { $time } UTC · { $lines }
diagnostics-collected-unknown = · { $lines }
diagnostics-search = Bericht durchsuchen
diagnostics-search-clear = Suche leeren

diagnostics-copy = Bericht kopieren
diagnostics-share = Bericht teilen…
diagnostics-share-tooltip = Der Bericht wird verschlüsselt auf bin.garudalinux.org hochgeladen. Danach steht ein Link zum Teilen bereit.
diagnostics-report-copied = Bericht in die Zwischenablage kopiert.
