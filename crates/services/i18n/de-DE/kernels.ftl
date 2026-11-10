kernels-subtitle = Ein neu installierter Kernel steht nach dem nächsten Neustart im Bootmenü zur Auswahl.
# $release is what `uname -r` prints.
kernels-running-release = Laufender Kernel: { $release }
kernels-refresh = Neu einlesen
kernels-reading = Kernel werden eingelesen…
kernels-unavailable = Keine Kernel verfügbar. Die Paketdatenbanken von pacman konnten nicht gelesen werden.
kernels-none = Keine Kernel gefunden. Die Repositorys bieten keinen an. Aktualisiere die Paketdatenbanken auf der Seite „Wartung“.

kernels-installed = Installiert
kernels-available = Verfügbar
kernels-count =
    { $count ->
        [one] 1 Kernel
       *[other] { $count } Kernel
    }

kernels-running = Läuft
kernels-installs = Wird beim Übernehmen installiert
kernels-removes = Wird beim Übernehmen entfernt
# $version and $repo name what the repository offers.
kernels-version = { $version } aus { $repo }
kernels-version-local = { $version }, nicht in den Repositorys
kernels-version-installed = installiert: { $installed }
kernels-running-note = Dieser Kernel läuft gerade und kann nicht entfernt werden. Starte mit einem anderen Kernel neu, um ihn zu entfernen.

kernels-headers-installed = Header installiert
kernels-headers-missing = Header fehlen
kernels-headers-queued = Header werden beim Übernehmen installiert
kernels-queue-headers = Header vormerken
kernels-unqueue-headers = Header nicht mehr vormerken

# $module and $version name one DKMS module.
kernels-module-installed = { $module } { $version }: installiert
kernels-module-built = { $module } { $version }: gebaut, nicht installiert
kernels-module-broken = { $module } { $version }: defekt
kernels-module-missing = { $module } { $version }: fehlt
kernels-module-queued = { $module } { $version }: Bau vorgemerkt
kernels-queue-builds = Modulbau vormerken
kernels-unqueue-builds = Modulbau nicht mehr vormerken

kernels-dkms-loading = DKMS-Module werden abgefragt…
kernels-dkms-unavailable = dkms ist nicht installiert, daher gibt es keine Module zu bauen.
kernels-dkms-failed = dkms konnte die Module nicht auflisten.
kernels-dkms-ok = Alle DKMS-Module sind für die Kernel mit Headern gebaut.
kernels-headers-banner =
    { $count ->
        [one] Einem Kernel fehlen die Header, daher kann DKMS keine Module für ihn bauen.
       *[other] { $count } Kerneln fehlen die Header, daher kann DKMS keine Module für sie bauen.
    }
kernels-builds-banner =
    { $count ->
        [one] Ein DKMS-Modul wurde nicht gebaut. DKMS baut für jeden Kernel zusätzliche Treiber, etwa für NVIDIA oder VirtualBox.
       *[other] { $count } DKMS-Module wurden nicht gebaut. DKMS baut für jeden Kernel zusätzliche Treiber, etwa für NVIDIA oder VirtualBox.
    }
kernels-queue-all-headers = Alle Header vormerken
kernels-queue-all-builds = Alle Modulbauten vormerken

# Why a click was not queued. $name is the kernel's package name.
kernels-refusal-running = { $name } läuft gerade und bleibt deshalb installiert.
kernels-refusal-running-unknown = Der laufende Kernel ist unbekannt, daher kann kein Kernel entfernt werden. Starte den Computer neu und versuche es erneut.
kernels-refusal-last-kernel = { $name } ist der letzte installierte Kernel. Installiere einen anderen, bevor du ihn entfernst.
kernels-dismiss = Ausblenden

# The link that opens Boot options with a kernel offered as the default.
kernels-set-default = Als Standard festlegen…
kernels-set-default-name = { $kernel } als Standardeintrag im Bootmenü festlegen
