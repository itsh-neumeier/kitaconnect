# ADR-033 – TOTP und Wiederherstellung vorbereiten

Datum: 21.10.2026. Status: bestätigte Regeln und technische Konkretisierung für KC-006E / #66.
Grundlage: Spezifikation v5.9, ADR-019 und ADR-029 bis ADR-032.

## Istzustand

PR #71 ist offen und zur Durchsicht bereit. Kopf 74c5e1731be0db55f1ef8698d7119f8fcd5cce7c besteht alle fünf Pflichtprüfungen für Push und PR.
Die WebAuthn-Bestätigung betrifft localhost, Benutzerverifikation und synchronisierte Passkeys.
Sie bestätigt keine allgemeinen MFA- oder Wiederherstellungsregeln.
Es gibt noch keine TOTP-Registrierung, Wiederherstellung oder produktive Anmeldung.

## Bibliotheksprüfung

Für zeitbasierte Einmalpasswörter wird totp-rs 6.0.0 vorgeschlagen.
Die Paketmetadaten nennen MIT und Rust 1.88. Projektcompiler Rust 1.99 erfüllt diese Mindestversion.
Vorgesehene Merkmale: keine Standardmerkmale, nur `zeroize`.
Die Bibliothek erhält die geprüfte Serverzeit ausdrücklich. Ihre Methoden zum Lesen der lokalen Systemzeit werden nicht benötigt.
Das Merkmal zur Serialisierung von Geheimnissen wird nicht aktiviert.
Die Prüfung liefert den passenden Zeitschritt zurück. Die Bibliothek verhindert dessen erneute Nutzung nicht selbst.
KitaConnect muss deshalb den zuletzt verbrauchten Schritt atomar und dauerhaft speichern.

Für die verschlüsselte Speicherung des erforderlichen gemeinsamen TOTP-Geheimnisses kommt RustCrypto aes-gcm 0.11.1 in Betracht.
Die Paketmetadaten nennen Apache-2.0 oder MIT und Rust 1.85.
Vorgesehene Merkmale: keine Standardmerkmale, nur `aes`, `alloc` und `zeroize`.
Die Verschlüsselung muss Integrität und Bindung an Identität, Faktor und Schlüsselversion prüfen.
Eine Nonce muss je Schlüssel eindeutig sein. Ein späterer Schlüsselanbieter muss Wiederanlauf, Nutzungslimit und Rotation ausdrücklich absichern.
Es wird kein eigenes Verschlüsselungs- oder TOTP-Verfahren entwickelt.

Ein getrenntes, ignoriertes Prüfprojekt hat diese beiden Kandidaten aufgelöst, ohne Anwendungscode zu bauen.
Am 21.10.2026 besteht dessen Graph mit 32 Paketen einschließlich Prüfpaket cargo-audit 0.22.2 mit `--deny warnings` ohne Befund.
RustSec-Datenbank: e6f94f3760d5e6c54c1b05f204002263bc025524, 1.348 Hinweise.
Befehl: `cargo-audit audit --file .local/mfa-dependency-review/Cargo.lock --db .local/advisory-db --deny warnings`.
Dieser Nachweis gilt nicht für den späteren Projekt-Lockfile, einen nativen Build oder eine rechtliche Vertriebsfreigabe.
Der Projekt-Lockfile und die bestehenden Migrationen bleiben in diesem Vorbereitungsschritt unverändert.

## Vertragsvorschlag

Der [Vertragsentwurf](../../contracts/identity-access/mfa-recovery-v1.md) trennt Einrichtung, Prüfung, Faktorwechsel und Wiederherstellung.
Die technische Prüfung liefert einen begrenzten Nachweis; sie erzeugt keine Sitzung oder Fachberechtigung.
Wiederherstellungscodes werden zufällig erzeugt. Im Bestand darf ausschließlich deren Ableitung liegen.
TOTP-Geheimnisse benötigen eine geschützte, entschlüsselbare Speicherung; eine Passwort-Hashfunktion genügt dafür nicht.
Registrierung, Verbrauch, Widerruf und gegebenenfalls Benachrichtigungsauftrag benötigen atomare Datenbankvorgänge.
Eine neue Migration darf bestehende Migrationen 0001 bis 0003 nicht verändern.
Erzwungene RLS bleibt ohne erlaubende Laufzeit-Policy bestehen.

Produktiver Schlüsselanbieter, Rotation und Betriebszugang bleiben vor produktiver Nutzung gesondert zu entscheiden.
Die synthetische Testumgebung darf nur eigene zufällige Testschlüssel im Arbeitsspeicher verwenden.
Ein fehlender Schlüssel darf weder unverschlüsselte Speicherung noch einen Ersatzschlüssel auslösen.

## Offene Sicherheitsentscheidungen

| Thema | Angefragter Vorschlag | Sperre ohne Antwort |
|---|---|---|
| MFA-Zuordnung | Pflicht bei Verwaltungsrechten und besonders sensiblen Daten; sonst zunächst optional. Zusätzliche Prüfung sensibler Aktionen. Konkrete Zuordnung über KC-008. | Keine abhängige Anmeldefreigabe oder automatische Rollenzuordnung. |
| Wiederherstellung | Einmalige offline aufbewahrte Codes. E-Mail allein setzt MFA nicht zurück. Sitzungen widerrufen und erforderliche Faktoren neu einrichten. Ohne Nachweis bleibt ein Ersatzverfahren gesperrt. | Keine davon abhängige Wiederherstellungsimplementierung. |
| Benachrichtigung | Sicherheitsprotokoll und atomar vorgemerkte Nachricht an bereits bestätigten Kontakt bei Einrichtung, Wechsel, Widerruf und Wiederherstellung. Nachricht ohne Geheimnisse. | Keine ungefragte Auswahl des Benachrichtigungsverfahrens. |

Die ersten beiden Vorschläge wurden im Projektchat gemeinsam erneut zur Entscheidung gestellt.
Die Benachrichtigungsregel wurde als zusätzliche, für das Akzeptanzkriterium erforderliche Frage gestellt.
Die Projektleitung hat anschließend ausdrücklich „Ja, alle genannten Regeln bestätigen“ geantwortet.
Damit sind MFA-Zuordnung, einmalige Offline-Wiederherstellung mit Sitzungswiderruf und Faktorneueinrichtung sowie Sicherheitsprotokoll und Benachrichtigungsauftrag bestätigt.
Die Tabelle dokumentiert die angefragten Regeln und die zuvor bestehenden Sperren.
Die konkrete Rechtezuordnung bleibt KC-008 vorbehalten. Sitzungsgrenzen und produktiver Schlüsselbetrieb bleiben offen.
Der aktualisierte Vertrag wird vor Implementierung versioniert.

Der Adapter verwendet die bestehende dauerhafte Versuchsreservierung aus KC-006C; diese technische Voraussetzung wird im Backlog ergänzt.
AES-GCM verwendet im Test einen frischen zufälligen RAM-Schlüssel und vorab dauerhaft reservierte eindeutige Zähler-Nonces.
Der Schlüssel kann weder importiert noch exportiert werden. Produktiver Schlüsselanbieter und Rotation werden dadurch nicht freigegeben.
Eine neue Migration ergänzt MFA-Bestand, verschlüsselte Faktoren, abgeleitete Offline-Codes, begrenzte Wiederherstellungsnachweise und atomare Ereignisaufträge.
Vorbereitete Faktoren ersetzen den aktiven Faktor erst nach gültiger Bestätigung. Alte Zeitschritte werden nicht erneut akzeptiert.
Wiederherstellung sperrt normale Anmeldeabschlüsse bis zur Neueinrichtung; die spätere Gateway-Anbindung muss diese aktuelle Sperre auch bei anderen Nachweisarten prüfen.

## Quellen

- [RFC 6238: TOTP und einmalige Nutzung](https://www.rfc-editor.org/rfc/rfc6238.html)
- [totp-rs 6.0.0: Paketmetadaten](https://crates.io/crates/totp-rs/6.0.0)
- [totp-rs 6.0.0: Prüfung mit Zeitschritt](https://docs.rs/totp-rs/6.0.0/totp_rs/struct.Totp.html)
- [aes-gcm 0.11.1: Bibliothek und Sicherheitsgrenzen](https://docs.rs/aes-gcm/0.11.1/aes_gcm/)
- [OWASP: MFA, Faktorwechsel und Wiederherstellung](https://cheatsheetseries.owasp.org/cheatsheets/Multifactor_Authentication_Cheat_Sheet.html)
