# Rust-Dienste

`bootstrap` prüft nur die Rust-Werkzeugkette. Das Programm öffnet keinen Netzwerkport.
Die geplanten Dienstgrenzen stehen im [P0-Plan](../docs/15_P0_Initialisierung.md).
Axum, Tokio und SQLx sind durch v5.9 vorgegeben. Ihre Einbindung folgt nach
der Vertrags- und Datenhoheitsentscheidung in KC-002. P0 enthält keine Migrationen.
