# The catalog toggle pages (Components, …).

toggles-search = Diese Seite durchsuchen
# The chip that shows every section.
toggles-all = Alle
toggles-search-hint = Diese Seite durchsuchen…
toggles-nothing-matches = Keine Treffer
# $search is what the user typed.
toggles-nothing-matches-detail = Kein Eintrag dieser Seite enthält „{ $search }“ im Namen oder in der Beschreibung.
toggles-clear-search = Suche leeren
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
       *[other] Der Zustand von { $count } Einträgen konnte nicht gelesen werden. Ihre Schalter zeigen eventuell den falschen Zustand.
    }
toggles-state-unknown = Zustand unbekannt.
# $names lists the titles of the entries, joined by commas.
toggles-needs = Benötigt { $names }
toggles-needs-any = Benötigt eines davon: { $names }
toggles-turns-on = Wird beim Übernehmen eingeschaltet
toggles-enable = Aktivieren
toggles-disable = Deaktivieren
toggles-turns-off = Wird beim Übernehmen ausgeschaltet
toggles-dismiss = Ausblenden

# Why the catalog could not be loaded.
toggles-problem = Die Liste konnte nicht geladen werden.
toggles-problem-unreadable = Eine Katalogdatei konnte nicht gelesen werden.
toggles-problem-syntax = Eine Katalogdatei ist beschädigt.
toggles-problem-invalid = Eine Katalogdatei enthält einen ungültigen Eintrag.
