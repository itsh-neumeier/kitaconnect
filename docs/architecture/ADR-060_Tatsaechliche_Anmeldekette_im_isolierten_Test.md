# ADR-060: Tatsächliche Anmeldekette im isolierten Test

- Datum: 26.10.2026
- Status: Technische Prüfentscheidung vor Testcode
- Umfang: KC-006F; keine Änderung fachlicher Regeln

## Entscheidung

Der [Prüfvertrag](../../contracts/identity-access/login-chain-test-v1.md) verbindet vorhandene Laufzeitpfade ohne positives Anmeldeport-Testdoppel.
Faktor-Bibliotheken und lokale Transaktionen verwenden den tatsächlichen Quellenleser mit mTLS und getrennten Datenbanken.
Migration 0011 gilt zusätzlich im tatsächlichen HTTP-Testbestand. Eigene synthetische Identitäten und Einrichtungen verhindern Konflikte mit früheren Teilprüfungen.
Vorherige synthetische Porttests bleiben als getrennte Nachweise erhalten.

## Begründung und Grenzen

Getrennte positive Faktor-, Transport- und Speichertests beweisen noch nicht ihre gemeinsame korrekte Verbindung.
Der Endnachweis prüft deshalb aktuelle Quellen bei Beginn, Abschluss und Sitzungsprüfung sowie Rollen-, Mitgliedschafts- und Dienstentzug.
Alte positive Datenstände dürfen Ausfall oder Entzug nicht ersetzen.
Es gibt keine neue Fachfreigabe, Anwendungsmigration oder erlaubende Laufzeitpolicy.
Gateway, Benutzeranmeldung über HTTP, Browser und ursprüngliche Gesamtabnahmen bleiben ausdrücklich offen.
