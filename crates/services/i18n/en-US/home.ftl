home-hello = Hello, { $user }!
home-welcome = Welcome to Garuda Linux
home-subtitle = Maintain and set up your system.

home-status = System Status
home-refresh = Read Again
home-checking = Checking the system…
home-all-clear =
    { $count ->
        [one] The one check passed
       *[other] All { $count } checks passed
    }
home-status-failed = The update check failed.
home-failed =
    { $count ->
        [one] One check could not run, so its result is missing.
       *[other] { $count } checks could not run, so their results are missing.
    }
home-show-details = Show Details
home-hide-details = Hide Details
# $reason is the program's own words, in English.
home-failed-updates = Repository updates: { $reason }
home-failed-aur = AUR updates: { $reason }
home-failed-merges = Files to merge: { $reason }
home-failed-last-update = Last update: { $reason }
home-failed-disk = Disk space: { $reason }
home-failed-services = Services: { $reason }
home-failed-cleanup = Orphans and package cache: { $reason }
home-failed-health = garuda-health: { $reason }

home-updates =
    { $count ->
        [one] One update available
       *[other] { $count } updates available
    }
home-updates-detail = Update now in the Console, or queue the update with your other changes.
home-update-now = Update Now
home-queue-update = Queue Update
home-unqueue-update = Don't Update
home-show-packages = Show Packages
home-aur-updates =
    { $count ->
        [one] One AUR update available
       *[other] { $count } AUR updates available
    }
home-aur-detail = Packages of the AUR (Arch User Repository) are built by paru in the Console. Review the build files and answer its questions there.
home-aur-update = Update in Console
home-stale = The last update is overdue
home-stale-detail = Arch based systems should be updated at least every two weeks.
home-merges = Configuration Files to Merge
home-merges-review = Review…
home-merges-meld = Open in meld
home-merges-title = Configuration Files to Merge
home-merges-reading = Reading the files…
home-merges-failed = The files could not be read.
home-merges-none = Nothing left to merge.
home-merges-files = Files
home-merges-pair = { $copy } beside { $file }
home-merges-queued = queued
home-merges-take-back = Take Back
home-merges-unreadable = A file could not be read. Use meld instead.
home-merges-gone = The file in use is gone. The copy can take its place.
home-merges-same = The copy is the same as the file in use.
home-merges-older = An older saved version. It can only be removed.
home-merges-remove = Remove the Copy
home-merges-keep = Keep Current
home-merges-take-new = Take New
home-merges-take-saved = Take Saved
home-merges-change = Change { $number }
home-merges-unchanged = { $count ->
        [one] One unchanged line
       *[other] { $count } unchanged lines
    }
home-merges-keep-file = Keep Current File
home-merges-use-file = Use New File
home-merges-restore-file = Restore Saved File
home-merges-queue-picks = Queue the Merge
home-merges-edit = Edit Result…
home-merges-edit-name = Merged text
home-merges-edit-invalid = The text is too large or holds a NUL character.
home-merges-edit-back = Back
home-merges-edit-queue = Queue the Edited Text
home-merges-applied = This runs with your other changes. Where snapper is set up, a snapshot comes first. The old file is kept in /var/lib/garuda-toolbox/backups.
home-health-errors = garuda-health found problems
home-health-warnings = garuda-health found warnings
home-health-fix = Fix
home-health-check = Show in Console
home-restart = Restart to finish the update
home-restart-detail = The running kernel or system libraries were replaced.
home-restart-button = Restart…

home-restart-question = Restart Now?
home-restart-question-detail = Save your work first. Open programs are closed.
home-restart-confirm = Restart
home-cancel = Cancel
home-restart-denied = The restart was not authorized.
home-restart-failed = The computer could not be restarted. Restart it from the desktop's menu.

home-pending = Pending Updates
home-pending-none = No updates are waiting.
home-pending-aur = AUR
home-pending-versions = { $from } → { $to }

home-live = Live System
home-install = Install Garuda Linux
home-install-detail = Start the installer. It asks for your password.
home-install-button = Start Installer
# $reason is the system's own words, in English.
home-install-failed = The installer could not start: { $reason }
home-programs-failed = The live system's tools could not be looked for.
home-chroot = Repair an Installed System
home-chroot-detail = Open a root shell inside the Garuda Linux installed on this computer, in the Console. Type exit to leave it.
home-chroot-button = Open in Console

home-next = Next Steps
home-next-setup = Pick the apps you want and install them in one go
home-next-maintenance = Keep your system in a good state
home-next-review =
    { $count ->
        [one] One change waits to be applied
       *[other] { $count } changes wait to be applied
    }


# $days counts whole days since the last update.
home-summary-fine = Everything is fine
home-summary-attention =
    { $count ->
        [one] One thing needs attention
       *[other] { $count } things need attention
    }
home-summary-updated-today = · updated today
home-summary-updated =
    { $days ->
        [one] · last update yesterday
       *[other] · last update { $days } days ago
    }

# The stat tiles under the greeting: a name, then its figure.
home-stat-updates = Updates
home-stat-updates-none = Up to date
home-stat-updates-count =
    { $count ->
        [one] One available
       *[other] { $count } available
    }
home-stat-disk = Disk
# $free is a size such as "38.2 GiB".
home-stat-disk-free = { $free } free
home-stat-health = Health
home-stat-health-fine = No problems
home-stat-health-warnings = Warnings
home-stat-health-errors = Problems found
home-stat-last-update = Last update
home-stat-last-update-today = Today
home-stat-last-update-days =
    { $days ->
        [one] Yesterday
       *[other] { $days } days ago
    }
# A figure not read yet, or whose check could not run.
home-stat-unknown = Not known

home-disk = Disk space is low
home-disk-detail = Remove orphans, clean the package cache or delete old snapshots on Maintenance.
home-disk-button = Open Maintenance
home-services =
    { $count ->
        [one] A service failed
       *[other] { $count } services failed
    }
home-services-button = Show in Services
home-cleanup = Orphans and Package Cache
# $cache is a size such as "6.4 GiB".
home-cleanup-detail-cache = No package is orphaned. The package cache holds { $cache }.
home-cleanup-detail =
    { $orphans ->
        [one] One package is no longer needed. The package cache holds { $cache }.
       *[other] { $orphans } packages are no longer needed. The package cache holds { $cache }.
    }
home-cleanup-queue = Queue Clean-up
home-cleanup-unqueue = Don't Clean Up

home-check-updates = Repository updates
home-check-aur = AUR updates
home-check-merges = Files to merge
home-check-disk = Disk space
home-check-services = Services
home-check-cleanup = Orphans and cache
home-check-health = garuda-health

home-next-diagnostics = Get help with a report for the forum
home-recent = Recently Used

home-news = News
home-news-garuda-page = Garuda forum
home-news-arch-page = Arch news
home-news-garuda = Garuda
home-news-arch = Arch
# A post that asks for a manual step around an update.
home-news-manual = Manual step
# $title is the post's title, as its site gives it.
home-news-open = Open { $title }
# Over Home when saved changes did not come back at start. $count is how many.
home-queue-lost =
    { $count ->
        [one] One queued change could not be restored. The log file says why.
       *[other] { $count } queued changes could not be restored. The log file says why.
    }
home-queue-lost-dismiss = Dismiss
# A run from Home did not start, as other changes are being applied.
home-busy = Other changes are still being applied, so this did not start. Start it again once they are done.
