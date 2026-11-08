# The accessible name of the terminal, as Orca announces it.
console-terminal = Konsolenausgabe
console-leave-hint = Strg+Tab verlässt das Terminal.
console-idle-hint = Hier erscheint die Ausgabe, sobald du Änderungen übernimmst.
console-printed-nothing = Die Änderungen haben keine Ausgabe erzeugt.
console-cancel = Abbrechen
console-hide = Konsole ausblenden
console-resize = Höhe der Konsole ändern

console-output-menu = Aktionen für die Konsolenausgabe
console-copy-output = Ausgabe kopieren
console-share-output = Ausgabe teilen…
console-clear-output = Ausgabe leeren
console-output-copied = Ausgabe in die Zwischenablage kopiert.

console-status-running = Änderungen werden übernommen…
console-status-succeeded = Alle Änderungen übernommen
# Steps that found nothing to do, such as a file already as wanted.
console-status-partly = { $count ->
        [one] Übernommen, 1 Schritt war schon erledigt
       *[other] Übernommen, { $count } Schritte waren schon erledigt
    }
console-status-nothing = Keine Änderungen nötig
# A dry run: no step ran.
console-status-tested = Testmodus: Es wurde nichts geändert
console-status-failed = Nicht alle Änderungen wurden übernommen

console-busy = Es werden noch andere Änderungen übernommen. Versuche es danach erneut.
console-cancel-waits = Der Vorgang wird abgebrochen, sobald die Passwortabfrage beantwortet ist.
# $reason is the helper's own words, shown as they are.
console-rejected = Das Hilfsprogramm hat die Änderungen abgelehnt: { $reason }

step-waiting = Wartet
step-running = Läuft
step-succeeded = Erledigt
# The step found nothing to do, such as a file already as wanted.
step-nothing-to-do = Nichts zu tun
# A dry run runs no step.
step-not-run = Nicht ausgeführt (Testmodus)
step-cancelled = Abgebrochen
step-failed-exit = Fehlgeschlagen (Exit-Code { $code })
step-failed-signal = Fehlgeschlagen (Signal { $signal })
step-failed-unknown = Fehlgeschlagen

# pkexec did not start the helper.
refusal-dismissed = Die Passwortabfrage wurde geschlossen. Es wurde nichts geändert.
refusal-denied = Der Passwortdialog wurde geschlossen oder das Passwort nicht angenommen. Es wurde nichts geändert.
refusal-no-agent = Die Passwortabfrage konnte nicht angezeigt werden. Melde dich in einer Desktop-Sitzung mit polkit-Agent an.
refusal-helper-missing = Das Hilfsprogramm von Garuda Toolbox fehlt. Installiere garuda-toolbox neu.
refusal-broken = pkexec lässt sich auf diesem System nicht ausführen. Details stehen im Log.

# The plan failed outside any step. $detail is the system's words.
helper-missing-secret = Für Schritt { $detail } fehlt ein Passwort.
helper-start = Das Hilfsprogramm ließ sich nicht starten: { $detail }
helper-pkexec = pkexec kann das Hilfsprogramm nicht starten: { $detail }
helper-no-pipes = Das Hilfsprogramm wurde ohne Ein- und Ausgabe gestartet.
helper-lost = Die Verbindung zum Hilfsprogramm wurde unterbrochen: { $detail }
helper-ended = Das Hilfsprogramm wurde beendet, bevor die Änderungen abgeschlossen waren: { $detail }
helper-user-steps = Die Schritte in der App wurden unerwartet abgebrochen: { $detail }

# Desktop notifications when the window is in the background.
notify-applied = Änderungen übernommen
notify-failed = Fehler beim Übernehmen
notify-applied-body = Garuda Toolbox hat deine Änderungen übernommen.
notify-failed-body = Ein Schritt ist fehlgeschlagen. Details findest du in der Konsole.

# An install failed as a mirror no longer has a package file.
console-mirror-missing = Auf dem Spiegelserver fehlt eine Datei, die diese Installation benötigt, da die Paketdatenbanken älter sind als der Spiegelserver. Eine Systemaktualisierung bringt sie auf den aktuellen Stand.
console-update-and-retry = System aktualisieren und erneut versuchen
