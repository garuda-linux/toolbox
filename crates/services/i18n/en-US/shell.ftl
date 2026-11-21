
sidebar-collapse = Collapse sidebar
sidebar-expand = Expand sidebar
sidebar-search = Search

console-title = Console
console-idle = No changes are being applied

# The command palette (Ctrl+K).
palette-search = Search pages, commands, apps and packages
# Labels over the palette's groups of rows. The Maintenance group takes
# the page's name.
palette-group-pages = Pages
palette-group-commands = Commands
palette-group-apps = Apps
palette-group-packages = Packages
palette-group-aur = AUR
# The last row of a group that shows only its best rows. $count is how many rows the group has, $page the name of the page that lists them all.
palette-show-all = Show all { $count } in { $page }
# The last row of a group that shows only its best rows, which shows the rest right there. $count is how many rows the group has.
palette-show-all-here = Show all { $count }
command-apply = Apply changes
command-toggle-sidebar = Toggle sidebar
command-toggle-console = Toggle console
command-open-palette = Command palette
command-review-changes = Review changes
command-quit = Quit
command-focus-search = Search this page
command-help = Help for this page
command-clear-changes = Remove all changes
command-zoom-in = Zoom in
command-zoom-out = Zoom out
command-zoom-reset = Reset zoom
command-apply-tooltip = Run every queued change now, with one password
command-toggle-sidebar-tooltip = Show or hide the page names in the sidebar
command-toggle-console-tooltip = Show or hide the console with the output of the last run
command-open-palette-tooltip = Find a page, a command or an app by its name
command-review-changes-tooltip = List the queued changes before you apply them
command-clear-changes-tooltip = Remove every queued change. This cannot be undone
command-quit-tooltip = Close Garuda Toolbox. Queued changes are kept
command-focus-search-tooltip = Put the cursor into the search of the open page
command-help-tooltip = Open the user guide on the page about the open one
command-zoom-in-tooltip = Show everything in the window larger, up to 150 %
command-zoom-out-tooltip = Show everything in the window smaller, down to 80 %
command-zoom-reset-tooltip = Show everything in the window at its normal size again

# Notes at the top right of the page about background results.
toast-name = Notification
toast-plan-succeeded = Your changes were applied.
toast-plan-failed = A step failed. Open the Console to see what happened.
toast-link-failed = The link did not open.
settings-not-saved = A setting could not be saved.
toast-queue-not-saved = The queued changes could not be saved. They are lost when the app closes.
# A garuda-toolbox queue command changed the window's queued changes.
toast-queue-edited-outside = The queue was changed from the command line.
# A program the app started, such as the installer or a Maintenance tool,
# ended with an error. $name is its program name.
toast-tool-failed = { $name } ended with an error. The log file says why.
toast-tool-not-started = { $name } could not be started: { $reason }
# After Copy name in a row menu.
toast-copied = Copied “{ $name }” to the clipboard.

# The names of the window's parts for screen readers.
pane-sidebar = Pages
pane-page = Page
pane-console = Console
pane-changes = Changes to apply
