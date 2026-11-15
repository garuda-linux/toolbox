# The App Catalog page (Discover). App names and descriptions come from
# the catalog in English.

discover-search = Apps suchen
discover-search-hint = Apps suchen…
discover-sort = Apps sortieren
discover-all = Alle
discover-sort-name = Name
discover-sort-selected = Ausgewählte zuerst
discover-actions = Weitere Aktionen für { $name }
discover-unreadable = Die installierten Apps konnten nicht ermittelt werden. Die Zeilen zeigen daher nicht an, was installiert ist. Das Installieren einer bereits installierten App bewirkt nichts.
discover-problem = Einige Kategorien konnten nicht geladen werden. Die übrigen werden angezeigt.
discover-nothing-matches-detail = Die Suche umfasst Namen, Beschreibungen und Paketnamen in allen Kategorien.
# A chip picked a category without a match while others have some. $category is the category's name, $search what the user typed.
discover-nothing-in-category = Keine App in { $category } passt zu „{ $search }“
discover-nothing-in-category-detail = Andere Kategorien haben passende Apps.
# $count is how many apps of every category match the search.
discover-show-all-matches =
    { $count ->
        [one] Den Treffer zeigen
       *[other] Alle { $count } Treffer zeigen
    }
discover-none = Keine Apps vorhanden
discover-none-detail = Apps, die deine Repositorys nicht anbieten, sind ausgeblendet.

# The categories, by their catalog id.
discover-category-documents = Dokumente
discover-category-games = Spiele
discover-category-internet = Internet
discover-category-multimedia = Multimedia
discover-category-other = Sonstiges
discover-category-science = Wissenschaft
discover-category-security = Sicherheit
discover-category-utilities = Werkzeuge
