# The catalog toggle pages (Components, …).

toggles-search = Diese Seite durchsuchen
# The chip that shows every section.
toggles-all = Alle
toggles-search-hint = Diese Seite durchsuchen…
# $search is what the user typed.
toggles-nothing-matches-detail = Kein Eintrag dieser Seite enthält „{ $search }“ im Namen oder in der Beschreibung.
# A chip picked a section without a match while other sections have some. $section is the section's title, $search what the user typed.
toggles-nothing-in-section = Keine Treffer in { $section }
toggles-nothing-in-section-detail = Andere Abschnitte haben Einträge mit „{ $search }“ im Namen oder in der Beschreibung.
# $count is how many entries of the whole page match the search.
toggles-show-all-matches =
    { $count ->
        [one] Den Treffer zeigen
       *[other] Alle { $count } Treffer zeigen
    }
# $on and $total are counts of the entries of a section, a chip or the page, shown as "2/8".
toggles-section-count = { $on }/{ $total }
# The count while some states could not be read, so how many are on is not known. $total is the count of entries.
toggles-section-count-unread = ?/{ $total }
toggles-unknown =
    { $count ->
        [one] Der Zustand eines Eintrags konnte nicht gelesen werden. Sein Schalter zeigt eventuell den falschen Zustand.
       *[other] Der Zustand von { $count } Einträgen konnte nicht gelesen werden. Die Schalter zeigen eventuell den falschen Zustand.
    }
toggles-state-unknown = Zustand unbekannt.
# $names lists the titles of the entries, joined by commas.
toggles-needs = Benötigt { $names }
toggles-needs-any = Benötigt eines davon: { $names }
toggles-turns-on = Wird beim Übernehmen eingeschaltet
toggles-enable = Aktivieren
toggles-disable = Deaktivieren
toggles-turns-off = Wird beim Übernehmen ausgeschaltet
# Beside the title of a setting that is on, as "Installed" is beside a package.
toggles-enabled = Aktiviert
toggles-dismiss = Ausblenden

# Why the catalog could not be loaded.
toggles-problem = Die Liste konnte nicht geladen werden.
toggles-problem-unreadable = Eine Katalogdatei konnte nicht gelesen werden.
toggles-problem-syntax = Eine Katalogdatei ist beschädigt.
toggles-problem-invalid = Eine Katalogdatei enthält einen ungültigen Eintrag.
toggles-problem-reason = Die Liste konnte nicht geladen werden. { $reason }

# Eine Katalogdatei, die nicht geladen werden konnte, dann der Eintrag darin und sein Problem.
# $value ist der Wert der Datei, etwa ein Name, ein Pfad oder eine Zahl.
catalog-error-read = { $path }: { $error }
catalog-error-syntax = { $source }: { $error }
catalog-error-entry = { $source }: Eintrag „{ $id }“: { $problem }
catalog-check-package = das Paket { $name }
catalog-check-service = den Dienst { $name }
catalog-check-user-service = den Benutzerdienst { $name }
catalog-check-group = die Gruppe { $name }
catalog-check-user-config = die Benutzerkonfiguration { $edit }
catalog-problem-duplicate-check = ein anderer Eintrag überwacht { $value }
catalog-problem-bad-name = { $kind } `{ $value }` ist ungültig: { $reason }
catalog-problem-bad-id = die Kennung besteht aus Wörtern in Kleinbuchstaben, verbunden durch Bindestriche
catalog-problem-duplicate-id = die Kennung wird zweimal verwendet
catalog-problem-duplicate-path = ein anderer Eintrag zeigt auf „{ $value }“
catalog-problem-admin-group = die Gruppe „{ $value }“ gewährt Root-Rechte, sie wird nur auf der Seite Benutzer geändert
catalog-problem-empty-name = der Name ist leer
catalog-problem-empty-title = der Titel ist leer
catalog-problem-empty-description = die Beschreibung ist leer
catalog-problem-hidden-character = „{ $value }“ enthält ein verborgenes Zeichen
catalog-problem-empty-icon = das Symbol ist leer
catalog-problem-no-entries = keine Einträge
catalog-problem-empty-requirement = die Liste der Voraussetzungen ist leer
catalog-problem-unknown-requirement = benötigt „{ $value }“, das ist kein Eintrag dieses Katalogs
catalog-problem-requirement-repeats = benötigt „{ $value }“ zweimal
catalog-problem-requires-itself = der Eintrag benötigt sich selbst
catalog-problem-requirement-cycle = die Voraussetzungen bilden einen Kreis: { $value }
catalog-problem-no-files = keine Dateien
catalog-problem-outside-skel = „{ $value }“ liegt nicht unter /etc/skel/
catalog-problem-parent-component = „{ $value }“ enthält „..“
catalog-problem-not-writable = „{ $value }“ muss zugleich ein root-eigener Eintrag ohne read_only und eine Zeile von toolbox_plan::WRITABLE_CONFIG_FILES oder WRITABLE_CONFIG_FOLDERS sein
catalog-problem-read-only-writable = „{ $value }“ ist read_only, also darf es weder eine Zeile von toolbox_plan::WRITABLE_CONFIG_FILES oder WRITABLE_CONFIG_FOLDERS sein noch eine Datei in einem dieser Ordner
catalog-problem-read-only-home = read_only braucht einen root-eigenen Pfad
catalog-problem-empty-content = der Inhalt ist leer
catalog-problem-reload-without-user-config = reload braucht eine Prüfung user-file, user-ini oder user-json
catalog-problem-unknown-reload = unbekanntes reload „{ $value }“, bekannt ist kwin
catalog-problem-restart-without-user-config = restart-user-units braucht eine Prüfung user-file, user-ini oder user-json
catalog-problem-restart-repeats = restart-user-units nennt „{ $value }“ zweimal
catalog-problem-empty-desktop = die Liste der Desktops ist leer oder hat einen leeren Namen
catalog-problem-also-without-package = also braucht eine Paketprüfung
catalog-problem-empty-also = die Liste von also ist leer
catalog-problem-also-repeats = also nennt „{ $value }“ zweimal
catalog-problem-empty-conflicts = die Liste der Konflikte ist leer
catalog-problem-conflict-repeats = conflicts nennt „{ $value }“ zweimal
catalog-problem-unknown-conflict = steht im Konflikt mit „{ $value }“, das ist kein Eintrag dieses Katalogs
catalog-problem-conflicts-itself = der Eintrag steht im Konflikt mit sich selbst
catalog-problem-conflicts-requirement = steht im Konflikt mit „{ $value }“, den er auch benötigt
catalog-problem-bad-icon = das Symbol „{ $value }“ ist kein einfacher Dateiname
catalog-problem-missing-icon = das Symbol „{ $value }“ fehlt im Symbolordner
catalog-problem-bad-glyph = das Glyph „{ $value }“ ist kein freedesktop-Symbolname
catalog-problem-protected-also = das Paket „{ $value }“ bei also ist geschützt, der Eintrag ließe sich nie ausschalten
catalog-problem-bad-url = die URL „{ $value }“ ist kein einfacher https-Link
catalog-problem-bad-colour = „{ $value }“ ist keine Farbe der Form #rrggbb
catalog-problem-ansi-count = ansi hat { $value } Farben, es braucht 16
catalog-problem-empty-licence = die Lizenz ist leer
catalog-problem-home-path-empty = der Pfad im Home-Verzeichnis ist leer
catalog-problem-home-path-too-long = der Pfad im Home-Verzeichnis ist länger als { $value } Bytes
catalog-problem-home-path-absolute = der Pfad im Home-Verzeichnis `{ $value }` ist absolut, er muss relativ zum Home-Verzeichnis sein
catalog-problem-home-path-invalid-character = der Pfad im Home-Verzeichnis enthält `{ $value }`
catalog-problem-home-path-invalid-component = der Pfad im Home-Verzeichnis `{ $value }` hat einen leeren Bestandteil oder einen Bestandteil `.` oder `..`
catalog-problem-config-word-empty = das Konfigurationswort ist leer
catalog-problem-config-word-too-long = das Konfigurationswort ist länger als { $value } Bytes
catalog-problem-config-word-invalid-character = das Konfigurationswort enthält `{ $value }`
catalog-problem-config-word-padded = das Konfigurationswort `{ $value }` beginnt oder endet mit einem Leerzeichen
catalog-problem-config-text-too-large = die Datei ist größer als { $value } Bytes
catalog-problem-config-text-not-text = die Datei ist kein Text
catalog-problem-config-path-empty = der Pfad der Konfigurationsdatei ist leer
catalog-problem-config-path-too-long = der Pfad der Konfigurationsdatei ist länger als { $value } Bytes
catalog-problem-config-path-relative = der Pfad der Konfigurationsdatei `{ $value }` ist nicht absolut
catalog-problem-config-path-invalid-character = der Pfad der Konfigurationsdatei enthält `{ $value }`
catalog-problem-config-path-invalid-component = der Pfad der Konfigurationsdatei `{ $value }` hat einen leeren, zu langen, `.`- oder `..`-Bestandteil
catalog-problem-config-path-outside-roots = der Pfad der Konfigurationsdatei `{ $value }` liegt außerhalb der Ordner, die die Toolbox bearbeitet
catalog-problem-config-path-withheld = `{ $value }` wird nie als Konfigurationsdatei gelesen oder geschrieben
