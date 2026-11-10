## The command line's one-line refusals. They say what the page's own
## texts say, in one lower-case line without a final full stop.

cli-switch-unknown = the catalog has no such switch
cli-switch-unmet = this switch needs another one that is off, turn that on first

cli-repair-not-offered = boot repair is only offered on the live system
cli-repair-unavailable = the disks could not be read, so nothing can be repaired
cli-repair-unknown-system = this system is not in the list anymore, see `bootrepair show`
cli-repair-in-use = the running system uses this disk, so nothing is written to it
cli-repair-unknown-target = choose where the boot loader goes first
cli-repair-unknown-layout = choose where the system lies first
cli-repair-no-passphrase = enter the passphrase of the encrypted partition

cli-account-unavailable = the user accounts of this system could not be read
cli-account-unknown-user = there is no user by this name
cli-account-unknown-group = one of the groups does not exist, or is about to be removed
cli-account-root = this is a root account, Garuda Toolbox does not change it
cli-account-current-user = this is the account you are logged in with, it cannot lock or remove itself
cli-account-system-account = this is a system account, only accounts of people can be removed
cli-account-no-admin-group = this system has no wheel group a new account can join, so it cannot be an administrator
cli-account-outside-user-range = this account has a user ID outside 1000 to 60000, so it stays as it is
cli-account-last-admin = this is the last member of the groups wheel and sudo, without one nobody could administer this system
cli-account-last-admin-lock = this is the last member of the groups wheel and sudo, with a locked password nobody could administer this system
cli-account-lock-state-unknown = this account cannot be locked while it is unknown which admins are locked
cli-account-primary-group = this is the primary group of a user, so it stays
cli-account-system-group = this is a system group, installed programs rely on it
cli-account-user-exists = a user by this name exists already
cli-account-group-exists = a group by this name exists already
cli-account-shell-not-listed = this shell is not listed in /etc/shells
cli-account-uid-in-use = another account already has this user ID
cli-account-uid-out-of-range = accounts of people take a user ID from 1000 to 60000, system accounts one below 1000
cli-account-home-taken = another account already has this home folder
cli-membership = `{ $name }`: { $why }

cli-locale-system-locale = this is the system language, choose another system language before you disable it
cli-locale-last-locale = at least one locale must stay enabled
cli-locale-not-enabled = this locale is not enabled, enable it first

cli-driver-other-file = another driver settings file in modprobe.d blacklists this driver, Garuda Toolbox only changes its own file
cli-driver-unavailable = the drivers of this system could not be read
cli-driver-not-listed = this is no network driver of this system, see `network drivers list`
cli-driver-loaded = this driver is loaded already
cli-driver-not-loaded = this driver is not loaded

cli-boot-needs-confirm =
    `{ $value }` opens a root shell at boot or switches a security layer off
    pass --force if that is what you want
cli-boot-unavailable = this system has no /etc/default/grub, so there is nothing to change
cli-boot-custom-command-line = the kernel command line is not changed here, edit /etc/default/grub instead
cli-boot-too-many-parameters = the kernel command line is full
cli-boot-unknown-theme = `{ $value }` is not an installed Plymouth theme
cli-boot-no-kernel-cmdline = this system has no /etc/kernel/cmdline, its boot entries carry their own parameters
cli-boot-custom-kernel-cmdline = the kernel command line is not changed here, edit /etc/kernel/cmdline instead
cli-boot-protected-parameter = `{ $value }` stays, without it the system finds no root file system to start
cli-boot-empty = nothing was entered
cli-boot-too-long = `{ $value }` is too long
cli-boot-invalid-character = `{ $value }` holds a character that is not allowed here, such as a blank, a quote, a backslash or a dollar sign
cli-boot-invalid-component = `{ $value }` is not a full path to a file
cli-boot-wrong-file-type = `{ $value }` is not a file of the right type
cli-boot-not-usable = `{ $value }` cannot be used here
