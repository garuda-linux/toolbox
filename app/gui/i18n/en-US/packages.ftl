# The Packages page. Package names, descriptions, versions and file
# paths come from pacman's databases as they are.

packages-refresh = Read again
packages-search = Search packages
packages-search-hint = Search packages…
packages-loading = Reading the package lists…
packages-unavailable = The packages are not available on this system: the package databases of pacman could not be read.
packages-failed = The package databases could not be read.

packages-filter-installed = Installed
packages-filter-updates = Updates
packages-filter-orphans = Orphans
packages-filter-all = All

packages-none = No packages to show
packages-none-detail = The repositories offer none and none is installed.
packages-no-updates = No pending updates
packages-no-updates-detail = Home checks for updates. Installed packages with a newer version show here.
packages-no-orphans = No orphans
packages-no-orphans-detail = Every package installed as a dependency is still needed by another one.
packages-nothing-matches = No package matches “{ $search }”
packages-nothing-matches-detail = The search looks at the names and descriptions under the chip that is picked.
packages-clear-search = Clear search

# Why a click queued nothing.
packages-refused-not-offered = { $package } cannot be installed: no configured repository offers it.
packages-refused-unread = { $package } cannot be queued yet: the installed packages are still being read.
packages-refused-running-kernel = { $package } is the kernel that runs right now, so it stays installed.
packages-refused-running-unknown = { $package } stays: the kernel that runs right now is not known, so no kernel can be removed. Restart the computer and try again.
packages-refused-last-kernel = { $package } is the last installed kernel. Install another one before you remove it.
packages-dismiss = Dismiss
# A row button's name for screen readers: $action is the button, such as Install, $name the package.
packages-action-name = { $action } { $name }

packages-install = Install
packages-remove = Remove
packages-take-back = Take back

packages-tag-dependency = Dependency
packages-tag-foreign = Not from a repository
packages-installed = Installed
packages-installs = Installs with Apply
packages-removes = Removes with Apply
# In place of a repository's name.
packages-no-repository = no repository

# The question before a removal.
packages-remove-title = Remove { $package }?
packages-remove-alone = It is removed alone: it pulled in no dependency that nothing else needs.
packages-remove-along = { $count ->
        [one] It takes along 1 dependency that nothing else needs:
       *[other] It takes along { $count } dependencies that nothing else needs:
    }
packages-remove-note = The removal waits in the queue. When it runs, the Console lists what pacman really removes.
packages-cancel = Cancel

# The -name texts are what screen readers say.
packages-back = ← Packages
packages-back-name = Back to Packages
# $package is the package the page was opened from.
packages-back-to = ← { $package }
packages-back-to-name = Back to { $package }
# Shows the rest of a list of names.
packages-more = +{ $count } more
packages-more-name = Show { $count } more
packages-details-loading = Reading the package…
packages-details-failed = The package could not be read.
packages-gone = { $package } is no longer in any package database
packages-gone-detail = It was removed, or a repository dropped it.
packages-copy = Copy { $label }
packages-copied = Copied to the clipboard.
packages-value-none = None

packages-info-repository = Repository
packages-info-installed-version = Installed version
packages-info-url = Website
packages-info-licenses = Licenses
packages-info-groups = Groups
packages-info-size = Installed size
packages-info-date = Install date
packages-info-reason = Install reason
packages-info-provides = Provides
packages-info-conflicts = Conflicts with
packages-info-replaces = Replaces
packages-reason-explicit = Installed explicitly
packages-reason-dependency = Installed as a dependency

packages-depends = Depends on
packages-optional = Optional dependencies
packages-required-by = Required by
# The count of installed files follows it, dimmed.
packages-files = Files
packages-copy-list = Copy list
packages-files-copied = The file list was copied to the clipboard.
# What screen readers call the list of files.
packages-files-list = Installed files

packages-history = History
packages-history-all = Show all
packages-history-failed = pacman's log could not be read.
packages-history-none = No changes found
packages-history-installed = Installed
packages-history-upgraded = Upgraded
packages-history-downgraded = Downgraded
packages-history-reinstalled = Reinstalled
packages-history-removed = Removed

# The AUR chip, shown while Settings turns the AUR on. Names,
# descriptions and versions come from the AUR's metadata dump.
packages-filter-aur = AUR
packages-aur-off = AUR packages are hidden.
packages-aur-off-settings = Turn them on in Settings
packages-aur-loading = Reading the AUR's package list…
packages-aur-failed = The AUR's package list could not be downloaded. The next try is a day after this one.
packages-aur-waiting = The AUR's package list is not here yet. The last download failed, the next try is a day after it.
# $date is the day the list was downloaded, such as 2025-01-10.
packages-aur-note = List of { $date }. AUR packages are community scripts that are built on your computer.
packages-tag-aur = AUR
packages-aur-out-of-date = Out of date
packages-aur-builds = Builds with Apply
packages-aur-votes = { $count ->
        [one] 1 vote
       *[other] { $count } votes
    }

# The question before an AUR build.
packages-aur-install-title = Build { $package } from the AUR?
packages-aur-install-body = Built from a community script on your computer. paru shows the build script in the Console before it builds.

packages-aur-maintainer = Maintainer
packages-aur-orphaned = Nobody, orphaned
packages-aur-votes-label = Votes
packages-aur-popularity = Popularity
packages-aur-submitted = First submitted
packages-aur-updated = Last updated
packages-aur-out-of-date-since = Out of date since
packages-aur-page = AUR page
packages-aur-make-depends = Make dependencies
