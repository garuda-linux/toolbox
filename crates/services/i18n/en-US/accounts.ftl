# $name is the account or group.

account-refusal-unavailable = The user accounts of this system could not be read.
account-refusal-unknown-user = There is no user named { $name }.
account-refusal-unknown-group = One of the groups does not exist, or is about to be removed.
account-refusal-root = { $name } is a root account. Garuda Toolbox does not change it.
account-refusal-current-user = { $name } is the account you are logged in with. It cannot lock or remove itself.
account-refusal-system-account = { $name } is a system account. Only accounts of people can be removed.
account-refusal-outside-user-range = { $name } has a user ID outside 1000 to 60000, so it stays as it is.
account-refusal-last-admin = { $name } is the last member of the groups wheel and sudo. Without one nobody could administer this system.
account-refusal-last-admin-lock = { $name } is the last member of the groups wheel and sudo. With a locked password nobody could administer this system.
account-refusal-lock-state-unknown = { $name } cannot be locked while it is unknown which admins are locked.
account-refusal-primary-group = { $name } is the primary group of a user, so it stays.
account-refusal-system-group = { $name } is a system group. Installed programs rely on it.
account-refusal-user-exists = A user named { $name } exists already.
account-refusal-group-exists = A group named { $name } exists already.
account-refusal-shell-not-listed = This shell is not listed in /etc/shells.
account-refusal-uid-in-use = Another account already has this user ID.
account-refusal-uid-out-of-range = Accounts of people take a user ID from 1000 to 60000, system accounts one below 1000.
account-refusal-home-taken = Another account already has this home folder.
