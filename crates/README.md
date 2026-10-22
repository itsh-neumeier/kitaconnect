# Gemeinsame Rust-Bibliotheken

Hier dürfen künftig technische Bibliotheken für Telemetrie und Vertragsformate liegen.
`service-transport` ergänzt in P1 eine technische mTLS-Grundlage nach ADR-044.
Sie prüft tatsächliche Zertifikate und enthält keine Datenbank- oder Fachrechte.
Fachmodelle und Datenbankzugriffe gehören zum zuständigen Dienst.
Gemeinsame Bibliotheken dürfen keine dienstübergreifenden Tabellenzugriffe erlauben.
P0 legt noch keine solche Bibliothek an.
