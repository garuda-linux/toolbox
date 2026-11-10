
page-services-subtitle = System- und Benutzerdienste starten, stoppen und aktivieren.
services-system = System
services-user = Benutzer
services-scope = Angezeigte Dienste
services-search = Dienste durchsuchen
services-search-hint = Dienste durchsuchen…
services-hide-disabled = Deaktivierte ausblenden
services-hide-inactive = Inaktive ausblenden
services-refresh = Neu einlesen
services-sort = Dienste sortieren
services-sort-name = Name
services-sort-state = Probleme zuerst
services-reading = Dienste werden eingelesen…
services-none = Keine passenden Dienste.
services-unavailable = systemd ist nicht erreichbar.
services-denied = Keine Berechtigung, die Dienste zu lesen.
services-failed = Die Dienste konnten nicht gelesen werden.
services-retry = Erneut versuchen
services-none-detail = Ändere die Suche oder die Filter, um mehr Dienste zu sehen.
# Why an action of a unit's menu cannot be picked, shown on hover.
services-why-busy = Erst möglich, wenn die laufenden Änderungen abgeschlossen sind.
services-why-masked = Ein maskierter Dienst kann nicht starten. Demaskiere ihn zuerst.
services-why-not-running = Nur ein laufender Dienst kann seine Konfiguration neu laden.
services-why-running = Der Dienst läuft bereits.
services-why-stopped = Der Dienst läuft nicht.
services-at-boot = Beim Systemstart
# $unit is a unit name such as sshd.service.
services-start-at-boot = { $unit } beim Systemstart starten

# $name is the unit's name, such as sshd.service.
services-actions = Aktionen für { $name }
services-now = Sofort
services-with-apply = Beim Übernehmen
services-start = Starten
services-stop = Stoppen
services-restart = Neu starten
services-reload = Neu laden
services-mask = Maskieren
services-unmask = Demaskieren

services-enables = Wird beim Übernehmen aktiviert
services-disables = Wird beim Übernehmen deaktiviert
services-masks = Wird beim Übernehmen maskiert
services-unmasks = Wird beim Übernehmen demaskiert

# The unit's state. A detail in brackets says what it does right now.
services-active-active = Läuft
services-active-failed = Fehlgeschlagen
services-active-activating = Wird gestartet
services-active-deactivating = Wird gestoppt
services-active-reloading = Wird neu geladen
services-active-inactive = Gestoppt
services-active-maintenance = Räumt auf
services-active-refreshing = Wird aktualisiert
services-active-other = Unbekannter Zustand
services-finished = Aktiv (fertig)
# $state is the state above, $detail one of the words below.
services-state-detail = { $state } ({ $detail })
services-sub-running = läuft
services-sub-exited = beendet
services-sub-listening = lauscht
services-sub-waiting = wartet
services-sub-active = aktiv
services-sub-mounted = eingehängt
services-sub-plugged = angeschlossen
services-sub-elapsed = abgelaufen
services-sub-abandoned = aufgegeben
services-sub-failed = fehlgeschlagen
services-sub-cleaning = räumt auf
services-sub-dead = läuft nicht
services-sub-starting = startet
services-sub-stopping = stoppt
services-sub-reloading = lädt neu
services-sub-restarting = startet neu

services-file-static = Statisch
services-file-masked = Maskiert
services-file-indirect = Indirekt
services-file-generated = Erzeugt
services-file-linked = Verlinkt
services-file-alias = Alias
services-file-transient = Temporär
services-file-bad = Unlesbare Datei
services-file-unknown = Dateizustand unbekannt
services-file-other = Anderer Dateizustand
services-file-static-hint = Kann nicht aktiviert werden. Er startet, wenn ein anderer Dienst oder das System ihn braucht.
services-file-masked-hint = Gesperrt: Er lässt sich überhaupt nicht starten, auch nicht durch einen anderen Dienst.
services-file-indirect-hint = Startet über einen anderen Dienst oder einen Timer, nicht von selbst.
services-file-generated-hint = Beim Start aus einer anderen Datei erzeugt, zum Beispiel /etc/fstab.
services-file-linked-hint = Die Unit-Datei liegt außerhalb der Dienstverzeichnisse und ist per Symlink eingebunden.
services-file-alias-hint = Ein weiterer Name für einen Dienst, der unter seinem eigenen Namen aufgeführt ist.
services-file-transient-hint = Von einem Programm zur Laufzeit erzeugt. Nach einem Neustart ist er verschwunden.
services-file-bad-hint = systemd konnte die Unit-Datei nicht lesen.
services-file-unknown-hint = Zu diesem Dienst meldet systemd keine Datei.
services-file-other-hint = systemd meldet einen Dateizustand, den diese App noch nicht kennt.
# $verb is the action (Stop, Restart, Mask), $unit the unit.
services-ask-title = { $unit }: { $verb }?
services-ask-why = Deine Sitzung hängt von { $unit } ab: Anmeldebildschirm, Netzwerk, Passwortabfrage oder Systembus. Stoppst du den Dienst, kann deine grafische Sitzung enden oder die Verbindung abbrechen.
services-ask-cancel = Abbrechen
