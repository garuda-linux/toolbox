kernels-subtitle = A kernel you install is used after the next restart, chosen in the boot menu.
# $release is what `uname -r` prints.
kernels-running-release = Running kernel: { $release }
kernels-refresh = Read again
kernels-reading = Reading the kernels…
kernels-unavailable = Kernels are not available on this system: the package databases of pacman could not be read.
kernels-none = No kernels found. The repositories offer none: update the package databases on the Maintenance page.

kernels-installed = Installed
kernels-available = Available
kernels-count =
    { $count ->
        [one] 1 kernel
       *[other] { $count } kernels
    }

kernels-running = Running
kernels-installs = Installs with Apply
kernels-removes = Removes with Apply
# $version and $repo name what the repository offers.
kernels-version = { $version } from { $repo }
kernels-version-local = { $version }, in no repository
kernels-version-installed = installed { $installed }
kernels-running-note = This kernel runs right now, so it stays. Restart with another kernel to remove it.

kernels-headers-installed = Headers installed
kernels-headers-missing = Headers missing
kernels-headers-queued = Headers install with Apply
kernels-queue-headers = Queue headers
kernels-unqueue-headers = Don't install headers

# $module and $version name one DKMS module.
kernels-module-installed = { $module } { $version }: installed
kernels-module-built = { $module } { $version }: built, not installed
kernels-module-broken = { $module } { $version }: broken
kernels-module-missing = { $module } { $version }: missing
kernels-module-queued = { $module } { $version }: build queued
kernels-queue-builds = Queue module builds
kernels-unqueue-builds = Don't build modules

kernels-dkms-loading = Asking dkms for its modules…
kernels-dkms-unavailable = dkms is not installed, so there are no modules to build.
kernels-dkms-failed = dkms could not list its modules.
kernels-dkms-ok = Every DKMS module is built for the kernels with headers.
kernels-headers-banner =
    { $count ->
        [one] One kernel lacks its headers, so DKMS cannot build modules for it.
       *[other] { $count } kernels lack their headers, so DKMS cannot build modules for them.
    }
kernels-builds-banner =
    { $count ->
        [one] One DKMS module is not built. DKMS builds extra drivers, such as those of NVIDIA or VirtualBox, for each kernel.
       *[other] { $count } DKMS modules are not built. DKMS builds extra drivers, such as those of NVIDIA or VirtualBox, for each kernel.
    }
kernels-queue-all-headers = Queue all headers
kernels-queue-all-builds = Queue all builds

# Why a click was not queued. $name is the kernel's package name.
kernels-refusal-running = { $name } is the kernel that runs right now, so it stays installed.
kernels-refusal-running-unknown = The kernel that runs right now is not known, so no kernel can be removed. Restart the computer and try again.
kernels-refusal-last-kernel = { $name } is the last installed kernel. Install another one before you remove it.
kernels-dismiss = Dismiss

# The link that opens Boot options with a kernel offered as the default.
kernels-set-default = Set as default…
kernels-set-default-name = Set { $kernel } as the default boot entry
