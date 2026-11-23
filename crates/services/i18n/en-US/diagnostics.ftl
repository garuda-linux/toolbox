# The Diagnostics page: reports about the system for a forum post, and
# their encrypted upload. The reports themselves stay in English.

page-diagnostics-subtitle = Reports about this computer for a forum post or a bug report.

diagnostics-reports = Reports
# Unfolds the report rows while a report shows.
diagnostics-another = Collect Another Report
diagnostics-collect = Collect
# What screen readers call a row's Collect. $name is the report's name.
diagnostics-collect-name = Collect { $name }
diagnostics-cancel = Cancel

diagnostics-inxi = System information
diagnostics-inxi-description = Hardware, drivers and system overview from garuda-inxi.
diagnostics-garuda-health = System health
diagnostics-garuda-health-description = What garuda-health finds wrong with the system.
diagnostics-systemd-analyze = Boot time
diagnostics-systemd-analyze-description = How long each service took to start and what the boot waited for, from systemd-analyze.
diagnostics-journalctl = Journal
diagnostics-journalctl-description = The last 1000 lines of the system journal. Asks for your password when your account may not read it.
diagnostics-pacman = pacman log
diagnostics-pacman-description = What pacman logged since it last ran.
diagnostics-dmesg = Kernel log
diagnostics-dmesg-description = The kernel's messages from dmesg. Asks for your password when the system keeps them from users.
diagnostics-all = Full report
diagnostics-all-description = All of the above in one report. May ask for your password twice.

# Where the collection stands.
diagnostics-idle-detail = The report stays on this computer unless you share it.
diagnostics-collecting = Collecting { $name }…
# $step counts from 1.
diagnostics-collecting-step = Collecting { $name } ({ $step } of { $count })…
diagnostics-collecting-detail = Starting another collector cancels this one.
diagnostics-empty = { $name } printed nothing.
diagnostics-unavailable = { $name } is not available on this system.
diagnostics-unavailable-detail = The program it needs is not installed.
diagnostics-dismissed = { $name } was not collected: the password dialog was closed.
diagnostics-denied = { $name } was not collected: your account may not read this.
diagnostics-no-agent = { $name } was not collected: no password dialog could appear, because this desktop session runs no polkit authentication agent.
diagnostics-helper-missing = { $name } was not collected: the helper of Garuda Toolbox is not installed. Reinstall Garuda Toolbox.
# $reason is the program's own words.
diagnostics-failed = { $name } failed: { $reason }
diagnostics-report-label = Diagnostics report

# The report's header line. $time is when it came in, as 14:05.
diagnostics-lines =
    { $count ->
        [one] one line
       *[other] { $count } lines
    }
diagnostics-collected = · collected { $time } UTC · { $lines }
diagnostics-collected-unknown = · { $lines }
diagnostics-search = Search the report
diagnostics-search-clear = Clear the Search

diagnostics-copy = Copy Report
diagnostics-share = Share Report…
diagnostics-share-tooltip = Uploads the report, encrypted, to bin.garudalinux.org and gives you a link to share.
diagnostics-report-copied = Copied the report to the clipboard.
