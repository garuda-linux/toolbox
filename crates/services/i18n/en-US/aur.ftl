# Why a setup builds an AUR package through paru's own way. $reason is one
# of the aur-route texts below.
aur-old-path = This setup uses the old paru path: { $reason }

aur-route-no-account = the account has no entry in /etc/passwd, as with systemd-homed or LDAP accounts
aur-route-linked = the home folder or ~/.cache is a link, or a folder the build needs is missing
aur-route-shared = a folder the build uses can be written by other users
aur-route-through-sudo = the command line asks for the password through sudo, which does not say who asked
aur-route-other = the build folder cannot be checked

# Why a queued AUR build has no build order yet. $name is the package.
aur-order-unread = { $name } has no build order yet: paru could not be asked.
aur-order-route = { $name } has no build order: its build folder cannot be used.
aur-order-missing = { $name } has no build order: the AUR does not have it.
aur-order-not-named = { $name } has no build order: paru does not build it from the AUR.
