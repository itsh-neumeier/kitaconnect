# Interner Vertrag: Aktuelle Mitgliedschaft lesen v1

Stand: 21.10.2026. Teil von KC-006F / #67. Grundlage: v5.9, ADR-014/019 und Dienstmatrix.

## Umfang und Datenhoheit

organization-config speichert die Zuordnung einer globalen Identitätsreferenz zu einer Einrichtung.
Die Referenz beweist keine Anmeldung. Sie verwendet den bereits festgelegten ganzzahligen Kennungsbereich.
Es gibt keinen Fremdschlüssel und keinen Tabellenzugriff zum Identitätsdienst.
Eine Mitgliedschaft gewährt allein keine Fachrechte und bestimmt allein keine MFA-Anforderung.

Dieser Teil ergänzt ausschließlich den internen Lesebestand und dessen Datenbanknachweise.
Positive Zuordnungen werden nur im isolierten Test über den gekennzeichneten Test-Betriebszugang angelegt.
Eine öffentliche Anlage, Übertragung, Wiederaufnahme oder Rechtevergabe ist nicht enthalten.
Die entsprechende fachliche und sicherheitsbezogene Entscheidung bleibt vor einer solchen Operation erforderlich.

## Bestand

Eine Zuordnung gehört genau zu einer Identitätsreferenz und einer vorhandenen Einrichtung.
Sie enthält eine positive Versionsnummer und den Zustand aktiv oder widerrufen.
Ein neuer Datensatz beginnt mit Version 1 und ohne aktive Freigabe als Voreinstellung.
Kennungen bleiben unverändert. Jede Zustandsänderung erhöht die Version genau um eins.
Der Laufzeitrolle werden ausschließlich Leserechte gewährt. FORCE RLS bleibt ohne erlaubende Policy geschlossen.
Es gibt keine zusätzliche Identitätskopie, E-Mail-Adresse, Kinderakte oder Rollenliste.

## Aktuelle Abfrage

Die Operation erhält eine geprüfte bestehende Verbindung, eine Identitätsreferenz und eine genaue Einrichtung.
Sie MUSS außerhalb einer aufrufenden Transaktion laufen. Sie verändert keine Rolle und öffnet keine Verbindung.
Eine einzelne Datenbankabfrage liest aktive Zuordnung und vorhandene Einrichtungseinstellungen aus demselben aktuellen Datenbankstand.
Sie liefert höchstens einen begrenzten Datenstand mit beiden Versionsnummern und den drei Fristen.
Fehlende, widerrufene, fremde oder unsichtbare Zuordnungen und fehlende Einstellungen ergeben keinen Treffer.
Ungültige gespeicherte Werte und Datenbankfehler bleiben Fehler.
Der Adapter legt keine Einstellungen oder Mitgliedschaften beim Lesen an.

Jede spätere geschützte Prüfung MUSS erneut lesen. Ein früherer Datenstand erteilt keine wiederverwendbare Berechtigung.
Nach bestätigtem Widerruf ergibt die nächste Abfrage keinen Treffer, auch auf derselben offenen Verbindung.
Eine schon begonnene Abfrage kann den Stand vor einem konkurrierenden Widerruf lesen.
Die lokale Abfrage behauptet keine dienstübergreifende Atomarität.

## Grenzen der Integration

Dieser Teil implementiert weder Netzwerktransport noch mTLS oder Diensttokens.
Er ersetzt keinen vollständigen OrganizationAuthority-Adapter des Identitätsdienstes.
Dieser benötigt zusätzlich die aktuelle MFA-/Zweckentscheidung aus dem zuständigen Rechtebereich.
Die konkrete MFA-Zuordnung wurde am 21.10.2026 ausdrücklich bestätigt und wird in einem eigenen Rechte-/MFA-Vertrag umgesetzt.
Unbekannte notwendige Rechte-/Policy-Daten müssen Zugriff verweigern.
Eine echte geschützte Dienstintegration und KC-004C bleiben eigene Nachweise.

## Nachweise

Echte PostgreSQL-/SQLx-Tests prüfen genaue Zuordnung, fremde Einrichtung/Identität, fehlende Einstellungen,
Versionsgrenzen, unbekannte Referenzen, Widerruf auf offener Verbindung und tatsächliche Konkurrenz.
Sie prüfen Migration, Rücknahme, RLS für Laufzeit/Eigentümer, fehlende Schreibrechte und Neustart.
Alte Migrationen und ältere getrennte Testdatenbanken bleiben unverändert.
