# The accessible name of the terminal, as Orca announces it.
console-terminal = Console output
console-leave-hint = Ctrl+Tab leaves the terminal.
console-idle-hint = The output of the next apply shows here.
console-printed-nothing = The changes printed nothing.
console-cancel = Cancel
console-hide = Hide the Console
console-resize = Resize the Console

console-output-menu = Console output actions
console-copy-output = Copy output
console-share-output = Share output…
console-clear-output = Clear output
console-output-copied = Copied the output to the clipboard.

console-status-running = Applying changes…
console-status-succeeded = All changes applied
# Steps that found nothing to do, such as a file already as wanted.
console-status-partly = { $count ->
        [one] Applied, 1 step had nothing to do
       *[other] Applied, { $count } steps had nothing to do
    }
console-status-nothing = Nothing needed to change
# A dry run: no step ran.
console-status-tested = Test mode: nothing was changed
console-status-failed = Stopped before all changes were applied

console-busy = Other changes are still being applied. Apply these once they are done.
console-cancel-waits = The cancel takes effect once the password dialog is answered.
# $reason is the helper's own words, shown as they are.
console-rejected = The helper refused the changes: { $reason }

step-waiting = Waiting
step-running = Running
step-succeeded = Done
# The step found nothing to do, such as a file already as wanted.
step-nothing-to-do = Nothing to do
# A dry run runs no step.
step-not-run = Not run (test mode)
step-cancelled = Cancelled
step-failed-exit = Failed (exit code { $code })
step-failed-signal = Failed (signal { $signal })
step-failed-unknown = Failed

# pkexec did not start the helper.
refusal-dismissed = The password dialog was closed. Nothing was changed.
refusal-denied = The password dialog was closed or the password was not accepted. Nothing was changed.
refusal-no-agent = No password dialog could be shown. Log in to a desktop session with a polkit agent.
refusal-helper-missing = The Garuda Toolbox helper is not installed. Reinstall garuda-toolbox.
refusal-broken = pkexec cannot run on this system. Its message is in the log.

# The plan failed outside any step. $detail is the system's words.
helper-missing-secret = A password is missing for step { $detail }.
helper-start = The helper could not be started: { $detail }
helper-pkexec = pkexec cannot run the helper: { $detail }
helper-no-pipes = The helper started without its input and output.
helper-lost = The helper could not be followed: { $detail }
helper-ended = The helper ended before the changes were done: { $detail }
helper-user-steps = The steps run in the app stopped unexpectedly: { $detail }

# Desktop notifications when the window is in the background.
notify-applied = Changes applied
notify-failed = Changes failed
notify-applied-body = Garuda Toolbox finished applying your changes.
notify-failed-body = A step failed. The Console shows what happened.

# An install failed as a mirror no longer has a package file.
console-mirror-missing = A mirror no longer has a file this install needs: the package databases are older than the mirror. A system update brings them up to date.
console-update-and-retry = Update the system and try again
