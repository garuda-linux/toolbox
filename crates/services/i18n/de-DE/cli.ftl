## The command line's one-line refusals. They say what the page's own
## texts say, in one lower-case line without a final full stop.

cli-switch-unknown = der Katalog hat keinen solchen Schalter
cli-switch-unmet = dieser Schalter braucht einen anderen, der aus ist, schalte zuerst diesen ein

cli-repair-not-offered = die Startreparatur gibt es nur im Live-System
cli-repair-unavailable = die Datenträger konnten nicht gelesen werden, darum lässt sich nichts reparieren
cli-repair-unknown-system = dieses System steht nicht mehr in der Liste, siehe `bootrepair show`
cli-repair-in-use = das laufende System nutzt diesen Datenträger, darum wird nichts darauf geschrieben
cli-repair-unknown-target = wähle zuerst, wohin der Bootloader kommt
cli-repair-unknown-layout = wähle zuerst, wo das System liegt
cli-repair-no-passphrase = gib die Passphrase der verschlüsselten Partition ein

cli-account-unavailable = die Benutzerkonten dieses Systems konnten nicht gelesen werden
cli-account-unknown-user = es gibt keinen Benutzer mit diesem Namen
cli-account-unknown-group = eine der Gruppen gibt es nicht, oder sie wird gerade entfernt
cli-account-root = das ist ein Root-Konto, Garuda Toolbox ändert es nicht
cli-account-current-user = mit diesem Konto bist du angemeldet, es kann sich nicht selbst sperren oder entfernen
cli-account-system-account = das ist ein Systemkonto, nur Konten von Personen lassen sich entfernen
cli-account-no-admin-group = dieses System hat keine wheel-Gruppe, der ein neues Konto beitreten kann, daher kann es kein Administrator sein
cli-account-outside-user-range = dieses Konto hat eine Benutzer-ID außerhalb von 1000 bis 60000, darum bleibt es, wie es ist
cli-account-last-admin = das ist das letzte Mitglied der Gruppen wheel und sudo, ohne eines könnte niemand dieses System verwalten
cli-account-last-admin-lock = das ist das letzte Mitglied der Gruppen wheel und sudo, mit gesperrtem Passwort könnte niemand dieses System verwalten
cli-account-lock-state-unknown = dieses Konto lässt sich nicht sperren, solange unbekannt ist, welche Admins gesperrt sind
cli-account-primary-group = das ist die primäre Gruppe eines Benutzers, darum bleibt sie
cli-account-system-group = das ist eine Systemgruppe, installierte Programme verlassen sich auf sie
cli-account-user-exists = einen Benutzer mit diesem Namen gibt es schon
cli-account-group-exists = eine Gruppe mit diesem Namen gibt es schon
cli-account-shell-not-listed = diese Shell steht nicht in /etc/shells
cli-account-uid-in-use = ein anderes Konto hat diese Benutzer-ID schon
cli-account-uid-out-of-range = Konten von Personen bekommen eine Benutzer-ID von 1000 bis 60000, Systemkonten eine unter 1000
cli-account-home-taken = ein anderes Konto hat diesen Home-Ordner schon
cli-membership = `{ $name }`: { $why }

cli-locale-system-locale = das ist die Systemsprache, wähle eine andere Systemsprache, bevor du sie abschaltest
cli-locale-last-locale = mindestens eine Locale muss eingeschaltet bleiben
cli-locale-not-enabled = diese Locale ist nicht eingeschaltet, schalte sie zuerst ein

cli-driver-other-file = eine andere Treiber-Einstellungsdatei in modprobe.d sperrt diesen Treiber, Garuda Toolbox ändert nur seine eigene Datei
cli-driver-unavailable = die Treiber dieses Systems konnten nicht gelesen werden
cli-driver-not-listed = das ist kein Netzwerktreiber dieses Systems, siehe `network drivers list`
cli-driver-loaded = dieser Treiber ist schon geladen
cli-driver-not-loaded = dieser Treiber ist nicht geladen

cli-boot-needs-confirm =
    `{ $value }` öffnet beim Start eine Root-Shell oder schaltet eine Sicherheitsschicht ab
    gib --force an, wenn du das willst
cli-boot-unavailable = dieses System hat kein /etc/default/grub, darum gibt es nichts zu ändern
cli-boot-custom-command-line = die Kernel-Befehlszeile wird hier nicht geändert, bearbeite stattdessen /etc/default/grub
cli-boot-too-many-parameters = die Kernel-Befehlszeile ist voll
cli-boot-unknown-theme = `{ $value }` ist kein installiertes Plymouth-Theme
cli-boot-no-kernel-cmdline = dieses System hat kein /etc/kernel/cmdline, seine Starteinträge tragen ihre eigenen Parameter
cli-boot-custom-kernel-cmdline = die Kernel-Befehlszeile wird hier nicht geändert, bearbeite stattdessen /etc/kernel/cmdline
cli-boot-protected-parameter = `{ $value }` bleibt, ohne ihn findet das System kein Root-Dateisystem zum Starten
cli-boot-empty = es wurde nichts eingegeben
cli-boot-too-long = `{ $value }` ist zu lang
cli-boot-invalid-character = `{ $value }` enthält ein Zeichen, das hier nicht erlaubt ist, etwa ein Leerzeichen, ein Anführungszeichen, einen Backslash oder ein Dollarzeichen
cli-boot-invalid-component = `{ $value }` ist kein vollständiger Pfad zu einer Datei
cli-boot-wrong-file-type = `{ $value }` ist keine Datei der richtigen Art
cli-boot-not-usable = `{ $value }` lässt sich hier nicht verwenden
