# Words shared by every part of the app.

# The app's name in title bars and the About page.
app-name = Garuda Toolbox

# The language's own name, shown in the language picker.
language-name = English (United States)

# The ✕ in a search field and the button that empties a list's search.
search-clear = Clear search

# The title of a list that the search emptied, with the search as typed.
search-nothing-matches = Nothing matches “{ $search }”

# The ✕ in a dialog's title bar.
dialog-close = Close

# The eye in a password field while its text shows as dots.
password-show = Show password
# The eye in a password field while its text shows.
password-hide = Hide password

# Asked when the window is closed while changes are being applied.
close-running-question = Quit while changes are being applied?
close-running-detail = Quitting cancels the run. A step that is halfway done, such as a package update, may leave the system in a broken state.
close-running-keep = Keep running
close-running-quit = Cancel and quit

# Asked when the window is closed while Config Files shows an edit that was not applied.
close-edit-question = Quit without applying your edit?
close-edit-detail = The changes you made to a configuration file are not written yet. Quitting discards them.
close-edit-keep = Keep editing
close-edit-quit = Discard and quit

# The banner of a page whose switches and menus wait while a plan runs.
common-plan-running = Changes are being applied. This waits until they are done.

# The action of a banner that tells of a failed read.
common-read-again = Read again

# The actions a row's menu may hold on any page.
row-open-website = Open website
row-copy-name = Copy name
row-show-logs = Show logs
# The menu of a row without a ⋯ button, opened by a right click.
row-menu = Actions for { $name }
# The ⋯ button at a row's right with the actions behind its visible one.
row-more = More actions for { $name }
# The caption left of a sort dropdown.
common-sort-by = Sort by

# Why the protected list keeps an installed package, shown on its row.
protected-boot = Needed to start the system
protected-root = Needed for administrator rights
protected-login = Needed to log in
protected-network = Needed for the network connection
protected-package-manager = Needed to install and update software
protected-desktop = Needed by the desktop
# $name is the package, $why one of the reasons above.
protected-refused = { $name } stays installed. { $why }.
# $name is a dependency the removal would take along, $why one of the
# reasons above.
protected-along = This removal would also take { $name }, which stays installed. { $why }.
# The button that closes the missing-catalog window.
missing-catalog-quit = Quit
# The missing-catalog window: a title, what to do, then the missing folder and the reinstall command, each with a copy button. $label is the row's name.
missing-catalog-title = Garuda Toolbox is not installed completely
missing-catalog-body = A folder it needs is missing. Reinstall the garuda-toolbox package, then start the app again.
missing-catalog-folder = Missing folder
missing-catalog-reinstall = Reinstall with
missing-catalog-copy = Copy { $label }
missing-catalog-copied = Copied to the clipboard.
# Shown after Quit was confirmed during a run, until the canceled run ends and the window closes.
close-stopping-title = Stopping the run
close-stopping-detail = The window closes once the run has stopped.
