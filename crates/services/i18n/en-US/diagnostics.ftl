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
# The reasons of a failed collection. The words of a program stay as it printed them.
diagnostics-failed-no-program = No program ran.
diagnostics-failed-exit-code = Exit code: { $code }
diagnostics-failed-signal = Ended by a signal.
diagnostics-failed-start = { $program } could not be started: { $error }
diagnostics-failed-timed-out = { $program } took longer than { $seconds } seconds
diagnostics-failed-too-large = { $program } printed too much
diagnostics-failed-read = { $path }: { $error }
diagnostics-failed-helper-missing = the helper of Garuda Toolbox or pkexec is not installed
diagnostics-failed-helper-start = cannot start the helper: { $error }
diagnostics-failed-helper-timed-out = the helper gave no answer in time
diagnostics-failed-helper-too-long = the helper's answer is too long
diagnostics-failed-helper-pkexec = pkexec cannot run: { $words }
diagnostics-failed-helper-ended = the helper ended with { $ending }: { $words }
diagnostics-failed-ending-code = exit code { $code }
diagnostics-failed-ending-signal = a signal
diagnostics-failed-helper-unreadable = the helper's answer is unreadable
diagnostics-failed-helper-stopped = the read stopped before the helper answered
diagnostics-failed-answer = The answer of the helper of Garuda Toolbox was not understood: { $fault }
diagnostics-failed-answer-too-long = line of { $length } bytes exceeds the limit of { $limit }
diagnostics-failed-not-installed = The next program is not installed.
diagnostics-failed-dismissed = Not collected: the password dialog was dismissed.
diagnostics-failed-denied = Not collected: root rights were not given.
diagnostics-failed-no-agent = Not collected: no password dialog could appear.
diagnostics-failed-helper-not-installed = Not collected: the helper is not installed.
diagnostics-stopped = { $name } stopped before it answered.
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
