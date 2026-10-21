# ADR-041: Interner Mitgliedschaftsbestand

- Datum: 21.10.2026
- Status: Technische Konkretisierung für KC-006F
- Grundlage: v5.9, ADR-014/019, Dienstmatrix und [Lesevertrag](../../contracts/organization-config/membership-read-v1.md)

## Entscheidung

Der Organisationsdienst hält Identitätsreferenz, Einrichtung, Version und aktiven Zustand.
Die Identitätsreferenz bleibt eine dienstfremde Kennungsreferenz ohne Datenbank-Fremdschlüssel.
Eine einzige aktuelle Abfrage verbindet aktive Mitgliedschaft und Einrichtungseinstellungen innerhalb des Organisationsdienstes.
Der Laufzeitbestand bleibt durch FORCE RLS geschlossen und erhält ausschließlich Leserechte.
Positive Anlagen bleiben ausdrückliche synthetische Test-Betriebsdaten.

## Begründung und Folgen

Die Datenhoheit bleibt gemäß Dienstmatrix im Organisationsdienst.
Eine direkte Abfrage fremder Tabellen im Identitätsdienst wurde verworfen.
Eine Rollenliste oder eine alte positive Momentaufnahme wurde als Berechtigungsnachweis verworfen.
Bestätigter Widerruf muss beim nächsten Lesen wirken. Die Abfrage vermeidet einen positiven Zwischenspeicher.

## Grenzen

Die Mitgliedschaft erteilt keine Fachrechte und entscheidet allein nicht über MFA.
Die konkrete Zuordnung für Verwaltung und sensible Daten wurde am 21.10.2026 ausdrücklich bestätigt.
Ihre Umsetzung folgt in einem eigenen Rechte-/MFA-Vertrag; Mitgliedschaft allein liefert diese Entscheidung nicht.
Es entsteht keine öffentliche Mitgliedschaftsverwaltung, Übertragungsregel oder erlaubende Laufzeitpolicy.
Netzwerk, Dienstidentität, aktueller Rechteumfang und KC-004C benötigen gesonderte Nachweise.
