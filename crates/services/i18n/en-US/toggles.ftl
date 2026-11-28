# The catalog toggle pages (Components, …).

toggles-search = Search this page
# The chip that shows every section.
toggles-all = All
toggles-search-hint = Search this page…
# $search is what the user typed.
toggles-nothing-matches-detail = No entry of this page has “{ $search }” in its name or description.
# A chip picked a section without a match while other sections have some. $section is the section's title, $search what the user typed.
toggles-nothing-in-section = Nothing in { $section } matches
toggles-nothing-in-section-detail = Other sections have entries with “{ $search }” in their name or description.
# $count is how many entries of the whole page match the search.
toggles-show-all-matches =
    { $count ->
        [one] Show the Match
       *[other] Show All { $count } Matches
    }
# $on and $total are counts of the entries of a section, a chip or the page, shown as "2/8".
toggles-section-count = { $on }/{ $total }
# The count while some states could not be read, so how many are on is not known. $total is the count of entries.
toggles-section-count-unread = ?/{ $total }
toggles-unknown =
    { $count ->
        [one] The state of one entry could not be read, so its switch may show the wrong state.
       *[other] The state of { $count } entries could not be read, so their switches may show the wrong state.
    }
toggles-state-unknown = State not known.
# $names lists the titles of the entries, joined by commas.
toggles-needs = Needs { $names }
toggles-needs-any = Needs one of: { $names }
toggles-turns-on = Turns on with Apply
toggles-enable = Enable
toggles-disable = Disable
toggles-turns-off = Turns off with Apply
# Beside the title of a setting that is on, as "Installed" is beside a package.
toggles-enabled = Enabled
toggles-dismiss = Dismiss

# Why the catalog could not be loaded.
toggles-problem = The list could not be loaded.
toggles-problem-unreadable = A catalog file could not be read.
toggles-problem-syntax = A catalog file is damaged.
toggles-problem-invalid = An entry of a catalog file breaks a rule.
toggles-problem-reason = The list could not be loaded. { $reason }

# A catalog file that failed to load, then the entry in it and its problem.
# $value is the value the file holds, such as a name, a path or a number.
catalog-error-read = { $path }: { $error }
catalog-error-syntax = { $source }: { $error }
catalog-error-entry = { $source }: entry "{ $id }": { $problem }
catalog-check-package = the package { $name }
catalog-check-service = the service { $name }
catalog-check-user-service = the user service { $name }
catalog-check-group = the group { $name }
catalog-check-user-config = the user config { $edit }
catalog-problem-duplicate-check = another entry watches { $value }
catalog-problem-bad-name = invalid { $kind } `{ $value }`: { $reason }
catalog-problem-bad-id = the id must be lowercase words joined by dashes
catalog-problem-duplicate-id = the id is used twice
catalog-problem-duplicate-path = another entry points at "{ $value }"
catalog-problem-admin-group = the group "{ $value }" grants root, it is changed on the Users page only
catalog-problem-empty-name = empty name
catalog-problem-empty-title = empty title
catalog-problem-empty-description = empty description
catalog-problem-hidden-character = "{ $value }" holds a hidden character
catalog-problem-empty-icon = empty icon
catalog-problem-no-entries = no entries
catalog-problem-empty-requirement = the requires list is empty
catalog-problem-unknown-requirement = requires "{ $value }", which is no entry of this catalog
catalog-problem-requirement-repeats = requires "{ $value }" twice
catalog-problem-requires-itself = the entry requires itself
catalog-problem-requirement-cycle = the requirements form a cycle: { $value }
catalog-problem-no-files = no files
catalog-problem-outside-skel = "{ $value }" is not below /etc/skel/
catalog-problem-parent-component = "{ $value }" contains ".."
catalog-problem-not-writable = "{ $value }" must be both a root-owned entry without read_only and a line of toolbox_plan::WRITABLE_CONFIG_FILES or WRITABLE_CONFIG_FOLDERS
catalog-problem-read-only-writable = "{ $value }" is read_only, so it must be neither a line of toolbox_plan::WRITABLE_CONFIG_FILES or WRITABLE_CONFIG_FOLDERS nor a file in one of those folders
catalog-problem-read-only-home = read_only needs a root-owned path
catalog-problem-empty-content = empty content
catalog-problem-reload-without-user-config = reload needs a user-file, user-ini or user-json check
catalog-problem-unknown-reload = unknown reload "{ $value }", known: kwin
catalog-problem-restart-without-user-config = restart-user-units needs a user-file, user-ini or user-json check
catalog-problem-restart-repeats = restart-user-units names "{ $value }" twice
catalog-problem-empty-desktop = the desktop list is empty or has an empty name
catalog-problem-also-without-package = also needs a package check
catalog-problem-empty-also = the also list is empty
catalog-problem-also-repeats = also names "{ $value }" twice
catalog-problem-empty-conflicts = the conflicts list is empty
catalog-problem-conflict-repeats = conflicts names "{ $value }" twice
catalog-problem-unknown-conflict = conflicts with "{ $value }", which is no entry of this catalog
catalog-problem-conflicts-itself = the entry conflicts with itself
catalog-problem-conflicts-requirement = conflicts with "{ $value }", which it also requires
catalog-problem-bad-icon = the icon "{ $value }" is no plain file name
catalog-problem-missing-icon = the icon "{ $value }" is missing from the icon folder
catalog-problem-bad-glyph = the glyph "{ $value }" is no freedesktop icon name
catalog-problem-protected-also = the `also` package "{ $value }" is protected, so the entry could never go off
catalog-problem-bad-url = the url "{ $value }" is no plain https link
catalog-problem-bad-colour = "{ $value }" is no colour of the form #rrggbb
catalog-problem-ansi-count = ansi has { $value } colours, it needs 16
catalog-problem-empty-licence = empty licence
catalog-problem-home-path-empty = the home path is empty
catalog-problem-home-path-too-long = the home path is longer than { $value } bytes
catalog-problem-home-path-absolute = the home path `{ $value }` is absolute, it must be relative to the home
catalog-problem-home-path-invalid-character = the home path contains `{ $value }`
catalog-problem-home-path-invalid-component = the home path `{ $value }` has an empty, `.` or `..` component
catalog-problem-config-word-empty = the config word is empty
catalog-problem-config-word-too-long = the config word is longer than { $value } bytes
catalog-problem-config-word-invalid-character = the config word contains `{ $value }`
catalog-problem-config-word-padded = the config word `{ $value }` starts or ends with a blank
catalog-problem-config-text-too-large = the file is larger than { $value } bytes
catalog-problem-config-text-not-text = the file is no text
catalog-problem-config-path-empty = the config file path is empty
catalog-problem-config-path-too-long = the config file path is longer than { $value } bytes
catalog-problem-config-path-relative = the config file path `{ $value }` is not absolute
catalog-problem-config-path-invalid-character = the config file path contains `{ $value }`
catalog-problem-config-path-invalid-component = the config file path `{ $value }` has an empty, overlong, `.` or `..` component
catalog-problem-config-path-outside-roots = the config file path `{ $value }` is outside the folders the toolbox edits
catalog-problem-config-path-withheld = `{ $value }` is never read or written as a config file
