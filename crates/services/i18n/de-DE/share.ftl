# Sharing a text as an encrypted paste: the Diagnostics report or the
# Console's output. $subject is "report" or "output".

share-link-copied = Link in die Zwischenablage kopiert.
share-delete-link-copied = Löschlink in die Zwischenablage kopiert.

# The question before an upload.
share-upload-question = { $subject ->
    [report] Diesen Bericht hochladen?
   *[output] Diese Ausgabe hochladen?
}
share-upload-where = Der folgende Text wird an bin.garudalinux.org gesendet. Er wird bereits auf diesem Computer verschlüsselt, sodass der Server ihn nicht lesen kann. Jeder mit dem Link kann ihn aber lesen. Der Server löscht ihn nach einem Monat.
share-upload-check = Prüfe ihn auf weitere Angaben, die du nicht teilen möchtest. Die Suche markiert jeden Treffer.
share-upload-replaced-none = Soweit die Toolbox es erkennt, nennt der Text weder dich noch diesen Computer.
share-upload-replaced =
    { $count ->
        [one] Ein Wert, der dich oder diesen Computer nennt, wurde ersetzt, etwa der Benutzername, der Rechnername, eine Seriennummer oder eine öffentliche Adresse.
       *[other] { $count } Werte, die dich oder diesen Computer nennen, wurden ersetzt, etwa der Benutzername, der Rechnername, Seriennummern und öffentliche Adressen.
    }
share-upload-search = Text durchsuchen
share-upload-search-hint = Text durchsuchen
share-upload-search-clear = Suche leeren
share-upload-search-next = Nächster Treffer
share-upload-search-count = { $current } von { $count }
share-upload-text = Hochzuladender Text
share-upload-cancel = Abbrechen
share-upload-confirm = Hochladen

# The upload and its link.
share-uploading = { $subject ->
    [report] Der Bericht wird hochgeladen…
   *[output] Die Ausgabe wird hochgeladen…
}
share-uploaded = { $subject ->
    [report] Hochgeladen. Jeder mit diesem Link kann den Bericht lesen:
   *[output] Hochgeladen. Jeder mit diesem Link kann die Ausgabe lesen:
}
share-link = { $subject ->
    [report] Link zum hochgeladenen Bericht
   *[output] Link zur hochgeladenen Ausgabe
}
share-copy-link = Link kopieren
# The one row a place with little room shows after an upload.
share-uploaded-short = Hochgeladen.
share-link-actions = Weitere Aktionen für den Link
share-open-link = Im Browser öffnen
share-delete-link = { $subject ->
    [report] Löschlink: Beim Öffnen wird der Bericht vom Server entfernt.
   *[output] Löschlink: Beim Öffnen wird die Ausgabe vom Server entfernt.
}
share-copy-delete-link = Löschlink kopieren
share-delete-now = Jetzt löschen…
share-hide = Ausblenden
share-delete-question = { $subject ->
    [report] Den Bericht vom Server löschen?
   *[output] Die Ausgabe vom Server löschen?
}
share-delete-detail = Danach kann niemand mehr den Link öffnen, auch du nicht. Der Löschlink wird in deinem Browser geöffnet.
share-delete-cancel = Abbrechen
share-delete-confirm = Löschen

# Why an upload failed. $detail is the system's or the server's words.
share-upload-network = Das Hochladen ist fehlgeschlagen. bin.garudalinux.org ist nicht erreichbar. Prüfe die Netzwerkverbindung.
share-upload-refused = { $subject ->
    [report] Das Hochladen ist fehlgeschlagen. Der Paste-Dienst hat den Bericht abgelehnt, zum Beispiel weil er zu groß ist.
   *[output] Das Hochladen ist fehlgeschlagen. Der Paste-Dienst hat die Ausgabe abgelehnt, zum Beispiel weil sie zu groß ist.
}
share-upload-bad-answer = Das Hochladen ist fehlgeschlagen. Garuda Toolbox versteht die Antwort des Paste-Dienstes nicht.
share-upload-seal = { $subject ->
    [report] Das Hochladen ist fehlgeschlagen. Der Bericht konnte nicht verschlüsselt werden.
   *[output] Das Hochladen ist fehlgeschlagen. Die Ausgabe konnte nicht verschlüsselt werden.
}
share-upload-stopped = Das Hochladen ist wegen eines Fehlers in Garuda Toolbox fehlgeschlagen. Melde ihn im Forum oder im Issue-Tracker.
share-upload-detail = { $problem } ({ $detail })
share-upload-dismiss = Ausblenden
