# Organisationsadapter v1 – interner Rust-/SQLx-Vertrag

Stand: 19.10.2026. Issue: KC-004B / #51.
Grundlage: [Modellvertrag](model-v1.md) und [Speichervertrag](storage-v1.md).

## Operationen

Der Adapter übernimmt eine bestehende SQLx-PostgreSQL-Verbindung.
Er erzeugt weder Verbindungen noch Zugangsdaten und führt keine Migration oder Rollenänderung aus.
Die aufrufende Komponente muss Identität, aktuelle Rechte und Transport vor Verwendung prüfen.
Der Adapter gewährt keine Berechtigung. Die RLS-Sperre aus Migration 0001 bleibt unverändert.

| Operation | Ergebnis |
|---|---|
| Träger registrieren | Eine Kennung speichern; doppelte Kennung als Modellfehler melden |
| Einrichtung registrieren | Einrichtung mit optionalem Träger speichern; doppelte Kennung oder unbekannten Träger melden |
| Trägerbestand prüfen | Wahrheitswert innerhalb des für die Verbindung sichtbaren Bestands |
| Einrichtung nach Kennung lesen | Typisierter Modellwert oder kein sichtbarer Treffer |

Ein fehlender Treffer ist kein Beleg, dass die Kennung installationsweit nicht existiert.
Jede Änderung verwendet genau eine SQL-Anweisung. Mehrere Änderungen können durch eine vom Aufrufer geführte Transaktion zusammengefasst werden.
Der Adapter bestätigt oder verwirft diese Transaktion nicht selbst.
Er bietet weder Transfer noch Änderung oder Löschung bestehender Zuordnungen an.

## Kennungen und Fehler

Alle Tabellen- und Funktionsnamen sind vollständig qualifiziert.
Kennungen werden als gebundene Dezimalparameter übergeben und innerhalb von SQL nach Numeric gewandelt.
Es gibt keine SQL-Zusammensetzung aus Kennungen oder Schemanamen.
Beim Lesen werden exakte Ganzzahlwerte ohne Nachkommastellen nach u128 und dem passenden Kennungstyp gewandelt.
Ungültige gespeicherte Kennungen erzeugen einen Fehler, keinen Ersatzwert oder abgeschnittenen Wert.

Bekannte Primär- und Fremdschlüsselverletzungen werden auf die entsprechenden Modellfehler abgebildet.
Andere Datenbankfehler bleiben Fehler. Eine Rechteverletzung wird nicht als Erfolg oder fehlender Treffer ausgegeben.
Die öffentliche Fehlerbeschreibung des Adapters enthält keine Datenbankdetails.
Interne Fehlerquellen dürfen nicht ungeprüft in spätere HTTP-Antworten übernommen werden.

## Prüfungen

Der vorhandene isolierte PostgreSQL-Testläufer ruft die Rust-Integrationstests ausdrücklich auf.
Ohne Testumgebung bleiben sie im normalen Cargo-Testlauf als ignoriert sichtbar; der verpflichtende Datenbankprüfschritt führt sie aus.
Eine fehlende Testumgebung beim ausdrücklichen Aufruf muss fehlschlagen.
Die Verbindung läuft über einen kurzlebigen Unix-Socket des eigenen Containers, ohne TCP-Port.

Die Tests prüfen positive Anlage und Rücklesen, beide Kennungsgrenzen, Null-Träger,
doppelte Kennungen, unbekannte Träger, Rücknahme einer mehrteiligen Transaktion und Persistenz nach Server-Neustart.
Eine gesonderte Test-Loginrolle erhält ausschließlich die Laufzeitrechte.
Unter dieser Identität müssen Lesen durch RLS gefiltert und Schreiben verweigert werden.
Nur die Test-Betriebsidentität darf positive Testdaten trotz RLS anlegen.
Dies ist keine produktive Rechteentscheidung und ersetzt KC-004C nicht.
