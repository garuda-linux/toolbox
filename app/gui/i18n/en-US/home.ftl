home-hello = Hello, { $user }!
home-welcome = Welcome to Garuda Linux
home-subtitle = Maintain and set up your system.

home-status = System status
home-refresh = Read again
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
home-show-details = Show details
home-hide-details = Hide details
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
home-update-now = Update now
home-queue-update = Queue update
home-unqueue-update = Don't update
home-show-packages = Show packages
home-aur-updates =
    { $count ->
        [one] One AUR update available
       *[other] { $count } AUR updates available
    }
home-aur-detail = Packages of the AUR (Arch User Repository) are built by paru in the Console. Review the build files and answer its questions there.
home-aur-update = Update in Console
home-stale = The last update is overdue
home-stale-detail = Arch based systems should be updated at least every two weeks.
home-merges = Configuration files to merge
home-health-errors = garuda-health found problems
home-health-warnings = garuda-health found warnings
home-health-fix = Fix
home-health-check = Show in Console
home-restart = Restart to finish the update
home-restart-detail = The running kernel or system libraries were replaced.
home-restart-button = Restart…

home-restart-question = Restart now?
home-restart-question-detail = Save your work first. Open programs are closed.
home-restart-confirm = Restart
home-cancel = Cancel
home-restart-denied = The restart was not authorized.
home-restart-failed = The computer could not be restarted. Restart it from the desktop's menu.

home-pending = Pending updates
home-pending-none = No updates are waiting.
home-pending-aur = AUR
home-pending-versions = { $from } → { $to }

home-live = Live system
home-install = Install Garuda Linux
home-install-detail = Start the installer. It asks for your password.
home-install-button = Start installer
# $reason is the system's own words, in English.
home-install-failed = The installer could not start: { $reason }
home-programs-failed = The live system's tools could not be looked for.
home-chroot = Repair an installed system
home-chroot-detail = Open a root shell inside the Garuda Linux installed on this computer, in the Console. Type exit to leave it.
home-chroot-button = Open in Console

home-next = Next steps
home-next-setup = Pick the apps you want and install them in one go
home-next-maintenance = Keep your system in a good state
home-next-review =
    { $count ->
        [one] One change waits to be applied
       *[other] { $count } changes wait to be applied
    }


# $days counts whole days since the last update.
home-summary-fine = Everything is fine
home-summary-unknown = Some checks could not run
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

# $kernel is as uname prints it, $free a size such as "38.2 GiB".
home-fact-packages =
    { $count ->
        [one] One package
       *[other] { $count } packages
    }
home-fact-packages-foreign = { $count } packages · { $foreign } AUR
home-fact-kernel-days =
    { $amount ->
        [one] { $kernel } · up one day
       *[other] { $kernel } · up { $amount } days
    }
home-fact-kernel-new = { $kernel } · up less than an hour
home-fact-kernel-hours =
    { $amount ->
        [one] { $kernel } · up one hour
       *[other] { $kernel } · up { $amount } hours
    }
home-fact-disk = { $free } free on /

home-disk = Disk space is low
home-disk-detail = Remove orphans, clean the package cache or delete old snapshots on Maintenance.
home-disk-button = Open Maintenance
home-services =
    { $count ->
        [one] A service failed
       *[other] { $count } services failed
    }
home-services-button = Show in Services
home-cleanup = Orphans and package cache
# $cache is a size such as "6.4 GiB".
home-cleanup-detail-cache = No package is orphaned. The package cache holds { $cache }.
home-cleanup-detail =
    { $orphans ->
        [one] One package is no longer needed. The package cache holds { $cache }.
       *[other] { $orphans } packages are no longer needed. The package cache holds { $cache }.
    }
home-cleanup-queue = Queue clean-up
home-cleanup-unqueue = Don't clean up

home-check-updates = Repository updates
home-check-aur = AUR updates
home-check-merges = Files to merge
home-check-disk = Disk space
home-check-services = Services
home-check-cleanup = Orphans and cache
home-check-health = garuda-health

home-next-diagnostics = Get help with a report for the forum
home-recent = Recently used

home-news = News
home-news-garuda-page = Garuda forum
home-news-arch-page = Arch news
home-news-garuda = Garuda
home-news-arch = Arch
# A post that asks for a manual step around an update.
home-news-manual = Manual step
# $title is the post's title, as its site gives it.
home-news-open = Open { $title }
