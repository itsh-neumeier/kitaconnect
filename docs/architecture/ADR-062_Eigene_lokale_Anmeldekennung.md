# ADR-062: Eigene lokale Anmeldekennung

- Datum: 26.10.2026
- Status: Ausdrücklich bestätigte Kennungswahl; technische Konkretisierung vor Code
- Umfang: KC-006F, interner lesender Zuordnungsbestand

## Entscheidung

Die Projektleitung bestätigt eine eigene eindeutige Anmeldekennung je Installation unabhängig von E-Mail.
Der [Vertrag](../../contracts/identity-access/login-name-v1.md) ordnet diese Kennung ausschließlich im Identitätsdienst einer internen Identität zu.
Exakter Bytevergleich vermeidet ungefragte Alias- oder Zusammenführungsregeln. Der Leser erhält eigene begrenzte Verbindungen ohne positiven Cache.
Migration 0012 ergänzt den geschlossenen Bestand. Das Gateway erhält weder Datenbankrolle noch Zugriff auf fremde Tabellen.

## Begründung und Grenzen

Der vorhandene Anmeldekern arbeitet bereits mit unveränderlichen internen Kennungen. Die neue Auflösung ersetzt keine Faktorprüfung.
Eine E-Mail-Adresse bestimmt weder Identität noch Mitgliedschaft. Die nicht gewählte E-Mail-Alternative wird nicht implementiert.
Die erste interne Darstellung begrenzt Arbeit auf 256 UTF-8-Bytes; endgültige Vergabe- und Oberflächenregeln bleiben gesondert.
Dieser Schritt stellt keinen öffentlichen Kontoauflösungsendpunkt bereit. Unbekannte Konten müssen in der späteren Anmeldung verdeckt bleiben.
Umbenennung, Wiederverwendung und öffentliche Registrierung sind nicht Teil dieses Pakets.
Vorhandene Sitzungen benötigen weiterhin die aktuelle Sitzungs-, Quellen- und Berechtigungsprüfung.
Es entstehen keine erlaubende Laufzeitpolicy, neue externe Paketversion oder produktive Konten.
