
page-maintenance-subtitle = Keep the system in a good state: updates, cleanups, tools and repairs.
maintenance-refresh = Read again

maintenance-tabs = Maintenance tabs
maintenance-tab-common = Common
maintenance-tab-reset = Reset settings
maintenance-tab-repairs = Repairs

maintenance-operations = Package care
maintenance-operations-detail = Queue these to apply them with your other changes, or run one now in the Console.
maintenance-run-now = Run now
maintenance-unqueue = Remove from queue
maintenance-op-upgrade-system = Update system
maintenance-op-upgrade-system-detail = Update every package, with snapshots and checks when garuda-update is installed.
maintenance-op-upgrade-system-queue = Queue update
maintenance-op-remove-orphans = Remove orphans
maintenance-op-remove-orphans-detail = Remove packages that were installed as dependencies and are no longer needed.
maintenance-op-remove-orphans-queue = Queue removal
maintenance-op-remove-orphans-question = Packages that were installed for others and are no longer needed get removed. The Console lists them and asks once more before anything goes.
maintenance-op-clean-package-cache = Clean package cache
maintenance-op-clean-package-cache-detail = Delete cached packages of software that is no longer installed.
maintenance-op-clean-package-cache-queue = Queue cleanup
maintenance-op-clean-package-cache-question = Every cached package of software that is no longer installed is deleted. You cannot go back to those versions afterwards.
maintenance-op-refresh-keyring = Refresh keyring
maintenance-op-refresh-keyring-detail = Download the newest keys that prove the packages of the repositories are genuine (the keyring).
maintenance-op-refresh-keyring-queue = Queue refresh
maintenance-op-remove-database-lock = Remove the pacman lock
maintenance-op-remove-database-lock-detail = Delete a lock left behind by an interrupted update. Nothing happens while pacman runs.
maintenance-op-aur-upgrade-detail = Update the packages built from the AUR with paru, which asks its questions in the terminal.
maintenance-op-remove-database-lock-queue = Queue unlock

maintenance-tools = Tools
maintenance-tools-detail = Each tool opens in its own window.
maintenance-open = Open
maintenance-install-open = Install and open…
maintenance-tool-missing = Not installed. Installing it starts at once in the Console and asks for your password, then the tool opens.
maintenance-ranker-missing = Needs rate-mirrors, which is not installed. Installing it starts at once in the Console and asks for your password, then the ranking starts.
maintenance-ranking = Testing the mirrors. This takes up to five minutes. The Console opens when the new list is ready.
maintenance-tool-arch-mirrors = Refresh Arch mirrors
maintenance-tool-arch-mirrors-detail = Choose the fastest Arch Linux mirrors, the servers packages download from, with reflector-simple.
maintenance-tool-chaotic-mirrors = Refresh Chaotic-AUR mirrors
maintenance-tool-chaotic-mirrors-detail = Rank the mirrors of Chaotic-AUR (prebuilt packages from the Arch User Repository) by speed and save the fastest.
maintenance-tool-btrfs-assistant = Btrfs Assistant
maintenance-tool-btrfs-assistant-detail = Manage snapshots and run Btrfs maintenance.
maintenance-tool-edit-repositories = Edit repositories
maintenance-tool-edit-repositories-detail = Turn package repositories on or off with pace, the editor for pacman.conf.
maintenance-tool-merge-pacdiff = Merge configuration updates
maintenance-tool-merge-pacdiff-detail = Compare and merge .pacnew and .pacsave files side by side. An update leaves them beside a config file you changed, with its own new version.

# Why a tool did not start or the mirrors were not ranked. $tool is the
# tool's name, $reason the system's own words.
maintenance-problem-launch-missing = { $tool } could not be started because it is not installed.
maintenance-problem-launch-denied = { $tool } could not be started because this account may not run it.
maintenance-problem-launch = { $tool } could not be started: { $reason }
maintenance-problem-mirrors-timeout = Ranking the Chaotic-AUR mirrors took longer than five minutes and was stopped. The list of mirrors is unchanged.
maintenance-problem-mirrors-start = The ranking program rate-mirrors could not be started, so the Chaotic-AUR mirrors were not ranked: { $reason }
maintenance-problem-mirrors-exit = Ranking the Chaotic-AUR mirrors failed, so their list is unchanged. Check your internet connection and try again.
maintenance-problem-mirrors-invalid = Ranking gave no usable list of mirrors. The Chaotic-AUR list is unchanged.
maintenance-problem-dismiss = Dismiss

maintenance-reset = Application settings
maintenance-reset-detail = Choose the applications whose settings go back to Garuda's defaults. Your current files are kept as backups.
maintenance-reset-empty = No default settings found
maintenance-reset-empty-detail = This system has no Garuda defaults in /etc/skel.
maintenance-reset-button = Reset selected…
maintenance-reset-busy = Resetting the chosen settings…
maintenance-reset-done = Settings reset. The old files end in .bak and the date.
# $names lists the applications, separated by commas.
maintenance-reset-failed = Some settings could not be reset: { $names }
maintenance-reset-question = { $count ->
    [one] Reset 1 setting?
   *[other] Reset { $count } settings?
}
# $files is the comma list of the home files a reset replaces.
maintenance-reset-replaces = Replaces { $files }
maintenance-reset-files = These files are replaced by Garuda's defaults. The current ones are kept as backups:
maintenance-reset-cancel = Cancel
maintenance-reset-confirm = Reset

maintenance-repairs = Repairs
maintenance-repairs-detail = Each repair runs in the Console, where it can ask its questions.
maintenance-repairs-missing = The greyed out repairs below need garuda-update, which is not installed. Installing it unlocks them. It starts at once in the Console and asks for your password.
maintenance-install-garuda-update = Install garuda-update
maintenance-run = Run
maintenance-repair-health-check = System health check
maintenance-repair-health-check-detail = Look for common problems with garuda-health.
maintenance-repair-health-fix = Repair system health
maintenance-repair-health-fix-detail = Let garuda-health fix the common problems it finds.
maintenance-repair-remote-fix = Reset package manager settings
maintenance-repair-remote-fix-detail = Reset pacman.conf, the mirror lists and the keyring to Garuda's defaults.
maintenance-repair-remote-keyring = Reset keyring
maintenance-repair-remote-keyring-detail = Reset the package signing keys to Garuda's defaults.
maintenance-repair-remote-full-fix = Full reset
maintenance-repair-remote-full-fix-detail = Reset pacman fully and reinstall every package.
maintenance-repair-remote-reset-audio = Reset audio
maintenance-repair-remote-reset-audio-detail = Reinstall and reset the PipeWire audio configuration.
maintenance-repair-remote-reset-snapper = Reset Snapper
maintenance-repair-remote-reset-snapper-detail = Reset Snapper fully and create its configurations again.
maintenance-repair-remote-reinstall = Reinstall packages
maintenance-repair-remote-reinstall-detail = Reinstall every installed package.

# The question before a repair that reinstalls packages or throws away
# what the user had. $repair is the repair's name, such as Full reset.
maintenance-repair-question = Run { $repair }?
maintenance-repair-cancel = Cancel
maintenance-repair-remote-fix-question = Your pacman.conf and mirror lists are replaced by Garuda's defaults. Repositories you added are gone afterwards.
maintenance-repair-remote-full-fix-question = Your pacman.conf is replaced by Garuda's default and every installed package is downloaded and installed again. This takes long and needs a working connection.
maintenance-repair-remote-reset-audio-question = PipeWire and WirePlumber are installed again and your audio settings go back to their defaults.
maintenance-repair-remote-reset-snapper-question = Every Snapper snapshot and backup subvolume is deleted and Snapper's configurations are made again. You cannot go back to an older snapshot afterwards.
maintenance-repair-remote-reinstall-question = Every installed package is downloaded and installed again. This takes long and needs a working connection.
maintenance-repair-needs = Needs { $package }
