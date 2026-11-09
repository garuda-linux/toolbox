# The Packages page. Package names, descriptions, versions and file
# paths come from pacman's databases as they are.

packages-refresh = Neu einlesen
packages-search = Pakete suchen
packages-search-hint = Pakete suchen…
packages-loading = Paketlisten werden gelesen…
packages-unavailable = Keine Pakete verfügbar. Die Paketdatenbanken von pacman ließen sich nicht lesen.
packages-failed = Die Paketdatenbanken ließen sich nicht lesen.

packages-filter-installed = Installiert
packages-filter-updates = Updates
packages-filter-orphans = Verwaist
packages-filter-all = Alle

packages-none = Keine Pakete
packages-none-detail = Die Repositorys bieten keine an und es ist keines installiert.
packages-no-updates = Keine ausstehenden Updates
packages-no-updates-detail = Die Startseite sucht nach Updates. Installierte Pakete mit neuerer Version erscheinen hier.
packages-no-orphans = Keine verwaisten Pakete
packages-no-orphans-detail = Jedes als Abhängigkeit installierte Paket wird noch von einem anderen gebraucht.
packages-nothing-matches = Kein Paket passt zu „{ $search }“
packages-nothing-matches-detail = Durchsucht Namen und Beschreibungen im gewählten Filter.
packages-clear-search = Suche leeren

# Why a click queued nothing.
packages-refused-not-offered = { $package } lässt sich nicht installieren: Kein eingerichtetes Repository bietet es an.
packages-refused-unread = { $package } lässt sich noch nicht vormerken: Die installierten Pakete werden noch eingelesen.
packages-refused-running-kernel = { $package } ist der laufende Kernel und bleibt deshalb installiert.
packages-refused-running-unknown = { $package } bleibt installiert. Der laufende Kernel lässt sich nicht ermitteln, deshalb kann kein Kernel entfernt werden. Starte den Computer neu und versuch es noch einmal.
packages-refused-last-kernel = { $package } ist der letzte installierte Kernel. Installiere einen anderen, bevor du ihn entfernst.
packages-dismiss = Ausblenden
# A row button's name for screen readers: $action is the button, such as Install, $name the package.
packages-action-name = { $action } { $name }

packages-install = Installieren
packages-remove = Entfernen
packages-take-back = Zurücknehmen

packages-tag-dependency = Abhängigkeit
packages-tag-foreign = Fremdes Paket
packages-installed = Installiert
packages-installs = Zu installieren
packages-removes = Zu entfernen
# In place of a repository's name.
packages-no-repository = kein Repository

packages-remove-title = { $package } entfernen?
packages-remove-alone = Nur dieses Paket wird entfernt. Es hat keine Abhängigkeiten mitgebracht, die sonst kein Paket braucht.
packages-remove-along = { $count ->
        [one] Zusätzlich wird 1 Abhängigkeit entfernt, die kein anderes Paket braucht:
       *[other] Zusätzlich werden { $count } Abhängigkeiten entfernt, die kein anderes Paket braucht:
    }
packages-remove-note = Das Entfernen bleibt vorgemerkt. Sobald es läuft, zeigt die Konsole, was pacman tatsächlich entfernt.
packages-cancel = Abbrechen

# The -name texts are what screen readers say.
packages-back = ← Pakete
packages-back-name = Zurück zu den Paketen
# $package is the package the page was opened from.
packages-back-to = ← { $package }
packages-back-to-name = Zurück zu { $package }
packages-more = +{ $count } weitere
packages-more-name = { $count } weitere anzeigen
packages-details-loading = Paket wird eingelesen…
packages-details-failed = Das Paket ließ sich nicht lesen.
packages-gone = { $package } steht in keiner Paketdatenbank mehr
packages-gone-detail = Es wurde entfernt oder ein Repository bietet es nicht mehr an.
packages-copy = { $label } kopieren
packages-copied = In die Zwischenablage kopiert.
packages-value-none = Keine

packages-info-repository = Repository
packages-info-installed-version = Installierte Version
packages-info-url = Website
packages-info-licenses = Lizenzen
packages-info-groups = Gruppen
packages-info-size = Installationsgröße
packages-info-date = Installationsdatum
packages-info-reason = Installationsgrund
packages-info-provides = Stellt bereit
packages-info-conflicts = Steht in Konflikt mit
packages-info-replaces = Ersetzt
packages-reason-explicit = Ausdrücklich installiert
packages-reason-dependency = Als Abhängigkeit installiert

packages-depends = Hängt ab von
packages-optional = Optionale Abhängigkeiten
packages-required-by = Benötigt von
# The count of installed files follows it, dimmed.
packages-files = Dateien
packages-copy-list = Liste kopieren
packages-files-copied = Die Dateiliste wurde in die Zwischenablage kopiert.
# What screen readers call the list of files.
packages-files-list = Installierte Dateien

packages-history = Verlauf
packages-history-all = Alle anzeigen
packages-history-failed = Das pacman-Log ließ sich nicht lesen.
packages-history-none = Keine Änderungen gefunden
packages-history-installed = Installiert
packages-history-upgraded = Aktualisiert
packages-history-downgraded = Herabgestuft
packages-history-reinstalled = Erneut installiert
packages-history-removed = Entfernt

# Names, descriptions and versions come from the AUR's metadata dump.
packages-filter-aur = AUR
packages-aur-off = AUR-Pakete sind ausgeblendet.
packages-aur-off-settings = In den Einstellungen einblenden
packages-aur-loading = Die Paketliste des AUR wird eingelesen…
packages-aur-failed = Die Paketliste des AUR konnte nicht heruntergeladen werden. Der nächste Versuch folgt in einem Tag.
packages-aur-waiting = Die Paketliste des AUR liegt noch nicht vor. Der letzte Download ist fehlgeschlagen, der nächste folgt einen Tag danach.
# $date is the day the list was downloaded, such as 2025-01-10.
packages-aur-note = Stand: { $date }. AUR-Pakete sind Skripte der Community, die auf deinem Computer gebaut werden.
# After the note when the last download of a newer list failed.
packages-aur-stale = Die Liste konnte nicht aktualisiert werden und ist vielleicht veraltet.
packages-tag-aur = AUR
packages-aur-out-of-date = Als veraltet markiert
packages-aur-builds = Zu bauen
packages-aur-votes = { $count ->
        [one] 1 Stimme
       *[other] { $count } Stimmen
    }

packages-aur-install-title = { $package } aus dem AUR bauen?
packages-aur-install-body = Das Paket wird aus einem Skript der Community auf deinem Computer gebaut. paru zeigt dir das Build-Skript vorher in der Konsole.

packages-aur-maintainer = Betreuer
packages-aur-orphaned = Keiner (verwaist)
packages-aur-votes-label = Stimmen
packages-aur-popularity = Beliebtheit
packages-aur-submitted = Eingereicht
packages-aur-updated = Zuletzt geändert
packages-aur-out-of-date-since = Veraltet seit
packages-aur-page = AUR-Seite
packages-aur-make-depends = Build-Abhängigkeiten
