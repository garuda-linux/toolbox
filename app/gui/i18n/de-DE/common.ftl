# Words shared by every part of the app.

# The app's name in title bars and the About page.
app-name = Garuda Toolbox

# The language's own name, shown in the language picker.
language-name = Deutsch (Deutschland)

# The ✕ in a search field that empties it.
search-clear = Suche leeren

# The ✕ in a dialog's title bar.
dialog-close = Schließen

# Asked when the window is closed while changes are being applied.
close-running-question = Beenden, obwohl gerade Änderungen übernommen werden?
close-running-detail = Beenden bricht den Vorgang ab. Wird ein Schritt wie eine Paketaktualisierung mittendrin abgebrochen, kann das System danach beschädigt sein.
close-running-keep = Weiterlaufen lassen
close-running-quit = Abbrechen und beenden

# The banner of a page whose switches and menus wait while a plan runs.
common-plan-running = Es werden gerade Änderungen übernommen. Bis dahin lässt sich hier nichts ändern.

# The action of a banner that tells of a failed read.
common-read-again = Neu einlesen

# The actions a row's menu may hold on any page.
row-open-website = Website öffnen
row-copy-name = Namen kopieren
row-show-logs = Logs anzeigen
# The menu of a row without a ⋯ button, opened by a right click.
row-menu = Aktionen für { $name }
# The ⋯ button at a row's right with the actions behind its visible one.
row-more = Weitere Aktionen für { $name }
# The caption left of a sort dropdown.
common-sort-by = Sortieren nach

# Why the protected list keeps an installed package, shown on its row.
protected-boot = Wird für den Systemstart benötigt
protected-root = Wird für Root-Rechte benötigt
protected-login = Wird zum Anmelden benötigt
protected-network = Wird für die Netzwerkverbindung benötigt
protected-package-manager = Wird zum Installieren und Aktualisieren von Software benötigt
protected-desktop = Wird vom Desktop benötigt
# $name is the package, $why one of the reasons above.
protected-refused = { $name } bleibt installiert. { $why }.
# $name is a dependency the removal would take along, $why one of the
# reasons above.
protected-along = Diese Entfernung würde auch { $name } entfernen, das installiert bleibt. { $why }.
