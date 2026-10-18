# ADR-023 – Interne Organisationsmodellkomponente

Datum: 18.10.2026. Status: technische Festlegung im ausdrücklich beauftragten P1-Umfang.
Bezug: KC-004A (#50), ADR-014 und Dienstmatrix.

## Entscheidung

Das erste P1-Arbeitspaket ergänzt eine Rust-Bibliothek unter `services/organization-config/`.
Sie enthält das dienstbezogene Modell für Träger und Einrichtungen.
Der [interne Vertrag](../../contracts/organization-config/model-v1.md) beschreibt Operationen und Fehler vor der Umsetzung.
Es gibt keine zusätzliche externe Abhängigkeit, Datenbankmigration oder Netzwerkfreigabe.

Kennungen verwenden getrennte Rust-Typen mit positiven 128-Bit-Werten.
Dies verhindert die versehentliche Vertauschung von Träger- und Einrichtungskennungen innerhalb der Bibliothek.
Die Darstellung legt weder Erzeugungsverfahren noch Datenbank- oder HTTP-Format fest.
Einrichtungszuordnungen können über diese Modell-API nach Anlage nicht geändert werden.
Eine spätere Transferfunktion braucht einen gesonderten fachlichen und technischen Vertrag.

## Alternativen und Folgen

Eine gemeinsame Fachbibliothek unter `crates/` würde die Datenhoheit des zuständigen Dienstes verwischen.
Ein sofortiger HTTP-/Datenbankdienst würde Identitäts-, Policy- und Migrationsnachweise vorwegnehmen.
Die gewählte interne Komponente erlaubt kleine, prüfbare Schritte ohne neue Berechtigungsannahmen.

Die Bibliothek ist kein Sicherheitsnachweis für den späteren Dienst.
KC-004B prüft Persistenz; KC-004C prüft aktuelle Rechte und fremde Trägerzuordnung an der Dienstgrenze.
Die ursprünglichen Akzeptanzkriterien von KC-004 bleiben bestehen.
Die P0-Abnahme und der gesonderte P1-Auftrag stehen im Entscheidungsprotokoll.
