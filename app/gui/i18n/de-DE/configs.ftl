
page-configs-subtitle = Bekannte Konfigurationsdateien des Systems und deiner Apps.
configs-refresh = Neu einlesen
configs-search = Konfigurationsdateien suchen
configs-search-hint = Konfigurationsdateien suchen…
configs-search-detail = Durchsucht werden Namen, Pfade, Beschreibungen und die Dateien in Verzeichnissen.
configs-clear-search = Suche leeren
configs-reading = Konfigurationsdateien werden gesucht…
configs-unreadable = Die Liste der Konfigurationsdateien konnte nicht gelesen werden
configs-none = Keine der aufgeführten Konfigurationsdateien ist auf diesem System vorhanden
configs-no-match = Keine passende Konfigurationsdatei
configs-dismiss = Ausblenden
configs-list = Dateien: { $section }

configs-open-in-app = In der App öffnen
configs-edit = Bearbeiten
configs-view = Ansehen
configs-show-files = Dateien anzeigen
configs-hide-files = Dateien ausblenden
# $count is how many files the folder holds.
configs-files = { $count ->
        [one] 1 Datei
       *[other] { $count } Dateien
    }
configs-wiki-open = Wiki-Seite öffnen
configs-admin = Zum Speichern ist ein Administratorpasswort nötig
configs-read-only = Nur Ansicht: Die Datei legt fest, wer das System verwalten darf oder was als root läuft
# $target is where the link leads.
configs-link = Symlink zu { $target }
configs-not-edited-dangling = Hier nicht bearbeitbar: Der Symlink führt ins Leere
configs-not-edited-not-regular = Hier nicht bearbeitbar: Der Symlink führt zu einem Verzeichnis oder einem anderen Objekt, das keine Datei ist
configs-not-edited-outside = Hier nicht bearbeitbar: Der Symlink führt aus den Konfigurationsverzeichnissen des Systems heraus, zu einer Paketdatei oder einer generierten Datei

# $path names the file.
configs-problem-dismissed = { $path } wurde nicht gelesen: Du hast die Passwortabfrage geschlossen.
configs-problem-denied = { $path } wurde nicht gelesen, weil dafür ein Administratorpasswort nötig ist.
configs-problem-no-agent = { $path } wurde nicht gelesen, weil keine Passwortabfrage erscheinen konnte: In dieser Sitzung läuft kein polkit-Agent.
configs-problem-helper-missing = { $path } wurde nicht gelesen, weil das Hilfsprogramm von Garuda Toolbox fehlt. Installiere Garuda Toolbox neu.
configs-problem-missing = { $path } existiert nicht mehr.
configs-problem-link = { $path } oder ein übergeordnetes Verzeichnis wurde während des Lesens durch einen Symlink ersetzt und deshalb nicht angetastet. Lies die Dateien neu ein und versuche es erneut.
configs-problem-dangling = { $path } ist ein Symlink, der ins Leere führt. Es gibt nichts zu bearbeiten.
configs-problem-outside = { $path } ist ein Symlink aus den Konfigurationsverzeichnissen des Systems heraus, zu einer Paketdatei oder einer generierten Datei. Solche Dateien werden hier nicht bearbeitet.
configs-problem-not-regular = { $path } ist keine reguläre Datei.
configs-problem-too-large = { $path } ist größer als 1 MiB und kann hier nicht bearbeitet werden.
configs-problem-not-text = { $path } ist keine Textdatei und kann hier nicht bearbeitet werden.
configs-problem-no-editor = Es ist kein Terminal-Editor installiert. Installiere micro, nano oder vi und versuche es erneut.
configs-problem-busy = In der Konsole läuft gerade etwas anderes. Versuche es erneut, wenn es abgeschlossen ist.
configs-problem-editor-failed = Der Editor wurde mit einem Fehler beendet. { $path } bleibt unverändert.
configs-problem-write-failed = { $path } wurde nicht geschrieben. Den Grund zeigt die Konsole. Deine Änderungen bleiben erhalten.
configs-problem-copy-failed = Die Arbeitskopie konnte nicht verwendet werden.
configs-problem-unknown = Diese Datei kann hier nicht geöffnet werden.
configs-problem-read-only = Diese Datei legt fest, wer das System verwalten darf oder was als root läuft, und wird deshalb hier nur angezeigt. Bearbeite sie in einem Terminal mit sudo.
configs-problem-other = { $path } konnte nicht gelesen werden.
configs-view-too-large = { $path } ist größer als 1 MiB und kann hier nicht angezeigt werden.
configs-view-not-text = { $path } ist keine Textdatei und kann hier nicht angezeigt werden.
configs-view-busy = Es wird noch eine andere Datei gelesen. Versuche es erneut, sobald sie angezeigt wird.

# $editor names the editor, $backup where the previous version is kept.
configs-status-reading = { $path } wird gelesen…
configs-status-editing = { $path } ist in der Konsole mit { $editor } geöffnet. Speichere und beende den Editor, um weiterzumachen.
configs-status-comparing = Deine Kopie wird mit { $path } verglichen…
configs-status-review = Deine Änderungen an { $path } sind bereit zur Prüfung.
configs-status-writing = { $path } wird geschrieben…
configs-status-saved = { $path } gespeichert. Die vorherige Version wurde als { $backup } gesichert.
configs-status-unchanged = { $path } ist unverändert. Es gibt nichts zu speichern.
configs-show-console = Konsole anzeigen
configs-cancel-edit = Bearbeitung abbrechen
configs-review = Änderungen prüfen

configs-review-title = Änderungen an { $path } speichern?
# $target is the file the link leads to.
configs-review-link = { $path } ist ein Symlink. Gespeichert wird in sein Ziel { $target }. Der Symlink selbst bleibt unverändert.
configs-review-summary = Hinzugefügte Zeilen: { $added }. Entfernte Zeilen: { $removed }. Zum Speichern ist ein Administratorpasswort nötig. Die aktuelle Fassung wird als { $backup } gesichert.
configs-review-summary-home = Hinzugefügte Zeilen: { $added }. Entfernte Zeilen: { $removed }. Die aktuelle Fassung wird als { $backup } gesichert.
configs-review-write-failed = { $path } wurde nicht geschrieben. Den Grund zeigt die Konsole. Wurde die Datei inzwischen geändert, vergleiche erneut, um deine Änderungen mit dem aktuellen Stand abzugleichen.
configs-review-diff = Die Änderungen: hinzugefügte Zeilen beginnen mit einem Pluszeichen, entfernte mit einem Minuszeichen
configs-save = Speichern
configs-edit-again = Weiter bearbeiten
configs-compare-again = Erneut vergleichen
configs-discard = Verwerfen
configs-discard-ask = Verwerfen…
configs-discard-title = Änderungen verwerfen?
configs-discard-question = Deine Änderungen an { $path } gehen verloren. Die Datei bleibt unverändert.
configs-discard-keep = Weiter bearbeiten
configs-review-close = Schließen und Änderungen behalten
configs-review-lines =
    { $count ->
        [one] Zeile { $first }
       *[other] Zeilen { $first } bis { $last }
    }

configs-view-title = Inhalt von { $path }
configs-view-link = { $path } ist ein Symlink. Angezeigt wird der Inhalt seines Ziels { $target }.
configs-copy-all = Alles kopieren
configs-copied = In die Zwischenablage kopiert.
configs-close = Schließen
