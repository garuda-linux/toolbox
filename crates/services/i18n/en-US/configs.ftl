
page-configs-subtitle = Well-known configuration files of the system and your apps.
configs-refresh = Read again
configs-search = Search config files
configs-search-hint = Search config files…
configs-search-detail = The search looks at names, paths, descriptions and the files of folders.
configs-clear-search = Clear search
configs-reading = Looking for config files…
configs-unreadable = The list of config files could not be read
configs-none = None of the listed config files exists on this system
configs-no-match = No config file matches
configs-dismiss = Dismiss
configs-list = { $section } files

configs-open-in-app = Open in app
configs-edit = Edit
configs-view = View
configs-show-files = Show the files
configs-hide-files = Hide the files
# $count is how many files the folder holds.
configs-files = { $count ->
        [one] 1 file
       *[other] { $count } files
    }
configs-wiki-open = Open wiki page
configs-admin = Saving asks for an administrator password
configs-read-only = Only shown here: it decides who may administer the system or what runs as root
# $target is where the link leads.
configs-link = Link to { $target }
configs-not-edited-dangling = Not edited here: the link leads to no file
configs-not-edited-not-regular = Not edited here: the link leads to a folder or another thing that is no file
configs-not-edited-outside = Not edited here: the link leads out of the system's config folders, to a file of a package or a generated one

# $path names the file.
configs-problem-dismissed = { $path } was not read: the password dialog was closed.
configs-problem-denied = { $path } was not read: the password dialog was closed or the password was not accepted.
configs-problem-no-agent = { $path } was not read, because no password dialog could appear: this desktop session runs no polkit authentication agent.
configs-problem-helper-missing = { $path } was not read, because the helper of Garuda Toolbox is not installed. Reinstall Garuda Toolbox.
configs-problem-missing = { $path } does not exist anymore.
configs-problem-link = { $path } or a folder on its way was replaced by a link while it was read, so it was left alone. Look again and retry.
configs-problem-dangling = { $path } is a link that leads to no file, so there is nothing to edit.
configs-problem-outside = { $path } is a link out of the system's config folders, to a file of a package or a generated one. Such files are not edited here.
configs-problem-not-regular = { $path } is not a regular file.
configs-problem-too-large = { $path } is larger than 1 MiB, which is too large to edit here.
configs-problem-not-text = { $path } is no text file, so it cannot be edited here.
configs-problem-no-editor = No terminal editor is installed. Install micro, nano or vi and try again.
configs-problem-busy = Another task runs in the Console. Try again when it has finished.
configs-problem-editor-failed = The editor ended with an error and { $path } was not changed.
configs-problem-write-failed = { $path } was not written. The Console shows why. Your edit is kept.
configs-problem-copy-failed = The copy for editing could not be used.
configs-problem-unknown = This file cannot be opened here.
configs-problem-read-only = This file decides who may administer the system or what runs as root, so it is only shown here. Change it in a terminal with sudo.
configs-problem-other = { $path } could not be read.
configs-view-too-large = { $path } is larger than 1 MiB, which is too large to show here.
configs-view-not-text = { $path } is no text file, so it cannot be shown here.
configs-view-busy = Another file is still being read. Try again when it is shown.

# $editor names the editor, $backup where the previous version is kept.
configs-status-reading = Reading { $path }…
configs-status-editing = { $path } is open in the Console with { $editor }. Save and quit the editor to go on.
configs-status-comparing = Comparing your copy with { $path }…
configs-status-review = Your changes to { $path } wait for your review.
configs-status-writing = Writing { $path }…
configs-status-saved = Saved { $path }. The previous version is kept as { $backup }.
configs-status-unchanged = { $path } was not changed, so there is nothing to save.
configs-show-console = Show Console
configs-cancel-edit = Cancel edit
configs-review = Review changes

configs-review-title = Save the changes to { $path }?
# $target is the file the link leads to.
configs-review-link = { $path } is a link. Saving writes the file it leads to, { $target }. The link itself stays as it is.
configs-review-summary = Lines added: { $added }. Lines removed: { $removed }. Saving asks for an administrator's password. The file as it is now is kept as { $backup }.
configs-review-summary-home = Lines added: { $added }. Lines removed: { $removed }. The file as it is now is kept as { $backup }.
configs-review-write-failed = { $path } was not written. The Console shows why. When the file changed in the meantime, compare again to see your edit against the file as it is now.
configs-review-diff = The changes: added lines start with a plus sign, removed lines with a minus sign
configs-save = Save
configs-edit-again = Edit again
configs-compare-again = Compare again
configs-discard = Discard
configs-discard-ask = Discard…
configs-discard-title = Discard your edits?
configs-discard-question = Your edits to { $path } are thrown away. The file stays as it was.
configs-discard-keep = Keep editing
configs-review-close = Close and keep the edits
configs-review-lines =
    { $count ->
        [one] Line { $first }
       *[other] Lines { $first } to { $last }
    }

configs-view-title = The text of { $path }
configs-view-link = { $path } is a link. This is the text of the file it leads to, { $target }.
configs-copy-all = Copy all
configs-copied = Copied to the clipboard.
configs-close = Close
