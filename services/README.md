# Rust-Dienste

`bootstrap` prüft nur die Rust-Werkzeugkette. Das Programm öffnet keinen Netzwerkport.
Die geplanten Dienstgrenzen stehen im [P0-Plan](../docs/15_P0_Initialisierung.md).
Axum, Tokio und SQLx sind durch v5.9 vorgegeben. Ihre Einbindung folgt nach
den in KC-002 bestätigten Architekturentscheidungen und einem gesonderten Implementierungsauftrag.
Die [Dienstmatrix](../docs/architecture/KC-002_Dienstgrenzen.md) benennt Datenhoheit und Aufrufzwecke.
P0 enthält keine Migrationen.

P1 ist am 18.10.2026 ausdrücklich beauftragt. `organization-config` enthält zunächst nur
das interne Organisationsmodell als Rust-Bibliothek. Es öffnet keinen Port und speichert keine Daten.
Der [P1-Plan](../docs/22_P1_Start_KC-004.md) und der
[Modellvertrag](../contracts/organization-config/model-v1.md) grenzen die weiteren Arbeitspakete ab.
