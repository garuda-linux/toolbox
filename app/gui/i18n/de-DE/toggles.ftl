# The catalog toggle pages (Components, …).

toggles-search = Diese Seite durchsuchen
# The chip that shows every section.
toggles-all = Alle
toggles-search-hint = Diese Seite durchsuchen…
toggles-nothing-matches = Keine Treffer
# $search is what the user typed.
toggles-nothing-matches-detail = Kein Eintrag dieser Seite enthält „{ $search }“ im Namen oder in der Beschreibung.
toggles-clear-search = Suche leeren
# $on and $total are counts of the entries of a section, a chip or the page, shown as "2/8".
toggles-section-count = { $on }/{ $total }
toggles-unknown =
    { $count ->
        [one] Der Zustand eines Eintrags konnte nicht gelesen werden. Sein Schalter zeigt eventuell den falschen Zustand.
       *[other] Der Zustand von { $count } Einträgen konnte nicht gelesen werden. Ihre Schalter zeigen eventuell den falschen Zustand.
    }
toggles-state-unknown = Zustand unbekannt.
# $names lists the titles of the entries, joined by commas.
toggles-needs = Benötigt { $names }
toggles-needs-any = Benötigt eines davon: { $names }
toggles-turns-on = Wird beim Übernehmen eingeschaltet
toggles-turns-off = Wird beim Übernehmen ausgeschaltet
# $name is the entry's title.
toggles-website = Website von { $name }
toggles-dismiss = Ausblenden

# Why the catalog could not be loaded.
toggles-problem = Die Liste konnte nicht geladen werden.
toggles-problem-unreadable = Eine Katalogdatei konnte nicht gelesen werden.
toggles-problem-syntax = Eine Katalogdatei ist beschädigt.
toggles-problem-invalid = Eine Katalogdatei enthält einen ungültigen Eintrag.
