# The reports themselves stay in English.

page-diagnostics-subtitle = Systemberichte für Forenbeiträge und Fehlermeldungen.

diagnostics-report-picker = Bericht auswählen
diagnostics-collect = Erstellen
diagnostics-cancel = Abbrechen

diagnostics-inxi = Systeminformationen
diagnostics-inxi-description = Übersicht über Hardware, Treiber und System, erstellt mit garuda-inxi.
diagnostics-garuda-health = Systemzustand
diagnostics-garuda-health-description = Probleme, die garuda-health am System findet.
diagnostics-systemd-analyze = Startzeit
diagnostics-systemd-analyze-description = Startdauer der einzelnen Dienste und worauf der Systemstart gewartet hat, ermittelt mit systemd-analyze.
diagnostics-journalctl = Journal
diagnostics-journalctl-description = Die letzten 1000 Zeilen des Systemjournals. Fragt nach deinem Passwort, falls dein Konto keinen Lesezugriff hat.
diagnostics-pacman = pacman-Log
diagnostics-pacman-description = Einträge von pacman seit seinem letzten Lauf.
diagnostics-dmesg = Kernel-Log
diagnostics-dmesg-description = Kernelmeldungen aus dmesg. Fragt nach deinem Passwort, falls das System sie vor Benutzern verbirgt.
diagnostics-all = Vollständiger Bericht
diagnostics-all-description = Alles oben Genannte in einem Bericht. Kann zweimal nach deinem Passwort fragen.

diagnostics-idle = Wähle einen Bericht und klicke auf „Erstellen“.
diagnostics-idle-detail = Der Bericht bleibt auf diesem Computer, solange du ihn nicht teilst.
diagnostics-collecting = { $name } wird erstellt…
# $step counts from 1.
diagnostics-collecting-step = { $name } wird erstellt ({ $step } von { $count })…
diagnostics-collecting-detail = Startest du einen anderen Bericht, wird dieser abgebrochen.
diagnostics-empty = { $name } hat keine Ausgabe erzeugt.
diagnostics-unavailable = { $name } ist auf diesem System nicht verfügbar.
diagnostics-unavailable-detail = Das nötige Programm ist nicht installiert.
diagnostics-dismissed = { $name } wurde nicht erstellt: Du hast die Passwortabfrage geschlossen.
diagnostics-denied = { $name } wurde nicht erstellt: Dein Konto hat keinen Lesezugriff.
diagnostics-no-agent = { $name } wurde nicht erstellt: Die Passwortabfrage konnte nicht angezeigt werden, da in dieser Sitzung kein polkit-Agent läuft.
diagnostics-helper-missing = { $name } wurde nicht erstellt: Das Hilfsprogramm von Garuda Toolbox fehlt. Installiere Garuda Toolbox neu.
# $reason is the program's own words.
diagnostics-failed = { $name } ist fehlgeschlagen: { $reason }
diagnostics-report-label = Diagnosebericht

diagnostics-copy = Bericht kopieren
diagnostics-share = Bericht teilen…
diagnostics-share-tooltip = Lädt den Bericht verschlüsselt auf bin.garudalinux.org hoch und gibt dir einen Link zum Teilen.
diagnostics-report-copied = Bericht in die Zwischenablage kopiert.
