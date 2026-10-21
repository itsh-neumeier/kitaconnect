# ADR-043: Aktionsgebundene MFA-Zuordnung

- Datum: 21.10.2026
- Status: Technische Konkretisierung der bestätigten Interviewentscheidung vor Code
- Umfang: KC-006F, Grundlage für KC-008

## Entscheidung

Der [Aktionsvertrag](../../contracts/identity-access/operation-assurance-v1.md) bildet die bestätigten Aktionen und Datenklassen auf die vorhandene aktuelle Sitzungsprüfung ab.
Verwaltung und die bestätigten sensiblen Daten benötigen `RecentMultiFactor`.
Veröffentlichte Portfolios und einfache Kontaktdaten behalten ihre getrennten Objekt-/Feldrechte.
Unbekannte Einstufungen verweigern Zugriff.
Die Einstufung stammt aus der tatsächlichen Aktion im zuständigen Dienst.

## Begründung

Ein frei übergebener schwächerer Prüfmodus darf die bestätigte Zuordnung nicht umgehen.
Eine kleine typisierte Zuordnung vermeidet unterschiedliche MFA-Regeln in späteren Routen.
Die bestehende aktuelle Prüfung erhält Faktorvarianten, Fristen, Mitgliedschaft und Widerruf.
Es entsteht keine neue fachliche Berechtigung.

## Grenzen

Die Projektleitung hat die Zuordnung ausdrücklich mit „Diese Zuordnung bestätigen“ bestätigt.
Der Vertrag ändert weder Basisrollen noch die Schutzklassen der Spezifikation.
Unzugeordnete sensible Daten erhalten keine pauschale Erlaubnis.
Die Anwendung muss tatsächliche Objekt-/Feldrechte weiterhin aktuell prüfen.
Eine reine MFA-Prüfung erfüllt KC-008 nicht vollständig.
Es gibt keine Migration und keine produktive Rechtefreigabe.
