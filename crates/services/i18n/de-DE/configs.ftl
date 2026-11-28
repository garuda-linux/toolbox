
page-configs-subtitle = Bekannte Konfigurationsdateien des Systems und der installierten Apps.
configs-refresh = Neu einlesen
configs-search = Konfigurationsdateien suchen
configs-search-hint = Konfigurationsdateien suchen…
configs-search-detail = Durchsucht werden Namen, Pfade, Beschreibungen und die Dateien in Verzeichnissen.
configs-reading = Konfigurationsdateien werden gesucht…
configs-unreadable = Die Liste der Konfigurationsdateien konnte nicht gelesen werden
configs-none = Keine der aufgeführten Konfigurationsdateien ist auf diesem System vorhanden
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
configs-problem-dismissed = { $path } wurde nicht gelesen: Die Passwortabfrage wurde geschlossen.
configs-problem-denied = { $path } wurde nicht gelesen: Der Passwortdialog wurde geschlossen oder das Passwort nicht angenommen.
configs-problem-no-agent = { $path } wurde nicht gelesen, weil keine Passwortabfrage erscheinen konnte: In dieser Sitzung läuft kein polkit-Agent.
configs-problem-helper-missing = { $path } wurde nicht gelesen, weil das Hilfsprogramm von Garuda Toolbox fehlt. Garuda Toolbox neu installieren.
configs-problem-missing = { $path } existiert nicht mehr.
configs-problem-link = { $path } oder ein übergeordnetes Verzeichnis wurde während des Lesens durch einen Symlink ersetzt und deshalb nicht angetastet. Erneut prüfen und den Vorgang wiederholen.
configs-problem-dangling = { $path } ist ein Symlink, der ins Leere führt. Es gibt nichts zu bearbeiten.
configs-problem-outside = { $path } ist ein Symlink aus den Konfigurationsverzeichnissen des Systems heraus, zu einer Paketdatei oder einer generierten Datei. Solche Dateien werden hier nicht bearbeitet.
configs-problem-not-regular = { $path } ist keine reguläre Datei.
configs-problem-too-large = { $path } ist größer als 1 MiB und kann hier nicht bearbeitet werden.
configs-problem-not-text = { $path } ist keine Textdatei und kann hier nicht bearbeitet werden.
configs-problem-no-editor = Es ist kein Terminal-Editor installiert. micro, nano oder vi installieren und danach erneut versuchen.
configs-problem-busy = In der Konsole läuft bereits ein anderer Vorgang. Erneut versuchen, wenn er abgeschlossen ist.
configs-problem-editor-failed = Der Editor wurde mit einem Fehler beendet. { $path } bleibt unverändert.
configs-problem-write-failed = { $path } wurde nicht geschrieben. Den Grund zeigt die Konsole. Die Änderungen bleiben erhalten.
configs-problem-copy-failed = Die Arbeitskopie konnte nicht verwendet werden.
configs-problem-broken = { $path } wurde nicht gelesen, weil sich pkexec auf diesem System nicht ausführen lässt. Details stehen im Log.
configs-problem-failed = Bei { $path } ist unerwartet ein Fehler aufgetreten. Erneut versuchen.
configs-problem-helper-start = { $path } wurde nicht gelesen, weil das Hilfsprogramm von Garuda Toolbox nicht gestartet werden konnte.
configs-problem-helper-timed-out = { $path } wurde nicht gelesen, weil das Hilfsprogramm von Garuda Toolbox nicht rechtzeitig geantwortet hat. Erneut versuchen.
configs-problem-helper-too-long = { $path } wurde nicht gelesen, weil die Antwort des Hilfsprogramms von Garuda Toolbox zu lang ist.
configs-problem-helper-pkexec = { $path } wurde nicht gelesen, weil pkexec hier nicht ausgeführt werden kann.
configs-problem-helper-ended = { $path } wurde nicht gelesen, weil das Hilfsprogramm von Garuda Toolbox fehlgeschlagen ist.
configs-problem-helper-unreadable = { $path } wurde nicht gelesen, weil die Antwort des Hilfsprogramms von Garuda Toolbox nicht lesbar ist.
configs-problem-helper-stopped = { $path } wurde nicht gelesen, weil das Lesen beendet wurde, bevor das Hilfsprogramm geantwortet hat.
configs-problem-read-only = Diese Datei legt fest, wer das System verwalten darf oder was als root läuft, und wird deshalb hier nur angezeigt. Die Bearbeitung erfolgt in einem Terminal mit sudo.
configs-problem-read-failed = { $path } konnte nicht gelesen werden.
configs-problem-no-home = { $path } wurde nicht gelesen, weil dieses Konto kein Home-Verzeichnis hat.
configs-problem-cancelled = Das Lesen von { $path } wurde abgebrochen.
configs-view-too-large = { $path } ist größer als 1 MiB und kann hier nicht angezeigt werden.
configs-view-not-text = { $path } ist keine Textdatei und kann hier nicht angezeigt werden.
configs-view-busy = Es wird noch eine andere Datei gelesen. Erneut versuchen, sobald sie angezeigt wird.

# $editor names the editor, $backup where the previous version is kept.
configs-status-reading = { $path } wird gelesen…
configs-status-editing = { $path } ist in der Konsole mit { $editor } geöffnet. Zum Weitermachen den Editor speichern und beenden.
configs-status-comparing = Die Arbeitskopie wird mit { $path } verglichen…
configs-status-review = Die Änderungen an { $path } warten auf eine Prüfung.
configs-status-writing = { $path } wird geschrieben…
configs-status-saved = { $path } gespeichert. Die vorherige Version wurde als { $backup } gesichert.
configs-status-unchanged = { $path } ist unverändert. Es gibt nichts zu speichern.
configs-show-console = Konsole anzeigen
configs-cancel-edit = Bearbeitung abbrechen
configs-review = Änderungen prüfen

configs-review-title = Änderungen an { $path } speichern?
# $target is the file the link leads to.
configs-review-link = { $path } ist ein Symlink. Beim Speichern wird die Datei geschrieben, auf die er verweist, { $target }. Der Symlink selbst bleibt unverändert.
configs-review-summary = Hinzugefügte Zeilen: { $added }. Entfernte Zeilen: { $removed }. Zum Speichern ist ein Administratorpasswort nötig. Die aktuelle Fassung wird als { $backup } gesichert.
configs-review-summary-home = Hinzugefügte Zeilen: { $added }. Entfernte Zeilen: { $removed }. Die aktuelle Fassung wird als { $backup } gesichert.
configs-review-write-failed = { $path } wurde nicht geschrieben. Den Grund zeigt die Konsole. Wurde die Datei inzwischen geändert, sollte erneut verglichen werden, damit die Änderung mit dem aktuellen Stand abgeglichen wird.
configs-review-diff = Die Änderungen: hinzugefügte Zeilen beginnen mit einem Pluszeichen, entfernte mit einem Minuszeichen
configs-save = Speichern
configs-review-cancel = Abbrechen
configs-edit-again = Weiter bearbeiten
configs-compare-again = Erneut vergleichen
configs-discard = Verwerfen
configs-discard-ask = Verwerfen…
configs-discard-title = Änderungen verwerfen?
configs-discard-question = Die Änderungen an { $path } gehen verloren. Die Datei bleibt unverändert.
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
# Why a click on a row did nothing.
configs-refused-unknown = Diese Datei kann hier nicht geöffnet werden.
configs-refused-busy = In der Konsole läuft bereits ein anderer Vorgang. Erneut versuchen, wenn er abgeschlossen ist.
configs-refused-read-only = Diese Datei legt fest, wer das System verwalten darf oder was als root läuft, und wird deshalb hier nur angezeigt. Die Bearbeitung erfolgt in einem Terminal mit sudo.
