# Warum ein Setup ein AUR-Paket auf dem alten paru-Weg baut. $reason ist
# einer der aur-route-Texte unten.
aur-old-path = Dieses Setup nutzt den alten paru-Weg: { $reason }

aur-route-no-account = Das Konto hat keinen Eintrag in /etc/passwd, wie bei systemd-homed oder LDAP-Konten
aur-route-linked = Der Home-Ordner oder ~/.cache ist ein Link, oder ein Ordner, den der Bau braucht, fehlt
aur-route-shared = Ein Ordner, den der Bau nutzt, kann von anderen Benutzern beschrieben werden
aur-route-through-sudo = Die Kommandozeile fragt das Passwort über sudo ab, dabei ist nicht erkennbar, wer gefragt hat
aur-route-other = Der Bau-Ordner lässt sich nicht prüfen

# Warum einem queued AUR-Bau noch keine Bau-Reihenfolge vorliegt. $name ist das Paket.
aur-order-unread = { $name } hat noch keine Bau-Reihenfolge: paru konnte nicht gefragt werden.
aur-order-route = { $name } hat keine Bau-Reihenfolge: Der Bau-Ordner lässt sich nicht nutzen.
aur-order-missing = { $name } hat keine Bau-Reihenfolge: Das AUR hat das Paket nicht.
aur-order-not-named = { $name } hat keine Bau-Reihenfolge: paru baut es nicht aus dem AUR.
