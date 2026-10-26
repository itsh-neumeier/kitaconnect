# ADR-059: Dauerhafte Quellenversionen der Anmeldung

- Datum: 26.10.2026
- Status: Technische Konkretisierung vor Migration und Code
- Umfang: KC-006F; keine neue fachliche Berechtigung

## Entscheidung

Der [Bindungsvertrag](../../contracts/identity-access/source-bindings-v1.md) erweitert den vorhandenen Kontext ohne Versionsverlust.
Migration 0011 bindet vollständige Quellenversionen atomar an Vorgänge und Sitzungen.
Mitgliedschaft bleibt an ihre ursprüngliche Version gebunden. Einstellungen und Rollen erhalten getrennte monotone Beobachtungen.
Gleiche Versionen verlangen gleiche zugeordnete Rohwerte. Die rohe Rollen-MFA bleibt von der erhaltenen Sitzungs-MFA getrennt.
Neue Bestände verlangen vollständige Bindungen spätestens beim Commit. Fehlende Bestände verweigern die Prüfung.
Der tatsächliche Quellenleser verwendet diesen vollständigen Portkontext.

## Begründung

Eine einzige Einstellungsnummer erfasst weder Mitgliedschaftsentzug noch Rollenänderung.
Eine zwischenzeitlich entzogene Mitgliedschaft darf durch eine spätere erneute Zuordnung keine alte Sitzung wieder erlauben.
Die feste Mitgliedschaftsversion schützt davor auch dann, wenn während des Entzugs keine Anfrage erfolgte.
Getrennte Rohwerte erkennen widersprüchliche Ergebnisse derselben Version.
Eigene Tabellen ermöglichen einen sicheren Übergang, ohne unbekannte alte Versionen zu erfinden oder alte Migrationen umzuschreiben.

## Folgen und Grenzen

Vorherige unvollständige Bestände erlauben nach Migration 0011 keine Anmeldung oder Sitzung mehr.
Die bestehenden früheren isolierten Testbestände ohne diese Migration bleiben weiterhin als Teilnachweise ausführbar.
Sicherheitskritische positive Laufzeitprüfungen verwenden die neue vollständige Quelle und den migrierten Bestand.
Lokale Transaktionen schließen fremde Datenbestände nicht atomar ein. Jede Folgeprüfung fragt erneut aktuell ab.
Produktive RLS, Gateway, Browser und Gesamtabnahmen bleiben gesondert.
