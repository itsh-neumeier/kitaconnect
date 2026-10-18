# ADR-016 – Gesonderter Personalbereich je Träger

Datum: 18.10.2026. Status: von der Projektleitung entschieden; Umsetzung ausstehend.
Bezug: ADR-009, ADR-014, O-001-HR und KC-002.

## Grundlage und Entscheidung

Die Spezifikation fordert trägerweite Personalverwaltung und begrenzte Einrichtungsrechte.
Die Projektleitung hat am 18.10.2026 Variante `3A` gewählt.
Quelle: ausdrückliche Antwort `1A, 2A, 3A, 4A` im Projektchat.

Jeder Träger erhält einen gesonderten Personalbereich mit eigener Rechteprüfung.
Einrichtungen erhalten ausschließlich die ausdrücklich erlaubten Daten ihrer eingesetzten Mitarbeitenden.
Trägerlose Einrichtungen behalten einen eigenen Personalbereich.
Ein Mandant bleibt eine Einrichtung. Der Personalbereich macht den Träger nicht zu einem Mandanten mit Vollzugriff.

## Datenhoheit und technische Grenze

`staff-scheduling` bleibt allein zuständig für Personalakten, Verträge, Qualifikationen,
Einsatzzuordnungen, Dienstpläne, Arbeitszeiten und Abwesenheiten des Personals.
Der zentrale Personalbestand eines Trägers liegt in einem eigenen registrierten PostgreSQL-Schema.
RLS bedeutet Row-Level Security und ergänzt die ausdrücklich vergebenen Objektrechte.
Eine trägerlose Einrichtung führt ihren Personalbereich im eigenen Einrichtungsschema.

Personalstamm, Beschäftigungsverträge und trägerweite Qualifikationen gehören zum Personalbereich.
Einsatz-, Schicht- und Zeitdaten besitzen zusätzlich einen ausdrücklichen Einrichtungsbezug.
Sie werden durch den zuständigen Dienst im Einrichtungsschema geführt.
Ein Bezug auf eine Personalakte erteilt keine Leseberechtigung auf deren vollständigen Inhalt.
Die endgültige Tabellen- und Feldzuordnung folgt als geprüfter Dienstvertrag in KC-015.

Nur der zuständige Personaldienst erhält die nötigen Datenbankrechte für beide Bereiche.
Andere Dienste verwenden zweckgebundene Dienstverträge. Sie erhalten keine direkten Tabellenrechte.
Eine Einsatzansicht enthält nur die für die Aufgabe freigegebenen Felder.
Einrichtungskontext allein erlaubt keine Abfrage des gesamten Personalbereichs eines Trägers.

Die Rollen für Laufzeit, Eigentum, Migration und Provisionierung bleiben nach ADR-014 getrennt.
Die Migrationssteuerung behandelt Personal- und Einrichtungsschemas als unterschiedliche registrierte Ziele.
Datensicherung, Export und Wiederherstellung MÜSSEN ihre getrennten Rechte berücksichtigen.

## Einsatzwechsel und Rechteentzug

Ein Einsatz in einer anderen Einrichtung oder bei einem anderen Träger gewährt nur ausdrücklich zugewiesene Rechte.
Das Ende eines Einsatzes entzieht diese Rechte. Es kopiert keine vollständige Personalakte.
Trägerübergreifende Aufgaben benötigen eigene begrenzte Berechtigungen gemäß v5.9.
Die Entscheidung definiert keine neue Rechtsgrundlage und keinen automatischen Personaltransfer.
Änderungshistorie und Aufbewahrung folgen den fachlich und rechtlich bestätigten Regeln.

## Alternative und Folgen

Nicht gewählt wurde eine ausschließlich dezentrale Personalhaltung mit zusammengeführten Einrichtungsansichten.
Der eigene Personalbereich unterstützt die vereinbarte trägerweite Verantwortung.
Er benötigt zusätzliche Provisionierungs-, RLS-, Widerrufs- und Wiederherstellungsprüfungen.
Es gibt keine pauschale Rolle mit Zugriff auf alle Träger.

## Nachweise vor Implementierungsabnahme

- Fremde Einrichtungen und Träger können keine Personalakten lesen oder ändern.
- Eine berechtigte Einrichtung sieht nur erlaubte Felder der ihr zugewiesenen Mitarbeitenden.
- Ein beendeter Einsatz entzieht Rechte auch bei aktiver Sitzung und wiederverwendeter Verbindung.
- Eine trägerlose Einrichtung benötigt keinen erfundenen Träger, um Personal zu verwalten.
- Migration, Export und Wiederherstellung erhalten die Bereichsgrenzen.

Diese Nachweise gehören zu KC-005, KC-008, KC-015 und KC-033.
**Umsetzung in P0:** nur Dokumentation; keine Datenbank und keine Migration angelegt.
