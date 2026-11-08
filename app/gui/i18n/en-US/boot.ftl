page-boot-subtitle = Applying the changes writes /etc/default/grub, keeps the old file as a backup and builds the boot menu again.
page-boot-subtitle-sdboot = The settings of systemd-boot and the kernel command line that is built into each kernel image.
boot-refresh = Read again
boot-queued = Changes with Apply

boot-failed = The GRUB settings could not be read
boot-failed-detail = The note at the bottom of the window says why. Read again tries once more.
boot-missing = GRUB is not the boot loader of this system
boot-missing-detail = There is no /etc/default/grub, so this page has nothing to change. Other boot loaders are configured with their own tools.
boot-loading = Reading the boot settings…
boot-vm = You are running in a virtual machine. The boot splash works there in a limited way.

# Why an edit was not queued.
boot-refused-unavailable = This system has no /etc/default/grub, so there is nothing to change.
boot-refused-custom-command-line = The kernel command line is not changed here. Edit /etc/default/grub on the Config Files page.
boot-refused-too-many-parameters = The kernel command line is full.
boot-refused-unknown-theme = This is not an installed Plymouth theme: { $value }
boot-refused-empty = Nothing was entered.
boot-refused-too-long = This is too long: { $value }
boot-refused-invalid-character = This holds a character that is not allowed here, such as a blank, a quote, a backslash or a dollar sign: { $value }
boot-refused-invalid-path = This is not a full path to a file: { $value }
boot-refused-wrong-type = This is not a file of the right type: { $value }
boot-refused-invalid = This cannot be used here: { $value }
boot-dismiss = Dismiss

boot-menu = Boot menu
boot-default = Default entry
boot-default-known = The menu entry that is booted when you do not pick another.
boot-default-missing = GRUB has not generated a boot menu yet, so there are no entries to choose from.
boot-default-dismissed = The password dialog was closed, so the boot menu was not read and its entries are not listed. Read again tries once more.
boot-default-denied = The boot menu was not read: the password dialog was closed or the password was not accepted. Its entries are not listed. Read again tries once more.
boot-default-no-agent = The boot menu was not read, because no password dialog could appear: this desktop session runs no polkit authentication agent. Its entries are not listed.
boot-default-helper-missing = The boot menu was not read, because the helper of Garuda Toolbox is not installed. Its entries are not listed. Reinstall Garuda Toolbox.
boot-default-unknown = The boot menu could not be read, so its entries are not listed. Read again tries once more.
boot-default-not-asked = The boot menu has not been read yet. Read again reads it.
boot-default-loading = Reading the boot menu…
boot-default-remembered = GRUB boots the entry that was booted last.
boot-default-last = The entry booted last
boot-entry-in = { $title } (in { $parent })
boot-entry-unlisted = { $entry } (not in the boot menu)
boot-remember = Remember the last entry
boot-remember-detail = Makes the entry you booted last the default for the next start.
boot-timeout = Menu timeout
boot-timeout-detail = The seconds the menu waits before it boots the default entry.
boot-timeout-unknown = The file holds a timeout that is no number. Choosing one here replaces it.
boot-timeout-seconds = Menu timeout in seconds
# The unit after the timeout's number, the symbol for seconds.
boot-timeout-unit = s
boot-timeout-less = One second less
boot-timeout-more = One second more
boot-submenus = Group entries in submenus
boot-submenus-detail = Off shows every entry in the top menu, without "Advanced options".
boot-os-prober = Search for other operating systems
boot-os-prober-detail = Adds other installed systems to the boot menu with os-prober.

# A kernel the Kernels page proposed as the default.
boot-proposal-offer = The menu entry of the kernel { $kernel } can become the default. Nothing is queued until you choose Queue as default.
boot-proposal-same = The kernel { $kernel } already is the default entry.
boot-proposal-remember = GRUB boots the entry that was booted last. Switch Remember the last entry off to make the kernel { $kernel } the default.
boot-proposal-none = The boot menu has no entry for the kernel { $kernel }. Pick the entry yourself, or generate the menu again by applying any change of this page.
boot-proposal-loading = Looking for the boot menu entry of the kernel { $kernel }…
boot-proposal-queue = Queue as default
boot-proposal-keep = Do not change
boot-proposal-close = Close

boot-kernel = Kernel parameters
boot-parameters = Parameters passed to the kernel
boot-parameters-detail = Each parameter is one word such as nowatchdog or loglevel=3. They are written to GRUB_CMDLINE_LINUX_DEFAULT.
boot-parameters-none = No parameters.
boot-parameters-custom = The kernel command line holds quotes or other characters this page does not change: { $line } Edit /etc/default/grub on the Config Files page instead.
boot-parameter-new = New kernel parameter
boot-parameter-hint = New parameter
boot-parameter-add = Add
boot-parameter-remove = Remove { $parameter }
boot-flag-quiet = Hide most boot messages (quiet)
boot-flag-quiet-detail = Off shows every message of the kernel while the system starts.
boot-flag-splash = Show a boot splash (splash)
boot-flag-splash-detail = Shows a graphical boot animation instead of text. It is drawn by Plymouth, which is queued for installation when it is missing.
boot-flag-mitigations-off = Turn off CPU mitigations (mitigations=off)
boot-flag-mitigations-off-detail = Disables CPU security mitigations for better performance but less security.
boot-flag-sysrq = Enable the Magic SysRq key (sysrq_always_enabled=1)
boot-flag-sysrq-detail = Keeps the key combinations for emergency system control switched on.

boot-look = Appearance
boot-plymouth = Boot splash theme
boot-plymouth-detail = The animation Plymouth shows while the system starts. Applying it builds the initramfs again (the small start-up system the kernel loads first), which takes a while.
boot-plymouth-queued = Plymouth is queued for installation. Choose a theme after it is installed.
boot-plymouth-missing = Plymouth is not installed, so there are no themes to choose from.
boot-plymouth-none = Not available
boot-background = GRUB background picture
boot-background-detail = A PNG, JPEG or TGA picture behind the boot menu. A GRUB theme has its own background.
boot-background-from-theme = Set by the GRUB theme (GRUB_MENU_PICTURE). Choose a picture to replace it.
boot-background-title = Choose a background picture
boot-background-filter = Pictures
boot-theme = GRUB theme
boot-theme-detail = The theme.txt file of a GRUB theme, usually below /usr/share/grub/themes.
boot-theme-title = Choose a GRUB theme
boot-theme-filter = GRUB theme
boot-file-none = None
# $name is the field's name, such as GRUB theme.
boot-file-choose = Choose…
boot-file-choose-name = Choose { $name }
boot-file-clear = Use none

# systemd-boot, shown instead of the GRUB settings.
boot-sdboot-unset = systemd-boot's default
boot-sdboot-on = On
boot-sdboot-off = Off
boot-sdboot-default-first = The first entry of the menu
boot-sdboot-timeout-seconds = Seconds
boot-sdboot-timeout-detail = How long the menu waits before it starts the default entry.
boot-sdboot-timeout-menu-force = The menu waits for a key
boot-sdboot-timeout-menu-hidden = Hidden unless a key is held at start
boot-sdboot-timeout-menu-disabled = The menu is never shown
boot-sdboot-editor = Edit entries at boot
boot-sdboot-editor-detail = Lets anyone at the machine change the kernel command line in the menu.
boot-sdboot-auto-entries = Entries systemd-boot finds itself
boot-sdboot-auto-entries-detail = Other systems on the disk, such as Windows.
boot-sdboot-auto-firmware = Firmware settings entry
boot-sdboot-auto-firmware-detail = An entry that restarts into the firmware's setup.
boot-sdboot-parameters-detail = They are in /etc/kernel/cmdline and built into each kernel image.
boot-sdboot-no-cmdline = This system has no /etc/kernel/cmdline. Its boot entries carry their own parameters.
