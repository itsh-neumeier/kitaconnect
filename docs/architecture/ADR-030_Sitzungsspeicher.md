# ADR-030 – Aktueller Sitzungsbestand und atomarer Widerruf

Datum: 20.10.2026. Status: technische Konkretisierung für KC-006B im P1-Auftrag.
Grundlage: ADR-014, ADR-019 und ADR-029. Keine fachliche Rechtefreigabe.

## Entscheidung

Der [Speichervertrag](../../contracts/identity-access/session-storage-v1.md) wird vor Implementierung versioniert.
Der Dienst erhält ein eigenes restriktives Plattform-Schema mit getrennten Eigentümer-, Migrations- und Laufzeitrollen.
Identitäts- und Sitzungstabellen bleiben durch erzwungene RLS ohne erlaubende Policy für die Anwendung gesperrt.
Die positive Speicherprüfung verwendet nur den gesonderten Test-Betriebszugang.

Die interne Nachweiserzeugung verwendet 32 Bytes aus der Betriebssystem-Zufallsquelle.
Gespeichert wird ausschließlich deren SHA-256-Ableitung. Dies ist keine Passwortspeicherung oder selbst entwickelte Kryptografie.
Direkte Abhängigkeiten werden auf bereits im Lockfile vorhandene Versionen festgelegt:
getrandom 0.4.3, sha2 0.11.1, zeroize 1.9.1 und SQLx 0.9.0.
Tokio 1.54.0 dient den Integrationstests. Die genannten Kryptobibliotheken stehen laut Paketmetadaten unter MIT oder Apache-2.0.
Die vorhandene RustSec-Prüfung bleibt verpflichtend. Diese Festlegung ersetzt keine juristische Lizenzprüfung.

Prüfung, Ablauf und Aktivität sperren denselben aktuellen Datensatz vor der Zeitprüfung.
Die Datenbankzeit wird erst nach der Sperre gelesen. Widerruf konkurriert an derselben Zeile.
Die Modellregeln werden wiederverwendet; abgelaufene oder widerrufene Zustände werden nicht reaktiviert.
Es gibt keinen positiven Ergebniszwischenspeicher.

## Alternativen und Grenzen

Rohe Sitzungsnachweise in der Datenbank würden deren Offenlegung unmittelbar nutzbar machen und werden nicht gespeichert.
Selbstbeschreibende Browsernachweise ohne aktuelle serverseitige Prüfung würden den unmittelbaren Widerruf nicht leisten.
Eine vor der Sperrwartezeit gelesene Zeit könnte einen inzwischen abgelaufenen Datensatz freigeben.
Diese Alternativen werden verworfen.
Die atomare lokale Prüfung ist keine verteilte Fachtransaktion und keine pauschale Benutzerberechtigung.
Produktive Rollenbindung, aktuelle Mitgliedschaft, HTTP-Schutz und die offenen Interviewregeln folgen gesondert.

## Quellen

- [getrandom 0.4.3: Betriebssystemquelle und Fehler](https://docs.rs/getrandom/0.4.3/getrandom/fn.fill.html)
- [sha2 0.11.1: RustCrypto SHA-2](https://docs.rs/sha2/0.11.1/sha2/)
- [zeroize 1.9.1: verwalteter Puffer](https://docs.rs/zeroize/1.9.1/zeroize/struct.Zeroizing.html)
- [PostgreSQL 18: Zeilensperren](https://www.postgresql.org/docs/18/explicit-locking.html)
