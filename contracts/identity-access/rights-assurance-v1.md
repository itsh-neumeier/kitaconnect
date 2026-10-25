# Aktuelle Rollen-/MFA-Grundlage v1

Stand: 25.10.2026. Interner Speichervertrag für KC-008A / #98. Kein HTTP-Endpunkt.

## Bestand und Ableitung

Der Identitätsdienst besitzt `role_assurance_scopes`. Jede Zeile bindet genau eine Identität und Einrichtung.
Die Zeile enthält Version, Zustand und den vollständigen dienstlokalen Bestand eingestufter Rollen dieses Geltungsbereichs.
Dieser Bestand ist kein Cache einer zweiten Rollentabelle. Es gibt keine zusätzliche erlaubende Quelle.
Die Rollen enthalten technische Schlüssel und vollständig eingestufte Aktionsklassen.
Die Klassen erteilen keine Berechtigung. Objekt-, Feld-, Zweck- und Einwilligungsregeln bleiben KC-008B vorbehalten.
Es entstehen keine vorgegebenen Rollen oder Zuordnungen für echte Benutzer.

Das Dokumentformat lautet `{"format":1,"roles":[{"key":"synthetic","operations":["basic_contact_data"]}]}`.
Jede Rolle hat mindestens eine Klasse. Zulässige Klassen entsprechen ausschließlich ADR-043:
`role_administration`, `security_settings`, `health_data`, `custody_restrictions`, `personnel_file`,
`internal_development_record`, `published_portfolio` und `basic_contact_data`.
Die ersten sechs Klassen verlangen MFA. Die beiden übrigen Klassen verlangen für sich keine zusätzliche MFA.
Die Ableitung verwendet dieselbe Zuordnung wie die aktuelle Aktionsprüfung.
Die Anmeldeanforderung verlangt MFA, sobald mindestens eine eingestufte Rolle eine der sechs Klassen enthält.
Eine leere vollständige Rollenliste bedeutet ausschließlich: keine zugeordneten Rechte in diesem Bestand.
Sie erteilt keinen Fachzugriff. Ein unbekannter Bestand darf nicht durch diese leere Liste ersetzt werden.

Unbekannte Formate, Klassen und Felder sowie doppelte Schlüssel, Rollen oder Klassen verweigern die Ableitung.
Das Dokument ist höchstens 65.536 UTF-8-Bytes groß. Es enthält höchstens 64 Rollen mit je höchstens acht Klassen.
Technische Rollenschlüssel bestehen aus 1 bis 64 ASCII-Zeichen aus `a-z`, `0-9`, `_` und `-`.
Diese Grenzen schützen den Adapter. Sie sind keine Obergrenze der späteren fachlichen Rollenverwaltung.
Nicht vollständig aufgelöste trägerweite oder installationsweite Zuordnungen dürfen keinen aktiven Einrichtungsbestand erzeugen.
Der Vorlauf legt keine solchen Zuordnungen fest. Ihre Fachprüfung bleibt gesondert.

## Persistenz und Aktualität

Migration 0010 ergänzt ausschließlich diesen eigenen Bestand. Die Identität hat einen eigenen Fremdschlüssel.
Die Einrichtung bleibt eine positive Referenz ohne Fremdschlüssel oder Zugriff auf Organisationstabellen.
Ein vollständiger aktiver Bestand beginnt mit Version 1. Änderungen erhöhen die Version genau um eins.
Identität und Einrichtung bleiben unveränderlich. Löschen ist verboten. Widerruf ist endgültig.
Der Widerruf erhöht ebenfalls die Version. Ein späteres neues Vergabeverfahren benötigt einen gesonderten Vertrag.
Das Dokument wird vollständig in einer Zeile ersetzt. RLS kann keine einzelne Rolle aus einer sichtbaren Zeile entfernen.
Ein unvollständiger Zwischenstand darf nicht aktiv veröffentlicht werden.
Schreibzugriffe bleiben privilegierte, ausdrücklich geprüfte Wartung. Es gibt keine öffentliche Vergabe oder Laufzeitschreibberechtigung.

Der Leser verwendet eine einzige aktuelle SQL-Anweisung im Autocommit-Modus.
Er verweigert fremde offene Transaktionen, damit alte Transaktionssichten keine aktuelle Prüfung ersetzen.
Die Anweisung liest Zustand, Version und das gesamte Dokument für die genau angefragten Kennungen.
Fehlender, verdeckter, widerrufener, ungültiger oder nicht verfügbarer Bestand liefert ausschließlich einen Fehler.
Eine erfolgreiche Antwort enthält genaue Kennungen, aktuelle Version und die abgeleitete MFA-Anforderung.
Es gibt keinen positiven Cache, keinen Standardbestand und keinen gesondert gespeicherten MFA-Schalter.
Aktualität bezeichnet den Anweisungssnapshot. Eine später bestätigte Änderung muss die nächste Abfrage erkennen.
Ein Widerruf nach diesem Snapshot wird durch die nächste aktuelle Sitzungsprüfung erkannt.
Für atomare spätere Anmeldeänderungen bleibt die Einbindung in den Identitätssperrvertrag von KC-006F erforderlich.
Dieser Leser nimmt keine zusätzliche Identitätssperre und ruft keine Organisation zurück.

## Sicherheitsumfang und Nachweise

Die Laufzeitrolle erhält nur SELECT. ENABLE und FORCE RLS gelten ohne erlaubende Policy.
Nur der isolierte Testcontainer darf synthetische erlaubende Policies anlegen.
Fehler und Debug-Ausgaben geben weder Bestandsinhalte noch Datenbankdiagnosen aus.
Es gibt keine neue Paketabhängigkeit, Standardrolle oder fachliche Erlaubnis.
Reine Tests prüfen Format, Grenzen und alle Klassen. Tatsächliche SQLx-Tests prüfen Bindung, Entzug,
Versionswechsel, konkurrierende Transaktionen, Fehler, geschlossene RLS und Persistenz vor/nach Neustart.
Migrationstests prüfen vollständigen Rollback und verbotene Wiederholung.
Der Leser allein erfüllt weder den vollständigen Anmeldeadapter noch die ursprüngliche Rechtegesamtabnahme.
