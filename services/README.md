# Rust-Dienste

`bootstrap` prüft nur die Rust-Werkzeugkette. Das Programm öffnet keinen Netzwerkport.
Die geplanten Dienstgrenzen stehen im [P0-Plan](../docs/15_P0_Initialisierung.md).
Axum, Tokio und SQLx sind durch v5.9 vorgegeben. Ihre Einbindung erfolgt je Arbeitspaket
nach den bestätigten Architekturentscheidungen und dem ausdrücklichen Phasenauftrag.
Die [Dienstmatrix](../docs/architecture/KC-002_Dienstgrenzen.md) benennt Datenhoheit und Aufrufzwecke.
P0 enthält keine Migrationen.

P1 ist am 18.10.2026 ausdrücklich beauftragt. `organization-config` enthält das interne Organisationsmodell
und den SQLx-Speicheradapter als Rust-Bibliothek. Es startet keinen Netzwerkdienst.
Die Bibliothek verwendet eine bereits geprüfte Datenbankverbindung; Benutzerrechte und Laufzeitfreigabe folgen gesondert.
Der [P1-Plan](../docs/22_P1_Start_KC-004.md) und der
[Modellvertrag](../contracts/organization-config/model-v1.md) grenzen die weiteren Arbeitspakete ab.
Der [Adaptervertrag](../contracts/organization-config/adapter-v1.md) beschreibt die Speicherung und ihre verbleibenden Grenzen.
Eine gesonderte [Wartungs-API](../contracts/organization-config/schema-provisioning-v1.md) provisioniert leere Einrichtungsschemas atomar.
Sie benötigt eine privilegierte Betriebsverbindung und vergibt keine Anwendungsrechte.

KC-005B ergänzt ausschließlich [technische Dienstisolationstests](../contracts/organization-config/service-isolation-test-v1.md).
Die Testartefakte unter `database/tests/` gehören nicht zu den ausgelieferten Migrationen oder zur Anwendungsprovisionierung.
