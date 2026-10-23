# ADR-027 – Dienstrechte und RLS im technischen Testaufbau

Datum: 20.10.2026. Status: technische Testentscheidung im Sicherheitsumfang von KC-005B.
Grundlage: ADR-014, ADR-026 und P1-Auftrag. Keine produktive Rechtefreigabe.

## Entscheidung

Der [Testvertrag](../../contracts/organization-config/service-isolation-test-v1.md) wird vor der Implementierung versioniert.
Zwei bereits festgelegte Dienste erhalten synthetische Proben in zwei registrierten Einrichtungsschemas.
Ein fest eingebundenes SQL-Artefakt legt diese Proben ausschließlich im isolierten Testcontainer an.
Getrennte Eigentümer-, Migrations- und Laufzeitrollen machen die Rechte negativ prüfbar.
Die Testpolicy verwendet die angemeldete Datenbankidentität und eine feste Einrichtungskennung.
Sie verwendet keinen frei gesetzten Sitzungswert als Autorität.

## Alternativen und Grenzen

Eine Policy allein auf Basis von `current_setting('kc.facility_id')` wurde verworfen.
Die Laufzeit könnte diesen Wert selbst ändern. Ein solcher Test würde keine Identitätsgrenze nachweisen.
Eine produktive Rollen- oder Tokenarchitektur wird mit diesem Testaufbau nicht festgelegt.
Vor einer produktiven erlaubenden Policy sind Identitätsnachweis, Widerruf und Poolbindung ausdrücklich zu entscheiden.
Bis dahin bleibt die produktive Runtime durch die vorhandene RLS-Sperre eingeschränkt.

Der Test bindet nur ein versioniertes Artefakt ein. Er ist keine allgemeine Migrationssteuerung.
DDL im gemeinsamen Schema bleibt privilegiert und kann nicht durch einen Namenspräfix begrenzt werden.
Die Objektprüfung und der Transaktionsfehler prüfen den Testaufbau; sie ersetzen keine fachliche Freigabe.
Benutzerrechte bleiben KC-006/KC-008 zugeordnet. KC-005C ergänzt Verbindungs- und Konkurrenztests.

## Quellen

- [PostgreSQL 18: RLS, FORCE RLS und Grenzen](https://www.postgresql.org/docs/18/ddl-rowsecurity.html)
- [PostgreSQL 18: Schemas und Namensauflösung](https://www.postgresql.org/docs/18/ddl-schemas.html)
- [PostgreSQL 18: Funktionsrechte und search_path](https://www.postgresql.org/docs/18/sql-createfunction.html)
