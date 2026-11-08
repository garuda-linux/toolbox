
page-users-subtitle = Benutzer anlegen, Passwörter setzen, Konten sperren und Gruppen ändern.
users-refresh = Neu einlesen
users-tabs = Reiter für Benutzer und Gruppen
users-tab-users = Benutzer
users-tab-groups = Gruppen
users-dismiss = Ausblenden
users-cancel = Abbrechen
users-list-separator = {", "}

users-search = Benutzer suchen
users-search-hint = Benutzer suchen…
users-search-detail = Durchsucht Benutzernamen, UIDs, vollständige Namen, Home-Verzeichnisse, Shells und Gruppen.
users-add = Benutzer hinzufügen…
users-regular-only = Nur reguläre Benutzer
users-regular-only-detail = Blendet System- und Dienstkonten aus.
users-sort = Benutzer sortieren
users-order-name = Benutzername
users-order-id = UID
users-order-full-name = Vollständiger Name
users-order-home = Home-Verzeichnis
users-order-shell = Login-Shell
# $shown is how many the filter and search let through, $count all accounts.
users-count = { $shown } von { $count } Konten
users-reading = Konten werden gelesen…
users-unavailable = Benutzerkonten sind nicht verfügbar
users-unavailable-detail = /etc/passwd und /etc/group ließen sich nicht lesen.
users-none-regular = Keine regulären Benutzer
users-none-regular-detail = Schalte den Filter ab, um auch Systemkonten zu sehen.
users-show-all = Alle Konten anzeigen
users-no-match = Kein Benutzer passt zu „{ $search }“
users-clear-search = Suche leeren
users-list = Benutzer
users-keys = Mit den Pfeiltasten wählst du ein Konto, mit der Eingabetaste bearbeitest du es. Die Menütaste öffnet weitere Aktionen.

users-tag-you = Du
users-tag-locked = Gesperrt
users-tag-system = System
users-tag-queued = { $count ->
        [one] 1 Änderung vorgemerkt
       *[other] { $count } Änderungen vorgemerkt
    }
users-detail = UID { $uid }, Home-Verzeichnis { $home }, Shell { $shell }
users-groups-line = Gruppen: { $groups }
users-not-set = nicht gesetzt
users-edit = Bearbeiten…
users-password = Passwort setzen…
# What a screen reader hears for the row button above. $user is the account.
users-edit-named = { $user } bearbeiten…
users-more = Weitere Aktionen für { $user }
users-lock = Sperren vormerken
users-unlock = Entsperren vormerken
users-unqueue-lock = Nicht sperren
users-unqueue-unlock = Nicht entsperren
users-lock-unknown = Sperrstatus unbekannt
users-groups = Gruppen…
users-remove = Entfernen…
users-keep = Nicht entfernen
users-root = Root-Konten werden hier nicht geändert.

users-locks-loading = Gesperrte Konten werden abgefragt…
users-locks-dismissed = Es ist nicht bekannt, welche Konten gesperrt sind. Die Passwortabfrage wurde geschlossen.
users-locks-denied = Es ist nicht bekannt, welche Konten gesperrt sind. Dein Konto darf das nicht abfragen.
users-locks-no-agent = Es ist nicht bekannt, welche Konten gesperrt sind. In dieser Sitzung läuft kein polkit-Agent (das Programm, das nach deinem Passwort fragt, wenn eine App mehr Rechte braucht), daher erschien keine Passwortabfrage.
users-locks-helper-missing = Es ist nicht bekannt, welche Konten gesperrt sind. Das Hilfsprogramm von Garuda Toolbox ist nicht installiert. Installiere Garuda Toolbox neu.
users-locks-unknown = Es ist nicht bekannt, welche Konten gesperrt sind. Das Hilfsprogramm von Garuda Toolbox konnte es nicht auslesen.
users-locks-again = „Neu einlesen“ fragt erneut ab.

users-run-password-running = Das Passwort von { $user } wird gesetzt…
users-run-password-done = Das Passwort von { $user } wurde gesetzt.
users-run-password-failed = Das Passwort von { $user } ließ sich nicht setzen. Den Grund zeigt die Konsole.
users-run-new-account-running = Das Konto { $user } wird angelegt…
users-run-new-account-done = Das Konto { $user } wurde angelegt.
users-run-new-account-failed = Das Konto { $user } ließ sich nicht anlegen. Den Grund zeigt die Konsole.
users-show-console = Konsole anzeigen

users-refused-root = { $name } ist ein Root-Konto. Garuda Toolbox ändert es nicht.
users-refused-current-user = Mit { $name } bist du gerade angemeldet. Das eigene Konto lässt sich nicht sperren oder entfernen.
users-refused-system-account = { $name } ist ein Systemkonto. Entfernen lassen sich nur Konten von Personen.
users-refused-last-admin = { $name } ist das letzte Mitglied der Gruppen wheel und sudo. Ohne dieses Konto könnte niemand mehr dieses System verwalten.
users-refused-last-admin-lock = { $name } ist das letzte Mitglied der Gruppen wheel und sudo. Mit gesperrtem Passwort könnte niemand mehr dieses System verwalten.
users-refused-lock-state-unknown = { $name } lässt sich nicht sperren, solange unklar ist, welche Admins gesperrt sind.
users-refused-primary-group = { $name } ist die primäre Gruppe eines Benutzers und bleibt deshalb bestehen.
users-refused-system-group = { $name } ist eine Systemgruppe. Installierte Programme brauchen sie.
users-refused-user-exists = Einen Benutzer { $name } gibt es schon.
users-refused-group-exists = Eine Gruppe { $name } gibt es schon.
users-refused-unknown-user = Es gibt keinen Benutzer { $name }.
users-refused-unknown-group = Eine der Gruppen existiert nicht oder wird gerade entfernt.
users-refused-shell-not-listed = Diese Shell steht nicht in /etc/shells.
users-refused-uid-invalid = Die UID muss eine ganze Zahl von 1 bis 60000 sein.
users-refused-uid-in-use = Diese UID ist bereits vergeben.
users-refused-uid-out-of-range = Reguläre Konten bekommen eine UID von 1000 bis 60000, Systemkonten eine unter 1000.
users-refused-home-taken = Dieses Home-Verzeichnis gehört bereits einem anderen Konto.
users-refused-unavailable = Die Benutzerkonten dieses Systems ließen sich nicht lesen.

users-name-empty = Gib einen Namen ein.
users-name-too-long = Der Name ist zu lang.
users-name-invalid-start = Der Name muss mit einem Kleinbuchstaben oder einem Unterstrich beginnen.
users-name-invalid-character = Nur Kleinbuchstaben, Ziffern, Bindestriche und Unterstriche sind erlaubt.
users-name-invalid = Der Name ist ungültig.

users-home-empty = Gib ein Verzeichnis ein.
users-home-too-long = Das Home-Verzeichnis ist zu lang.
users-home-not-absolute = Das Home-Verzeichnis muss ein absoluter Pfad sein, etwa /home/ada.
users-home-invalid-character = Das Home-Verzeichnis darf keine Doppelpunkte, Zeilenumbrüche oder anderen Steuerzeichen enthalten.
users-home-outer-blank = Das Home-Verzeichnis darf nicht mit einem Leerzeichen beginnen oder enden.
users-home-invalid-component = Das Home-Verzeichnis darf kein „..“, keinen einzelnen Punkt, keine doppelten Schrägstriche und keinen abschließenden Schrägstrich enthalten.

users-full-name-too-long = Der vollständige Name ist zu lang.
users-full-name-invalid-character = Der vollständige Name darf keine Doppelpunkte, Kommas, Gleichheitszeichen, Anführungszeichen oder Zeilenumbrüche enthalten.
users-full-name-leading-dash = Der vollständige Name darf nicht mit einem Bindestrich beginnen.
users-full-name-outer-blank = Der vollständige Name darf nicht mit einem Leerzeichen beginnen oder enden.

users-password-empty = Gib ein Passwort ein.
users-password-too-long = Das Passwort ist zu lang.
users-password-control-character = Das Passwort darf keine Zeilenumbrüche oder anderen Steuerzeichen enthalten.
users-password-mismatch = Die Passwörter stimmen nicht überein.

users-add-title = Benutzer hinzufügen
users-field-name = Benutzername
users-field-name-hint = Kleinbuchstaben, Ziffern, Bindestriche und Unterstriche, höchstens 32 Zeichen.
users-field-full-name = Vollständiger Name
users-field-shell = Login-Shell
users-field-shell-hint = Zur Wahl stehen nur Shells aus /etc/shells.
users-shell-default = Systemvorgabe
users-field-home = Home-Verzeichnis
users-field-home-hint = Optional, als absoluter Pfad. Leer lassen für das übliche Verzeichnis unter /home.
users-field-home-edit-hint = Ein absoluter Pfad, etwa /home/ada.
users-field-uid = UID
users-field-uid-hint = Optional. 1000 bis 60000 für eine Person, unter 1000 für ein Systemkonto.
users-no-home = Kein Home-Verzeichnis
users-no-home-detail = Das Konto bekommt kein eigenes Verzeichnis.
users-system = Systemkonto
users-system-detail = Für Dienste statt Personen. Das Konto bekommt eine UID unter 1000 und ist bei „Nur reguläre Benutzer“ ausgeblendet.
users-field-password = Passwort
users-field-repeat = Passwort wiederholen
users-admin = Administrator
# $group is the admin group, wheel.
users-admin-detail = Darf mit dem eigenen Passwort Software installieren und das System ändern. Der Benutzer wird dazu Mitglied der Gruppe { $group }.
users-other-groups = Zusätzliche Gruppen
users-groups-search = Gruppen suchen
users-groups-search-hint = Gruppen suchen…
users-add-note = Der Benutzer wird sofort angelegt. Dafür wird dein Passwort abgefragt.
users-add-confirm = Benutzer jetzt hinzufügen
users-add-more = Weitere Optionen
# Beside the greyed out "Add user now": the first thing it waits for.
users-add-why-no-name = Gib einen Benutzernamen ein.
users-add-why-bad-name = Der Benutzername ist ungültig. Der Grund steht darunter.
users-add-why-no-password = Gib ein Passwort ein.
users-add-why-passwords-differ = Die Passwörter stimmen nicht überein.
users-add-why-fields = Ein Feld oben enthält einen Fehler.
users-add-why-busy = Wartet auf laufende Änderungen.

users-edit-title = { $user } bearbeiten
users-move-files = Dateien ins neue Verzeichnis verschieben
users-move-yes = Beim Übernehmen werden alle Dateien von { $old } nach { $new } verschoben. Das neue Verzeichnis darf noch nicht existieren und { $user } muss abgemeldet sein. Auf einem anderen Datenträger wird alles kopiert, was lange dauern kann.
users-move-no = Nur der Kontoeintrag ändert sich, Dateien werden nicht verschoben. Alles bleibt in { $old }. { $new } muss bereits existieren, sonst hat { $user } bei der nächsten Anmeldung kein Home-Verzeichnis. { $user } muss beim Übernehmen abgemeldet sein.
users-home-current-user = Das Home-Verzeichnis deines eigenen Kontos lässt sich nicht ändern, solange darin Programme laufen.
users-edit-note = Die Änderungen bleiben vorgemerkt, bis du sie übernimmst.
users-queue-changes = Änderungen vormerken

users-password-title = Passwort von { $user } setzen
users-password-note = Das Passwort wird sofort gesetzt. Dafür wird dein eigenes Passwort abgefragt. Diese Seite meldet, wenn es fertig ist. Das Passwort wird weder in den vorgemerkten Änderungen noch in einer Datei von Garuda Toolbox gespeichert.
users-password-confirm = Passwort jetzt setzen

users-groups-title = Gruppen von { $user }
users-groups-note = Die Änderungen bleiben vorgemerkt, bis du sie übernimmst. Sie gelten ab der nächsten Anmeldung des Benutzers.
users-choice-detail = GID { $gid }, Mitglieder: { $members }
users-choice-primary = primäre Gruppe, GID { $gid }, Mitglieder: { $members }
users-choice-new = neue Gruppe

users-remove-title = Benutzer { $user } entfernen?
users-remove-goes = Das Entfernen bleibt vorgemerkt, bis du es übernimmst. Entfernt wird:
users-remove-account = Das Konto { $user } und sein Passwort
users-remove-group = Die Gruppe { $user }, wenn der Benutzer ihr einziges Mitglied ist
users-remove-memberships = Seine Mitgliedschaften in: { $groups }
users-remove-home-goes = Das Home-Verzeichnis { $home } mit allen Dateien darin und das Mail-Postfach
users-remove-home-stays = Das Home-Verzeichnis { $home } bleibt auf dem Datenträger.
users-remove-delete-home = Auch das Home-Verzeichnis löschen
users-remove-confirm = Entfernen vormerken

groups-search = Gruppen suchen
groups-search-hint = Gruppen suchen…
groups-search-detail = Durchsucht Gruppennamen, GIDs und Mitglieder.
groups-add = Gruppe hinzufügen…
groups-sort = Gruppen sortieren
groups-order-name = Gruppenname
groups-order-id = GID
groups-order-members = Meiste Mitglieder
groups-list = Gruppen
groups-reading = Gruppen werden gelesen…
groups-unavailable = Gruppen sind nicht verfügbar
groups-no-match = Keine Gruppe passt zu „{ $search }“
groups-detail = GID { $gid }, Mitglieder: { $members }
groups-detail-empty = GID { $gid }, keine Mitglieder
groups-detail-new = Noch nicht angelegt
groups-tag-new = Neu
groups-tag-system = System
groups-tag-removed = Entfernen vorgemerkt
groups-remove = { $group } entfernen…
groups-unqueue-new = { $group } nicht anlegen
groups-unqueue-removal = { $group } behalten
groups-remove-title = Gruppe { $group } entfernen?
groups-remove-detail = Alle Mitglieder verlieren ihre Mitgliedschaft. Dateien der Gruppe behalten ihre GID. Das Entfernen bleibt vorgemerkt, bis du es übernimmst.
groups-remove-confirm = Entfernen vormerken

groups-add-title = Gruppe hinzufügen
groups-field-name = Gruppenname
groups-add-note = Die Gruppe bleibt vorgemerkt, bis du sie übernimmst. Benutzer kannst du ihr schon vorher zuweisen.
groups-add-confirm = Gruppe vormerken
