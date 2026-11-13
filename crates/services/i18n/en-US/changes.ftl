
changes-title = Queued changes
# The floating pill at the bottom right. $count is how many.
changes-pill =
    { $count ->
        [one] change queued
       *[other] changes queued
    }
changes-running =
    { $count ->
        [one] Applying change
       *[other] Applying changes
    }
changes-review = Review
changes-show-console = Show
changes-hide = Hide
changes-together = Applied together, one password
changes-apply-all =
    { $count ->
        [one] Apply the change
       *[other] Apply { $count } changes
    }
# The button that drops one change, named after it for screen readers.
changes-remove = Remove “{ $change }”
# Asks first, hence the ellipsis.
changes-remove-all-ask = Remove all…
changes-remove-all = Remove all
changes-remove-all-question =
    { $count ->
        [one] Remove the queued change?
       *[other] Remove all { $count } queued changes?
    }
changes-apply = Apply
changes-keep = Keep them
# Tag of a change whose last apply stopped before all its steps ended well.
change-unfinished = Did not finish
# Why Apply is disabled. A queue restored from an earlier run waits until
# every page has checked it against the system.
changes-wait-read = Apply waits until the system has been read.
changes-wait-locks = Apply waits for the lock state of the accounts. The Users and groups page says when it cannot be read.
changes-wait-aur = Apply waits for the package list of the AUR. The Packages page says when it cannot be read.

# $name is what the change is about.

change-account-lock = Lock the account { $name }
change-account-unlock = Unlock the account { $name }
change-clean-package-cache = Clean the package cache
change-dkms-build = Build the { $module } module for kernel { $kernel }
change-dns = Look up names through { $name }
change-driver-allow = Let the driver { $name } load
change-driver-blacklist = Keep the driver { $name } from loading
change-full-name-clear = Remove the full name of { $name }
change-full-name-set = Set the full name of { $user } to { $name }
change-garuda-update-flag-on = Turn on { $name } in Garuda Update
change-garuda-update-flag-off = Turn off { $name } in Garuda Update
change-group-add = Create the group { $name }
change-group-join = Add { $user } to the group { $group }
change-group-leave = Take { $user } out of the group { $group }
change-group-remove = Delete the group { $name }
change-grub-background = Use { $name } as the boot menu background
change-grub-background-none = Remove the boot menu background
change-grub-default = Start { $name } by default
change-grub-default-saved = Start the entry picked last time by default
change-grub-os-prober-on = Look for other systems for the boot menu
change-grub-os-prober-off = Stop looking for other systems for the boot menu
change-grub-parameters = Start the kernel with { $name }
change-grub-parameters-none = Start the kernel without extra parameters
change-sdboot-parameters = Build the kernel images with { $name }
change-sdboot-parameters-none = Build the kernel images without parameters
change-sdboot-plymouth-theme = Build the kernel images with the boot splash theme { $name }
change-grub-save-default-on = Remember the entry picked in the boot menu
change-grub-save-default-off = Stop remembering the entry picked in the boot menu
change-grub-submenus-on = Put older kernels in a submenu
change-grub-submenus-off = List every kernel in the main boot menu
change-grub-theme = Use the boot menu theme { $name }
change-grub-theme-none = Remove the boot menu theme
change-grub-timeout =
    { $name ->
        [one] Show the boot menu for { $name } second
       *[other] Show the boot menu for { $name } seconds
    }
change-hblock-on = Turn on ad and tracker blocking (hBlock)
change-hblock-off = Turn off ad and tracker blocking (hBlock)
change-home-folder = Move the home folder of { $user } to { $folder }
change-loader-timeout =
    { $name ->
        [one] Show the systemd-boot menu for { $name } second
       *[other] Show the systemd-boot menu for { $name } seconds
    }
change-loader-timeout-menu =
    { $name ->
        [menu-force] Show the systemd-boot menu until a key is pressed
       *[other] Hide the systemd-boot menu unless a key is held at start
    }
change-loader-timeout-unset = Use systemd-boot's own menu timeout
change-loader-default = Start { $name } by default with systemd-boot
change-loader-default-saved = Start the entry picked last time by default with systemd-boot
change-loader-default-unset = Start the first entry of the systemd-boot menu by default
change-loader-editor-on = Let anyone edit systemd-boot entries at start
change-loader-editor-off = Stop editing systemd-boot entries at start
change-loader-editor-unset = Use systemd-boot's own setting for editing entries
change-loader-auto-entries-on = Let systemd-boot add the entries it finds
change-loader-auto-entries-off = Stop systemd-boot adding the entries it finds
change-loader-auto-entries-unset = Use systemd-boot's own setting for the entries it finds
change-loader-auto-firmware-on = Add an entry for the firmware setup to systemd-boot
change-loader-auto-firmware-off = Remove the firmware setup entry from systemd-boot
change-loader-auto-firmware-unset = Use systemd-boot's own setting for the firmware entry
change-locale-enable = Add the language { $name }
change-locale-disable = Remove the language { $name }
change-login-shell = Change the login shell of { $user } to { $shell }
change-package-install = Install { $name }
change-package-remove = Remove { $name }
change-aur-install = Build and install { $name } from the AUR
change-plymouth-theme = Use the boot splash theme { $name }
change-refresh-keyring = Refresh the package signing keys
change-remove-database-lock = Remove the package database lock
change-remove-orphans = Remove packages nothing needs anymore
change-system-locale = Use { $name } as the system language
change-unit-enable = Start { $name } at boot
change-unit-disable = Stop starting { $name } at boot
change-unit-mask = Keep { $name } from starting at all
change-unit-unmask = Let { $name } start again
change-upgrade-system = Update the system
change-user-config-on = Turn on { $name }
change-user-config-off = Turn off { $name }
change-user-remove-keep-home = Delete the user { $name } and keep the home folder
change-user-remove-with-home = Delete the user { $name } with the home folder
change-user-unit-enable = Start { $name } at login
change-user-unit-disable = Stop starting { $name } at login
change-user-unit-mask = Keep the user service { $name } from starting
change-user-unit-unmask = Let the user service { $name } start again
change-wifi-backend = Run Wi-Fi through { $name }
