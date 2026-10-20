# ADR-028 – Nachweise für Poolrückgabe und konkurrierende Wartung

Datum: 20.10.2026. Status: technische Testentscheidung für KC-005C.
Grundlage: ADR-014, ADR-026, ADR-027 und P1-Auftrag.

## Entscheidung

Der [Testvertrag](../../contracts/organization-config/pool-maintenance-test-v1.md) wird vor seiner Implementierung versioniert.
Ein Testpool mit genau einer Verbindung macht verbliebenen Kontext nach Erfolg, Fehler und Abbruch sichtbar.
Die Prozesskennung belegt die Wiederverwendung. Ein Rückgabe-Hook setzt Sitzungsparameter zurück.
Der Test verwendet nur die vorhandene Test-Loginbindung. Eine produktive Poolarchitektur bleibt ausdrücklich offen.

Konkurrierende Wartung wird mit echten Verbindungen und beobachteten PostgreSQL-Sperrwarteschlangen geprüft.
Ein bloßer gleichzeitiger Start ohne beobachtete Überlappung reicht nicht als Nachweis.
Die erste Stufe behält die bestehende globale Wartungssperre bei. Eine spätere feinere Parallelisierung braucht eigene Nachweise.
Testmigrationen und Versionsmarken werden gemeinsam bestätigt oder zurückgenommen.
Eine fehlerhafte Welle stoppt nach dem betroffenen Schema. Frühere bestätigte Schritte bleiben erhalten.

## Alternativen und Grenzen

Neue Verbindungen pro Test würden die Poolrückgabe nicht prüfen. Diese Alternative wird verworfen.
Ein frei gesetzter Mandantenwert bleibt als alleinige Identität ausgeschlossen.
Eine vollständige Datenbankrücksetzung würde fremde bestätigte Änderungen gefährden und wird nicht verwendet.
Die Testmigrationshilfe ist kein produktiver Orchestrator. Sie akzeptiert nur zwei festgelegte technische Versionsschritte.
Produktive Identitätsbindung, aktuelle Benutzerrechte und Wiederherstellung bleiben gesonderte Abnahmegrenzen.

## Quellen

- [SQLx 0.9: Pool-Hooks](https://docs.rs/sqlx/0.9.0/sqlx/pool/struct.PoolOptions.html)
- [SQLx 0.9: Transaktionen und Rücknahme beim Verwerfen](https://docs.rs/sqlx/0.9.0/sqlx/struct.Transaction.html)
- [PostgreSQL 18: Transaktionssperren und pg_locks](https://www.postgresql.org/docs/18/explicit-locking.html)
