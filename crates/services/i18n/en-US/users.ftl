
page-users-subtitle = Add users, set passwords, lock accounts and change groups.
users-refresh = Read again
users-tabs = Users and groups tabs
users-tab-users = Users
users-tab-groups = Groups
users-dismiss = Dismiss
users-cancel = Cancel
users-list-separator = {", "}

users-search = Search users
users-search-hint = Search users…
users-search-detail = The search looks at user names, user IDs, full names, home folders, shells and groups.
users-add = Add user…
users-regular-only = Regular users only
users-regular-only-detail = Hides the accounts of the system and its services.
users-sort = Sort users
users-order-name = User name
users-order-id = User ID
users-order-full-name = Full name
users-order-home = Home folder
users-order-shell = Login shell
# $shown is how many the filter and search let through, $count all accounts.
users-count = Showing { $shown } of { $count } accounts
users-reading = Reading the accounts…
users-unavailable = User accounts are not available
users-unavailable-detail = /etc/passwd and /etc/group could not be read.
users-none-regular = No regular users
users-none-regular-detail = Clear the filter to see the accounts of the system.
users-show-all = Show all accounts
users-no-match = No user matches “{ $search }”
users-clear-search = Clear search
users-list = Users
users-keys = Arrow keys pick an account. Return edits it, the Menu key opens its other actions.

users-tag-you = You
users-tag-locked = Locked
users-tag-system = System
users-tag-queued = { $count ->
        [one] 1 change queued
       *[other] { $count } changes queued
    }
users-detail = User ID { $uid }, home { $home }, shell { $shell }
users-groups-line = Groups: { $groups }
users-not-set = not set
users-edit = Edit…
users-password = Set password…
# What a screen reader hears for the row button above. $user is the account.
users-edit-named = Edit { $user }…
users-more = More actions for { $user }
users-lock = Queue lock
users-unlock = Queue unlock
users-unqueue-lock = Don't lock
users-unqueue-unlock = Don't unlock
users-lock-unknown = Lock: state not known
users-groups = Groups…
users-remove = Remove…
users-keep = Unqueue removal
users-root = Root accounts are not changed here.

users-locks-loading = Asking which accounts are locked…
users-locks-dismissed = Which accounts are locked is not known: the password dialog was closed.
users-locks-denied = Which accounts are locked is not known: the password dialog was closed or the password was not accepted.
users-locks-no-agent = Which accounts are locked is not known: no password dialog could appear, because this desktop session runs no polkit authentication agent (the program that asks for your password when an app needs more rights).
users-locks-helper-missing = Which accounts are locked is not known: the helper of Garuda Toolbox is not installed. Reinstall Garuda Toolbox.
users-locks-unknown = Which accounts are locked is not known: the helper of Garuda Toolbox could not read it.
users-locks-again = Read again asks once more.

users-run-password-running = Setting the password of { $user }…
users-run-password-done = The password of { $user } is set.
users-run-password-failed = The password of { $user } could not be set. The Console shows why.
users-run-new-account-running = Adding the account { $user }…
users-run-new-account-done = The account { $user } is added.
users-run-new-account-failed = The account { $user } could not be added. The Console shows why.
users-show-console = Show Console

users-refused-root = { $name } is a root account. Garuda Toolbox does not change it.
users-refused-current-user = { $name } is the account you are logged in with. It cannot lock or remove itself.
users-refused-system-account = { $name } is a system account. Only accounts of people can be removed.
users-refused-outside-user-range = { $name } has a user ID outside 1000 to 60000, so it stays as it is.
users-refused-last-admin = { $name } is the last member of the groups wheel and sudo. Without one nobody could administer this system.
users-refused-last-admin-lock = { $name } is the last member of the groups wheel and sudo. With a locked password nobody could administer this system.
users-refused-lock-state-unknown = { $name } cannot be locked while it is unknown which admins are locked.
users-refused-primary-group = { $name } is the primary group of a user, so it stays.
users-refused-system-group = { $name } is a system group. Installed programs rely on it.
users-refused-user-exists = A user named { $name } exists already.
users-refused-group-exists = A group named { $name } exists already.
users-refused-unknown-user = There is no user named { $name }.
users-refused-unknown-group = One of the groups does not exist, or is about to be removed.
users-refused-shell-not-listed = This shell is not listed in /etc/shells.
users-refused-uid-invalid = The user ID is a whole number from 1 to 60000.
users-refused-uid-in-use = Another account already has this user ID.
users-refused-uid-out-of-range = Accounts of people take a user ID from 1000 to 60000, system accounts one below 1000.
users-refused-home-taken = Another account already has this home folder.
users-refused-unavailable = The user accounts of this system could not be read.

users-name-empty = Enter a name.
users-name-too-long = This is too long.
users-name-invalid-start = Names start with a lowercase letter or an underscore.
users-name-invalid-character = Only lowercase letters, digits, hyphens and underscores are allowed.
users-name-invalid = This is not valid.

users-home-empty = Enter a folder.
users-home-too-long = The home folder is too long.
users-home-not-absolute = The home folder is a full path that starts with a slash, such as /home/ada.
users-home-invalid-character = The home folder cannot hold a colon, a line break or another control character.
users-home-outer-blank = The home folder cannot start or end with a blank.
users-home-invalid-component = The home folder cannot hold “..”, a single dot, two slashes in a row or a slash at the end.
users-home-outside-roots = The home folder must lie below /home, /srv, /var/lib or /opt.

users-full-name-too-long = The full name is too long.
users-full-name-invalid-character = The full name cannot hold a colon, comma, equals sign, quotation mark or line break.
users-full-name-leading-dash = The full name cannot start with a hyphen.
users-full-name-outer-blank = The full name cannot start or end with a blank.

users-password-empty = Enter a password.
users-password-too-long = The password is too long.
users-password-control-character = The password cannot hold a line break or another control character.
users-password-mismatch = The two passwords differ.

users-add-title = Add a user
users-field-name = User name
users-field-name-hint = Lowercase letters, digits, hyphens and underscores, at most 32 characters.
users-field-full-name = Full name
users-field-shell = Login shell
users-field-shell-hint = Only shells listed in /etc/shells can be chosen.
users-shell-default = System default
users-field-home = Home folder
users-field-home-hint = Optional. A full path. Leave it empty for the usual folder below /home.
users-field-home-edit-hint = A full path such as /home/ada.
users-field-uid = User ID
users-field-uid-hint = Optional. 1000 to 60000 for a person, below 1000 for a system account.
users-no-home = No home folder
users-no-home-detail = The account gets no folder of its own.
users-system = System account
users-system-detail = For a service, not a person: a user ID below 1000, hidden by “Regular users only”.
users-field-password = Password
users-field-repeat = Repeat the password
users-admin = Administrator
# $group is the admin group, wheel.
users-admin-detail = May install software and change the system, after entering their own password. This puts the user into the group { $group }.
users-other-groups = Other groups beside the user's own one
users-groups-search = Search groups
users-groups-search-hint = Search groups…
users-add-note = The user is created right away. This asks for your own password.
users-add-confirm = Add user now
users-add-more = More options
# Beside the greyed out "Add user now": the first thing it waits for.
users-add-why-no-name = Enter a user name.
users-add-why-bad-name = The user name cannot be used. The note under it says why.
users-add-why-no-password = Enter a password.
users-add-why-passwords-differ = The passwords do not match.
users-add-why-fields = A field above says what is wrong.
users-add-why-busy = Waits until the changes being applied are done.

users-edit-title = Edit { $user }
users-move-files = Move the files to the new folder
users-move-yes = Applying this moves every file from { $old } to { $new }. The new folder must not exist yet, and { $user } must be logged out. On another disk this copies everything and can take long.
users-move-no = Only the entry of the account changes and no file is moved: everything stays in { $old }. { $new } must exist already, or { $user } has no home folder at the next login. { $user } must be logged out when you apply this.
users-home-current-user = The home folder of the account you are logged in with cannot change while it runs programs.
users-edit-note = The changes wait in the queue until you apply them.
users-queue-changes = Queue changes

users-password-title = Set the password of { $user }
users-password-note = The password is set right away and asks for your own password. This page shows when it is done. The password is not kept in the queue or in any file of Garuda Toolbox.
users-password-confirm = Set password now

users-groups-title = Groups of { $user }
users-groups-note = Changes wait in the queue until you apply them. They take effect at the user's next login.
users-choice-detail = group ID { $gid }, members: { $members }
users-choice-primary = primary group, group ID { $gid }, members: { $members }
users-choice-new = new group

users-remove-title = Remove the user { $user }?
users-remove-goes = The removal waits in the queue until you apply it. This goes:
users-remove-account = The account { $user } and its password
users-remove-group = The group { $user }, when the user is its only member
users-remove-memberships = Its membership in: { $groups }
users-remove-home-goes = The home folder { $home } with every file in it, and the mail spool
users-remove-home-stays = The home folder { $home } stays on the disk.
users-remove-delete-home = Also delete the home folder
users-remove-confirm = Queue removal

groups-search = Search groups
groups-search-hint = Search groups…
groups-search-detail = The search looks at group names, group IDs and members.
groups-add = Add group…
groups-sort = Sort groups
groups-order-name = Group name
groups-order-id = Group ID
groups-order-members = Most members
groups-list = Groups
groups-reading = Reading the groups…
groups-unavailable = Groups are not available
groups-no-match = No group matches “{ $search }”
groups-detail = Group ID { $gid }, members: { $members }
groups-detail-empty = Group ID { $gid }, no members
groups-detail-new = Not added yet
groups-tag-new = New
groups-tag-system = System
groups-tag-removed = Removal queued
groups-remove = Remove { $group }…
groups-unqueue-new = Don't add { $group }
groups-unqueue-removal = Keep { $group }
groups-remove-title = Remove the group { $group }?
groups-remove-detail = Its members leave it. Files that belong to the group keep its number as their group. The removal waits in the queue until you apply it.
groups-remove-confirm = Queue removal

groups-add-title = Add a group
groups-field-name = Group name
groups-add-note = The group waits in the queue until you apply it. Users can be put into it before that.
groups-add-confirm = Queue group
