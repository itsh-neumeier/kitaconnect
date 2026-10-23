# ADR-025 – SQLx-Adapter für Organisationsreferenzen

Datum: 19.10.2026. Status: technische Festlegung für den nächsten Teil von KC-004B.
Grundlage: ADR-024 und ausdrücklicher P1-Auftrag.

## Entscheidung

Der [Adaptervertrag](../../contracts/organization-config/adapter-v1.md) wird vor der Implementierung versioniert.
Der Adapter bleibt im Dienst `organization-config` und verwendet SQLx 0.9.0.
Die Standardfeatures sind deaktiviert. PostgreSQL, Tokio-Laufzeit und Rustls mit Ring sind ausdrücklich ausgewählt.
Tokio 1.54.0 wird für die Integrationstests mit minimalen Test-/Laufzeitfeatures festgelegt.
Cargo.lock fixiert den vollständigen Abhängigkeitsbestand. Die bestehende RustSec-Prüfung bleibt blockierend.

Kennungen werden ohne zusätzliche Dezimalbibliothek über gebundene Dezimaltexte nach Numeric gewandelt.
Die Bereichsprüfung bleibt in Modell und Datenbank erhalten. Das Rücklesen verwendet die exakte Ganzzahldarstellung.
Der Adapter erhält eine bestehende Verbindung statt einer frei zusammengesetzten Verbindungsadresse.
Er besitzt keine Logik für Rollenwechsel, Migration oder Benutzerrechte.
Eine spätere Laufzeitverbindung muss zusätzlich den geschützten Transport und die aktuellen Rechte nachweisen.

## Alternativen und Grenzen

Eine Übertragung über Gleitkommazahlen würde große Kennungen verfälschen und wird verworfen.
Eine weitere Dezimalbibliothek ist für die eng begrenzte Ganzzahldomäne nicht erforderlich.
Ein automatisch migrierender Anwendungsstart oder Rollenwechsel im Repository widerspricht ADR-014.
Die RLS-Sperre ohne erlaubende Policy bleibt deshalb unverändert bestehen.

Positive Datenbanktests verwenden ausschließlich den isolierten Test-Betriebszugang.
Eine eigene kurzlebige Test-Loginrolle prüft die tatsächliche Laufzeitverweigerung mit demselben Adapter.
Der PostgreSQL-Testcontainer bleibt ohne Netzwerk. Ein temporärer Unix-Socket verbindet den Host-Testprozess mit der Datenbank.
Das lokale Vertrauensprofil des Testcontainers ist keine produktive Authentisierung.

## Versions- und Lizenznachweise

Am 19.10.2026 aus der offiziellen Paketquelle geprüft: SQLx 0.9.0 benötigt Rust 1.94.0,
Tokio 1.54.0 benötigt Rust 1.85. Die Projektwerkzeugkette Rust 1.99.1 erfüllt diese Anforderungen.
SQLx nennt MIT oder Apache-2.0, Tokio nennt MIT. Dies ist keine juristische Veröffentlichungsfreigabe.
Neue Abhängigkeiten müssen vor Abnahme den bestehenden aktuellen RustSec-Abgleich bestehen.

- [SQLx 0.9.0: Laufzeit und TLS](https://docs.rs/sqlx/0.9.0/sqlx/)
- [SQLx: vorbereitete Abfragen](https://docs.rs/sqlx/0.9.0/sqlx/fn.query.html)
- [SQLx: Paketmetadaten](https://crates.io/crates/sqlx/0.9.0)
- [Tokio 1.54.0](https://docs.rs/tokio/1.54.0/tokio/)
- [Tokio: Paketmetadaten](https://crates.io/crates/tokio/1.54.0)
