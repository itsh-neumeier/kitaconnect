# ADR-031 – Begrenzte Argon2id-Prüfung

Datum: 20.10.2026. Status: technische Konkretisierung für KC-006C im P1-Auftrag.

## Entscheidung

Der [Passwortvertrag](../../contracts/identity-access/password-check-v1.md) wird vor Implementierung versioniert.
RustCrypto argon2 0.6.0 übernimmt die Kryptografie und das PHC-Format für gespeicherte Passwortnachweise.
Die Paketmetadaten nennen Rust 1.85 als Mindestversion und MIT oder Apache-2.0 als Lizenz.
Der Projektcompiler Rust 1.99 ist ausreichend. RustSec bleibt eine verpflichtende Prüfung.
Diese Prüfung ersetzt keine juristische Lizenzfreigabe und keine unabhängige Sicherheitsprüfung.

Das neue Profil verwendet Argon2id v19 mit 65.536 KiB, drei Durchläufen und Parallelitätsgrad eins.
Das einzige vorherige Profil entspricht der OWASP-Untergrenze von 19.456 KiB, zwei Durchläufen und Parallelitätsgrad eins.
Das neue Profil ist keine unveränderte Übernahme eines RFC-9106-Referenzprofils.
Eine feste Positivliste schützt vor rechenintensiven Parametern aus beschädigten Datenbankwerten.
Zwei feste Arbeitsprofile verringern Laufzeitunterschiede negativer Fälle.
Die prozessweite Begrenzung schützt den Arbeitsspeicher auch während eines Aufrufabbruchs.

Eine neue Migration ergänzt Passwortbestand und dauerhafte Versuchszähler.
Atomare Reservierung vor Hasharbeit schützt auch bei mehreren Dienstinstanzen.
Explizite Grenzen vermeiden eine ungefragte fachliche Sperrregel.
Die Laufzeit erhält keine erlaubende RLS-Policy. Ein Passwortergebnis erzeugt keine Sitzung.

## Alternativen und Grenzen

Reine Zähler im Arbeitsspeicher verlieren ihren Stand beim Neustart und begrenzen mehrere Instanzen nicht gemeinsam.
Ungeprüfte PHC-Parameter können unverhältnismäßig viel Speicher belegen.
Frühe Rückgabe nur für unbekannte Konten würde deren Erkennung erleichtern.
Diese Varianten werden verworfen.
Produktive Grenzwerte, Zuordnung von Netzwerkanfragen zu Prüfschlüsseln und Lastmessungen bleiben gesonderte Freigabeschritte.
Die offenen MFA-Regeln und die produktive Datenbankbindung bleiben unverändert offen.

## Quellen

- [RustCrypto argon2 0.6.0](https://docs.rs/argon2/0.6.0/argon2/)
- [Paketmetadaten und Abhängigkeiten](https://crates.io/crates/argon2/0.6.0)
- [OWASP: Passwortspeicherung](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)
- [RFC 9106: Argon2](https://www.rfc-editor.org/rfc/rfc9106.html)
