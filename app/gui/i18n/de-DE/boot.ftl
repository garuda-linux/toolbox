# The Boot options page: the variables of /etc/default/grub and the
# Plymouth theme, all queued for Apply.

page-boot-subtitle = Beim Übernehmen wird /etc/default/grub geschrieben, die alte Datei gesichert und das Bootmenü neu erstellt.
boot-refresh = Neu einlesen
boot-queued = Ändert sich beim Übernehmen

boot-failed = GRUB-Einstellungen konnten nicht gelesen werden
boot-failed-detail = Den Grund findest du unten im Fenster. Mit „Neu einlesen“ versuchst du es erneut.
boot-missing = Dieses System verwendet nicht GRUB als Bootloader
boot-missing-detail = /etc/default/grub fehlt, daher kann diese Seite nichts ändern. Andere Bootloader richtest du mit ihren eigenen Werkzeugen ein.
boot-loading = Boot-Einstellungen werden gelesen…
boot-vm = Du nutzt eine virtuelle Maschine. Der Splash-Screen funktioniert dort nur eingeschränkt.

# Why an edit was not queued.
boot-refused-unavailable = Auf diesem System fehlt /etc/default/grub, daher gibt es nichts zu ändern.
boot-refused-custom-command-line = Die Kernel-Befehlszeile lässt sich hier nicht ändern. Bearbeite /etc/default/grub auf der Seite „Konfigurationsdateien“.
boot-refused-too-many-parameters = Die Kernel-Befehlszeile ist voll.
boot-refused-unknown-theme = Kein installiertes Plymouth-Theme: { $value }
boot-refused-empty = Es wurde nichts eingegeben.
boot-refused-too-long = Die Eingabe ist zu lang: { $value }
boot-refused-invalid-character = Die Eingabe enthält ein unzulässiges Zeichen, etwa ein Leerzeichen, Anführungszeichen, einen Backslash oder ein Dollarzeichen: { $value }
boot-refused-invalid-path = Das ist kein vollständiger Dateipfad: { $value }
boot-refused-wrong-type = Die Datei hat den falschen Typ: { $value }
boot-refused-invalid = Dieser Wert ist hier nicht zulässig: { $value }
boot-dismiss = Ausblenden

boot-menu = Bootmenü
boot-default = Standardeintrag
boot-default-known = Der Menüeintrag, der gestartet wird, wenn du keinen anderen wählst.
boot-default-missing = GRUB hat noch kein Bootmenü erstellt, daher gibt es keine Einträge zur Auswahl.
boot-default-dismissed = Die Passwortabfrage wurde geschlossen, daher fehlen die Einträge des Bootmenüs. Mit „Neu einlesen“ versuchst du es erneut.
boot-default-denied = Die Einträge des Bootmenüs fehlen, weil zum Lesen das Passwort eines Administrators nötig ist. Mit „Neu einlesen“ versuchst du es erneut.
boot-default-no-agent = Die Einträge des Bootmenüs fehlen, weil keine Passwortabfrage erscheinen konnte: In dieser Sitzung läuft kein polkit-Agent.
boot-default-helper-missing = Die Einträge des Bootmenüs fehlen, weil das Hilfsprogramm von Garuda Toolbox nicht installiert ist. Installiere Garuda Toolbox neu.
boot-default-unknown = Das Bootmenü konnte nicht gelesen werden, daher fehlen seine Einträge. Mit „Neu einlesen“ versuchst du es erneut.
boot-default-not-asked = Das Bootmenü wurde noch nicht gelesen. Mit „Neu einlesen“ liest du es ein.
boot-default-loading = Das Bootmenü wird eingelesen…
boot-default-remembered = GRUB startet den zuletzt gewählten Eintrag.
boot-default-last = Zuletzt gestarteter Eintrag
boot-entry-in = { $title } (in { $parent })
boot-entry-unlisted = { $entry } (nicht im Bootmenü)
boot-remember = Letzten Eintrag merken
boot-remember-detail = Macht den zuletzt gestarteten Eintrag zum Standard für den nächsten Start.
boot-timeout = Wartezeit des Menüs
boot-timeout-detail = Wie viele Sekunden das Menü wartet, bevor es den Standardeintrag startet.
boot-timeout-unknown = Die eingetragene Wartezeit ist keine Zahl. Wählst du hier einen Wert, wird sie ersetzt.
boot-timeout-seconds = Wartezeit des Menüs in Sekunden
# The unit after the timeout's number, the symbol for seconds.
boot-timeout-unit = s
boot-timeout-less = Eine Sekunde weniger
boot-timeout-more = Eine Sekunde mehr
boot-submenus = Einträge in Untermenüs gruppieren
boot-submenus-detail = Ist das aus, stehen alle Einträge im Hauptmenü, ohne „Erweiterte Optionen“.
boot-os-prober = Nach anderen Betriebssystemen suchen
boot-os-prober-detail = Fügt andere installierte Systeme mit os-prober zum Bootmenü hinzu.

# A kernel the Kernels page proposed as the default.
boot-proposal-offer = Der Menüeintrag des Kernels { $kernel } kann Standard werden. Vorgemerkt wird er erst mit „Als Standard vormerken“.
boot-proposal-same = Der Kernel { $kernel } ist bereits der Standardeintrag.
boot-proposal-remember = GRUB startet den zuletzt gewählten Eintrag. Schalte „Letzten Eintrag merken“ aus, um den Kernel { $kernel } zum Standard zu machen.
boot-proposal-none = Das Bootmenü hat keinen Eintrag für den Kernel { $kernel }. Wähle den Eintrag selbst oder erstelle das Menü neu, indem du eine beliebige Änderung auf dieser Seite übernimmst.
boot-proposal-loading = Der Menüeintrag des Kernels { $kernel } wird gesucht…
boot-proposal-queue = Als Standard vormerken
boot-proposal-keep = Nicht ändern
boot-proposal-close = Schließen

boot-kernel = Kernel-Parameter
boot-parameters = Parameter für den Kernel
boot-parameters-detail = Jeder Parameter ist ein Wort wie nowatchdog oder loglevel=3. Sie werden in GRUB_CMDLINE_LINUX_DEFAULT geschrieben.
boot-parameters-none = Keine Parameter.
boot-parameters-custom = Die Kernel-Befehlszeile enthält Anführungszeichen oder andere Zeichen, die diese Seite nicht bearbeiten kann: { $line } Bearbeite /etc/default/grub stattdessen auf der Seite „Konfigurationsdateien“.
boot-parameter-new = Neuer Kernel-Parameter
boot-parameter-hint = Neuer Parameter
boot-parameter-add = Hinzufügen
boot-parameter-remove = { $parameter } entfernen
boot-flag-quiet = Die meisten Startmeldungen ausblenden (quiet)
boot-flag-quiet-detail = Ist das aus, zeigt der Kernel beim Systemstart alle Meldungen.
boot-flag-splash = Splash-Screen anzeigen (splash)
boot-flag-splash-detail = Zeigt beim Start eine grafische Animation statt Text. Sie stammt von Plymouth, das bei Bedarf zur Installation vorgemerkt wird.
boot-flag-mitigations-off = CPU-Schutzmaßnahmen ausschalten (mitigations=off)
boot-flag-mitigations-off-detail = Schaltet die Schutzmaßnahmen gegen CPU-Sicherheitslücken ab. Das bringt mehr Leistung, aber weniger Sicherheit.
boot-flag-sysrq = Magische SysRq-Taste einschalten (sysrq_always_enabled=1)
boot-flag-sysrq-detail = Hält die Notfall-Tastenkombinationen des Kernels dauerhaft aktiv.

boot-look = Aussehen
boot-plymouth = Theme des Splash-Screens
boot-plymouth-detail = Die Animation, die Plymouth beim Systemstart zeigt. Beim Übernehmen wird die Initramfs neu erstellt (das kleine Startsystem, das der Kernel zuerst lädt). Das dauert eine Weile.
boot-plymouth-queued = Plymouth ist zur Installation vorgemerkt. Wähle ein Theme, sobald es installiert ist.
boot-plymouth-missing = Plymouth ist nicht installiert, daher gibt es keine Themes zur Auswahl.
boot-plymouth-none = Nicht verfügbar
boot-background = GRUB-Hintergrundbild
boot-background-detail = Ein PNG-, JPEG- oder TGA-Bild hinter dem Bootmenü. Ein GRUB-Theme hat seinen eigenen Hintergrund.
boot-background-from-theme = Vom GRUB-Theme gesetzt (GRUB_MENU_PICTURE). Wähle ein Bild, um es zu ersetzen.
boot-background-title = Hintergrundbild wählen
boot-background-filter = Bilder
boot-theme = GRUB-Theme
boot-theme-detail = Die Datei theme.txt eines GRUB-Themes, meist unter /usr/share/grub/themes.
boot-theme-title = GRUB-Theme wählen
boot-theme-filter = GRUB-Theme
boot-file-none = Keines
boot-file-choose = Wählen…
boot-file-choose-name = { $name } wählen
boot-file-clear = Keines verwenden
