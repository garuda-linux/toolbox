page-locales-subtitle = Verfügbare Sprachen, Zahlen- und Datumsformate sowie Übersetzungen für die installierten Programme.
locales-refresh = Neu einlesen
locales-tabs = Reiter der Sprachen-Seite
locales-tab-locales = Locales
locales-tab-packs = Sprachpakete
locales-queued = Wird beim Übernehmen geändert

locales-search = Locales und Pakete durchsuchen
locales-search-hint = Suchen…
locales-filter-utf8 = Nur UTF-8
locales-filter-enabled = Nur aktivierte
locales-list = Locales
locales-reading = Locales werden eingelesen…
locales-none = Keine passende Locale
locales-none-detail = Die Suche umfasst Namen von Sprachen, Ländern, Locales und Zeichensätzen.
locales-unavailable = Locales sind auf diesem System nicht verfügbar
locales-unavailable-detail = Dieses System erzeugt seine Locales nicht aus /etc/locale.gen.
locales-enabled = Aktiviert
locales-disabled = Deaktiviert
locales-title-territory = { $language } ({ $territory })
locales-detail = { $code }, { $charset }
locales-detail-system = { $code }, { $charset }, Systemsprache
locales-detail-code = { $code }
locales-detail-code-system = { $code }, Systemsprache

locales-system = Systemsprache
locales-system-detail = Die Systemsprache gilt ab der nächsten Anmeldung. Zur Auswahl stehen nur aktivierte Locales. Zuerst muss eine Locale in der Liste unten aktiviert werden.
locales-system-unset = Nicht gesetzt
locales-system-choice = { $language }, { $code }
locales-system-choice-territory = { $language } ({ $territory }), { $code }

# Why a click was not queued.
locales-refused-system-locale = { $name } ist die Systemsprache. Vor dem Deaktivieren muss eine andere Systemsprache gewählt werden.
locales-refused-last-locale = Mindestens eine Locale muss aktiviert bleiben.
locales-refused-not-enabled = { $name } ist nicht aktiviert. Die Locale muss zuerst aktiviert werden.
locales-dismiss = Ausblenden

locales-packs-intro =
    Sprachpakete für die aktivierten Locales und installierten Programme. { $count ->
        [one] Ein Paket ist nicht installiert.
       *[other] { $count } Pakete sind nicht installiert.
    }
locales-packs-intro-plain = Sprachpakete für die aktivierten Locales und installierten Programme.
locales-packs-add-missing = Alle fehlenden vormerken
locales-packs-sort = Sprachpakete sortieren
locales-packs-sort-package = Paket
locales-packs-sort-program = Programm
locales-packs-sort-language = Sprache
locales-packs-sort-selected = Ausgewählte zuerst
locales-packs-list = Sprachpakete
locales-packs-reading = Sprachpakete werden eingelesen…
locales-packs-none = Keine Sprachpakete gefunden
locales-packs-none-detail = Kein installiertes Programm hat ein Sprachpaket für eine aktivierte Locale.
locales-packs-no-match = Kein passendes Sprachpaket
locales-packs-no-match-detail = Die Suche umfasst Namen von Paketen, Programmen, Sprachen und Ländern.
locales-packs-unavailable = Sprachpakete sind auf diesem System nicht verfügbar
locales-packs-unavailable-detail = Die Paketdatenbanken von pacman konnten nicht gelesen werden.
locales-pack-installed = Installiert
locales-pack-missing = Nicht installiert
locales-pack-detail = Für { $base }: { $locale }
