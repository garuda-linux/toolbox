# The Boot options page: the variables of /etc/default/grub and the
# Plymouth theme, all queued for Apply.

page-boot-subtitle = Beim Übernehmen wird /etc/default/grub geschrieben, die alte Datei gesichert und das Bootmenü neu erstellt.
page-boot-subtitle-sdboot = Die Einstellungen von systemd-boot und die Kernel-Befehlszeile, die in jedes Kernel-Image eingebaut wird.
boot-refresh = Neu einlesen
boot-queued = Ändert sich beim Übernehmen

boot-failed = GRUB-Einstellungen konnten nicht gelesen werden
boot-failed-detail = Der Grund steht unten im Fenster. Mit „Neu einlesen“ erneut versuchen.
boot-missing = Dieses System verwendet nicht GRUB als Bootloader
boot-missing-detail = /etc/default/grub fehlt, daher kann diese Seite nichts ändern. Andere Bootloader werden mit ihren eigenen Werkzeugen eingerichtet.
boot-loading = Boot-Einstellungen werden gelesen…
boot-vm = Dieses System läuft in einer virtuellen Maschine. Der Splash-Screen funktioniert dort nur eingeschränkt.

# Why an edit was not queued.
boot-refused-unavailable = Auf diesem System fehlt /etc/default/grub, daher gibt es nichts zu ändern.
# The question before a kernel parameter such as init=/bin/sh. $value is the parameter.
boot-risky-title = { $value } hinzufügen?
boot-risky-body = Dieser Parameter öffnet beim Start eine root-Shell oder schaltet eine Schutzschicht ab. Dann könnte jeder am Computer die Kontrolle übernehmen. Nur hinzufügen, wenn bekannt ist, wozu er gebraucht wird.
boot-risky-confirm = Parameter hinzufügen
boot-risky-cancel = Abbrechen
boot-refused-custom-command-line = Die Kernel-Befehlszeile lässt sich hier nicht ändern. Die Datei /etc/default/grub auf der Seite „Konfigurationsdateien“ bearbeiten.
boot-refused-too-many-parameters = Die Kernel-Befehlszeile ist voll.
boot-refused-unknown-theme = Kein installiertes Plymouth-Theme: { $name }
boot-refused-no-kernel-cmdline = Dieses System hat keine /etc/kernel/cmdline. Seine Booteinträge bringen ihre eigenen Parameter mit, daher gibt es nichts zu ändern.
boot-refused-custom-kernel-cmdline = Die Kernel-Befehlszeile lässt sich hier nicht ändern. Die Datei /etc/kernel/cmdline auf der Seite „Konfigurationsdateien“ bearbeiten.
boot-refused-protected-parameter = { $name } bleibt: Ohne diesen Parameter findet das System kein Root-Dateisystem zum Starten.
boot-refused-empty = Es wurde nichts eingegeben.
boot-refused-too-long = Die Eingabe ist zu lang: { $name }
boot-refused-invalid-character = Die Eingabe enthält ein unzulässiges Zeichen, etwa ein Leerzeichen, Anführungszeichen, einen Backslash oder ein Dollarzeichen: { $name }
boot-refused-invalid-path = Das ist kein vollständiger Dateipfad: { $name }
boot-refused-wrong-type = Die Datei hat den falschen Typ: { $name }
boot-refused-invalid = Dieser Wert ist hier nicht zulässig: { $name }
boot-dismiss = Ausblenden

boot-menu = Bootmenü
boot-default = Standardeintrag
boot-default-known = Der Menüeintrag, der startet, wenn kein anderer gewählt wird.
boot-default-missing = GRUB hat noch kein Bootmenü erstellt, daher gibt es keine Einträge zur Auswahl.
boot-default-dismissed = Die Passwortabfrage wurde geschlossen, daher fehlen die Einträge des Bootmenüs. Mit „Neu einlesen“ erneut versuchen.
boot-default-denied = Die Einträge des Bootmenüs fehlen: Der Passwortdialog wurde geschlossen oder das Passwort nicht angenommen. Mit „Neu einlesen“ erneut versuchen.
boot-default-no-agent = Die Einträge des Bootmenüs fehlen, weil keine Passwortabfrage erscheinen konnte: In dieser Sitzung läuft kein polkit-Agent.
boot-default-helper-missing = Die Einträge des Bootmenüs fehlen, weil das Hilfsprogramm von Garuda Toolbox nicht installiert ist. Garuda Toolbox neu installieren.
boot-default-broken = Die Einträge des Bootmenüs fehlen, weil pkexec auf diesem System nicht laufen kann.
boot-default-unknown = Das Bootmenü konnte nicht gelesen werden, daher fehlen seine Einträge. Mit „Neu einlesen“ erneut versuchen.
boot-default-not-asked = Das Bootmenü wurde noch nicht gelesen. Mit „Neu einlesen“ wird es eingelesen.
boot-default-loading = Das Bootmenü wird eingelesen…
boot-default-remembered = GRUB startet den zuletzt gestarteten Eintrag.
boot-default-last = Zuletzt gestarteter Eintrag
boot-entry-in = { $title } (in { $parent })
boot-entry-unlisted = { $entry } (nicht im Bootmenü)
boot-remember = Letzten Eintrag merken
boot-remember-detail = Macht den zuletzt gestarteten Eintrag zum Standard für den nächsten Start.
boot-timeout = Wartezeit des Menüs
boot-timeout-detail = Wie lange das Menü wartet, bevor es den Standardeintrag startet.
boot-timeout-unknown = Die eingetragene Wartezeit ist keine Zahl. Ein hier gewählter Wert ersetzt sie.
boot-timeout-seconds = Wartezeit des Menüs in Sekunden
# The unit after the timeout's number, the symbol for seconds.
boot-timeout-unit = s
boot-timeout-less = Eine Sekunde weniger
boot-timeout-more = Eine Sekunde mehr
boot-submenus = Einträge in Untermenüs gruppieren
boot-submenus-detail = Ohne diese Option stehen alle Einträge im Hauptmenü, und es gibt kein Untermenü „Erweiterte Optionen“.
boot-os-prober = Nach anderen Betriebssystemen suchen
boot-os-prober-detail = Fügt andere installierte Systeme mit os-prober zum Bootmenü hinzu.

# A kernel the Kernels page proposed as the default.
boot-proposal-offer = Der Menüeintrag des Kernels { $kernel } kann Standard werden. Vorgemerkt wird er erst mit „Als Standard vormerken“.
boot-proposal-same = Der Kernel { $kernel } ist bereits der Standardeintrag.
boot-proposal-remember = GRUB startet den zuletzt gestarteten Eintrag. „Letzten Eintrag merken“ ausschalten, um den Kernel { $kernel } zum Standard zu machen.
boot-proposal-none = Das Bootmenü hat keinen Eintrag für den Kernel { $kernel }. Den Eintrag selbst wählen oder das Menü neu erstellen, indem eine beliebige Änderung auf dieser Seite übernommen wird.
boot-proposal-sdboot-none = systemd-boot hat noch kein Image des Kernels { $kernel }. Sein Image wird gebaut, wenn der Kernel installiert oder aktualisiert wird. Den Eintrag selbst wählen, sobald er vorhanden ist.
boot-proposal-loading = Der Menüeintrag des Kernels { $kernel } wird gesucht…
boot-proposal-queue = Als Standard vormerken
boot-proposal-keep = Nicht ändern
boot-proposal-close = Schließen

boot-kernel = Kernel-Parameter
boot-parameters = Parameter für den Kernel
boot-parameters-detail = Jeder Parameter ist ein Wort wie nowatchdog oder loglevel=3. Sie werden in GRUB_CMDLINE_LINUX_DEFAULT geschrieben.
boot-parameters-none = Keine Parameter.
boot-parameters-custom = Die Kernel-Befehlszeile enthält Anführungszeichen oder andere Zeichen, die diese Seite nicht bearbeiten kann: { $line } Stattdessen die Datei /etc/default/grub auf der Seite „Konfigurationsdateien“ bearbeiten.
boot-parameter-new = Neuer Kernel-Parameter
boot-parameter-hint = Neuer Parameter
boot-parameter-add = Hinzufügen
boot-parameter-remove = { $parameter } entfernen
boot-parameter-kept = { $parameter } bleibt: Ohne diesen Parameter findet das System kein Root-Dateisystem zum Starten.
boot-flag-quiet = Die meisten Startmeldungen ausblenden (quiet)
boot-flag-quiet-detail = Ohne diese Option zeigt der Kernel beim Systemstart alle Meldungen.
boot-flag-splash = Splash-Screen anzeigen (splash)
boot-flag-splash-detail = Zeigt beim Start eine grafische Animation statt Text. Sie stammt von Plymouth, das bei Bedarf zur Installation vorgemerkt wird.
boot-flag-mitigations-off = CPU-Schutzmaßnahmen ausschalten (mitigations=off)
boot-flag-mitigations-off-detail = Schaltet die Schutzmaßnahmen gegen CPU-Sicherheitslücken ab. Das bringt mehr Leistung, aber weniger Sicherheit.
boot-flag-sysrq = Magische SysRq-Taste einschalten (sysrq_always_enabled=1)
boot-flag-sysrq-detail = Hält die Notfall-Tastenkombinationen des Kernels dauerhaft aktiv.

boot-look = Aussehen
boot-plymouth = Theme des Splash-Screens
boot-plymouth-detail = Die Animation, die Plymouth beim Systemstart zeigt. Beim Übernehmen wird die Initramfs neu erstellt (das kleine Startsystem, das der Kernel zuerst lädt). Das dauert eine Weile.
boot-plymouth-queued = Plymouth ist zur Installation vorgemerkt. Ein Theme wählen, sobald es installiert ist.
boot-plymouth-missing = Plymouth ist nicht installiert, daher gibt es keine Themes zur Auswahl.
boot-plymouth-none = Nicht verfügbar
boot-background = GRUB-Hintergrundbild
boot-background-detail = Ein PNG-, JPEG- oder TGA-Bild hinter dem Bootmenü. Ein GRUB-Theme hat seinen eigenen Hintergrund.
boot-background-from-theme = In /etc/default/grub als GRUB_MENU_PICTURE gesetzt, das manche GRUB-Themes lesen. Ein Bild wählen, um es zu ersetzen.
boot-background-title = Hintergrundbild wählen
boot-background-filter = Bilder
boot-theme = GRUB-Theme
boot-theme-detail = Die Datei theme.txt eines GRUB-Themes, meist unter /usr/share/grub/themes.
boot-theme-title = GRUB-Theme wählen
boot-theme-filter = GRUB-Theme
boot-file-none = Keines
boot-file-choose = Wählen…
boot-file-choose-name = { $name } wählen
boot-file-clear = { $name } leeren
boot-file-chooser-failed = Der Dateidialog ließ sich nicht öffnen. Ein Dateiauswahl-Portal wie xdg-desktop-portal-kde oder xdg-desktop-portal-gtk installieren und danach erneut versuchen.

# systemd-boot, statt der GRUB-Einstellungen gezeigt.
boot-sdboot-unset = Vorgabe von systemd-boot
boot-sdboot-segment-default = Vorgabe
boot-sdboot-on = An
boot-sdboot-off = Aus
boot-sdboot-default-first = Der erste Eintrag des Menüs
boot-sdboot-timeout-seconds = Sekunden
boot-sdboot-timeout-detail = Wie lange das Menü wartet, bevor es den Standardeintrag startet.
boot-sdboot-timeout-menu-force = Das Menü wartet auf einen Tastendruck
boot-sdboot-timeout-menu-hidden = Verborgen, außer wenn beim Start eine Taste gehalten wird
boot-sdboot-timeout-menu-disabled = Das Menü wird nie gezeigt
boot-sdboot-editor = Einträge beim Start bearbeiten
boot-sdboot-editor-detail = Erlaubt allen am Rechner, die Kernel-Befehlszeile im Menü zu ändern.
boot-sdboot-auto-entries = Einträge, die systemd-boot selbst findet
boot-sdboot-auto-entries-detail = Andere Systeme auf der Platte, etwa Windows.
boot-sdboot-auto-firmware = Eintrag für die Firmware-Einstellungen
boot-sdboot-auto-firmware-detail = Ein Eintrag, der in die Einrichtung der Firmware neu startet.
boot-sdboot-parameters-detail = Jeder Parameter ist ein Wort wie nowatchdog oder loglevel=3. Sie werden in /etc/kernel/cmdline geschrieben und in jedes Kernel-Image eingebaut.
boot-sdboot-parameters-custom = Die Kernel-Befehlszeile enthält Anführungszeichen oder andere Zeichen, die diese Seite nicht bearbeiten kann: { $line } Stattdessen die Datei /etc/kernel/cmdline auf der Seite „Konfigurationsdateien“ bearbeiten.
boot-sdboot-cmdline-running = /etc/kernel/cmdline gibt es noch nicht, daher sind das die Parameter, mit denen das System gestartet ist. Die erste Änderung legt die Datei an.
boot-sdboot-no-cmdline = Dieses System hat keine /etc/kernel/cmdline. Seine Booteinträge bringen ihre eigenen Parameter mit.
# Over the menu card. $count is how many entry files could not be read.
boot-sdboot-skipped =
    { $count ->
        [one] Ein Booteintrag konnte nicht gelesen werden.
       *[other] { $count } Booteinträge konnten nicht gelesen werden.
    }
# $count is how many entries are left out, $limit how many the page shows.
boot-sdboot-cut =
    { $count ->
        [one] Ein weiterer Eintrag fehlt, die Seite zeigt die ersten { $limit }.
       *[other] { $count } weitere Einträge fehlen, die Seite zeigt die ersten { $limit }.
    }
