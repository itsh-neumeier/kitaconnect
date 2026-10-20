# ADR-032 – WebAuthn-Vertrag und offene Freigaben

Datum: 20.10.2026. Status: bestätigtes Testprofil und technische Konkretisierung für KC-006D / #65.
Grundlage: Spezifikation v5.9, ADR-019 und ADR-029 bis ADR-031.

## Geprüfter Istzustand

PR #70 ist offen und zur Durchsicht bereit. Abschlusskopf e003eb055f4137ee66184152ce660e96d838be44 besteht alle fünf Pflichtprüfungen für Push und PR.
Main und bestehende Migrationen bleiben unverändert. Es gibt noch keinen WebAuthn-Bestand oder öffentlichen Anmeldeweg.
KC-006D darf keine Passwortpflicht als Voraussetzung hinzufügen.

## Technischer Vorschlag

Für die Serverprüfung wird webauthn-rs 0.5.5 vorgeschlagen.
Die Paketmetadaten nennen Rust 1.88 und MPL-2.0. Der Projektcompiler Rust 1.99 erfüllt die Mindestversion.
Die Bibliothek prüft das Protokoll und die Signaturen. KitaConnect entwickelt dafür keine eigene Kryptografie.
Der Passkey-Ablauf dieser Version verlangt Benutzerverifikation und erlaubt synchronisierte Authentikatoren.
Diese Eigenschaften sind durch „Alle drei Vorschläge bestätigen“ ausdrücklich bestätigt.

Für synthetische Protokolltests kommt webauthn-authenticator-rs 0.5.5 mit dem Testmerkmal `softpasskey` in Betracht.
Es gilt ebenfalls MPL-2.0 und Rust 1.88. Das Merkmal verwendet OpenSSL und weitere Testabhängigkeiten.
Der Software-Authentikator bleibt eine reine Entwicklungsabhängigkeit. Private Testschlüssel dürfen nur im Arbeitsspeicher entstehen.
Eine nachgestellte Benutzerverifikation ist ein Protokolltest und kein Nachweis einer tatsächlichen Geräte-PIN oder Biometrie.

Die Auswahl ist noch kein vollständiger Abhängigkeitsnachweis.
Vor Übernahme sind der konkrete Lockfile-Diff, native OpenSSL-Voraussetzungen und die vollständige RustSec-Prüfung zu prüfen.
Die juristische Lizenzprüfung bleibt offen. Aus den Paketmetadaten wird keine proprietäre Vertriebsfreigabe abgeleitet.

## Abgegrenzter Abhängigkeitsnachweis

Ein ignoriertes lokales Prüfprojekt hat ausschließlich die beiden Kandidaten aufgelöst; es wurde kein Anwendungscode gebaut.
Serverabhängigkeit: `webauthn-rs =0.5.5`, ohne Standardmerkmale, mit `danger-allow-state-serialisation`.
Dieses Merkmal wäre ausschließlich für den geschützten serverseitigen Datenbankzustand vorgesehen.
Clientseitige Zustandsspeicherung bleibt verboten. Merkmale zur Aufhebung der Benutzerverifikation oder zum Zugriff auf Credential-Interna sind nicht vorgesehen.
Testabhängigkeit: `webauthn-authenticator-rs =0.5.5`, ohne Standardmerkmale, mit `softpasskey`.

Am 20.10.2026 hat cargo-audit 0.22.2 diesen getrennten Graphen mit 143 Paketen einschließlich des lokalen Prüfpakets ohne Befund geprüft.
Befehl: `cargo-audit audit --file .local/webauthn-dependency-review/Cargo.lock --db .local/advisory-db --deny warnings`.
Die frisch aktualisierte RustSec-Datenbank enthält 1.345 Hinweise; Stand b19b2c83c2493961ccb12f9fc4699a00fb2c6c44.
Der Nachweis gilt für diesen Prüfgraphen. Er ersetzt weder den späteren Projekt-Lockfile-Nachweis noch eine allgemeine Sicherheitsprüfung.
Das Build-Skript des Test-Authentikators verlangt OpenSSL ab Version 3.0.
Ein erfolgreicher nativer Build wurde in diesem Vorbereitungsschritt nicht behauptet.

## Vertragsvorbereitung

Der [Vertragsentwurf](../../contracts/identity-access/webauthn-v1.md) beschreibt Zuständigkeiten, aktuelle Prüfung und Negativfälle.
Nach ausdrücklicher Entscheidung werden Vertrag und dieser ADR vor Implementierung aktualisiert und versioniert.

## Bestätigte Entscheidungen und verbleibende Grenzen

| Punkt | Angefragter Vorschlag | Folge ohne Antwort |
|---|---|---|
| RP-ID und Ursprung | Eine RP-ID und ein ausdrücklicher HTTPS-Ursprung je Installation; keine Platzhalter oder zusätzlichen Subdomains. Alternativ zunächst ausschließlich localhost im isolierten Test. | Keine eingebauten produktiven Domains und keine davon abhängige Implementierung. |
| Benutzerverifikation | Für den eigenständigen Passkey-Weg immer erforderlich. Keine Festlegung der MFA-Pflicht anderer Wege. | Kein aktivierter Passkey-Ablauf und keine Übernahme des Bibliotheksverhaltens als Projektentscheidung. |
| Synchronisierte Passkeys | Zulassen, wenn Benutzerverifikation erfüllt ist. | Keine Entscheidung für synchronisierte oder ausschließlich gerätegebundene Schlüssel vorwegnehmen. |

Die Projektleitung hat alle drei Vorschläge im Projektchat ausdrücklich bestätigt.
Damit gelten zunächst ausschließlich localhost im isolierten Test, erforderliche Benutzerverifikation und erlaubte synchronisierte Passkeys.
Die produktive Domain bleibt ausdrücklich offen. Die Tabelle dokumentiert die ursprünglich angefragten Vorschläge und ihre damaligen Sperren.
Der Adapter erhält keine produktive Ausweichkonfiguration und legt keine allgemeine MFA-Pflicht fest.
Die bisherigen offenen MFA-, Wiederherstellungs- und produktiven Datenbankfragen bleiben bestehen.

Der [aktualisierte Vertrag](../../contracts/identity-access/webauthn-v1.md) wird vor Code versioniert.
Eine neue Migration ergänzt Kontobindung, Credentials und ausschließlich serverseitige Vorgänge.
Vorgänge werden vor der Antwortprüfung dauerhaft verbraucht. Eine gemeinsame Kontosperre und Versionskennung verhindern Erfolg aus veraltetem Credential-Bestand.
Der Adapter begrenzt Antwortgröße, Credential-Zahl, gespeicherte Vorgänge und laufende Signaturprüfungen.
Konkrete produktive Grenzwerte und HTTP-Anbindung bleiben ausgenommen.

## Alternativen und Grenzen

Eine eigene Signatur- oder CBOR-Implementierung erhöht den Prüfbedarf und wird nicht vorgesehen.
Clientseitig gespeicherter Prüfzustand kann Wiederverwendung ermöglichen und wird ausgeschlossen.
Eine reine Fortschrittsmarkierung ohne echte positive und negative Protokolltests erfüllt #65 nicht.
Dieser Entwurf erfüllt deshalb noch keine Implementierungs- oder Gesamtabnahme.

## Quellen

- [webauthn-rs 0.5.5: Paketmetadaten](https://crates.io/crates/webauthn-rs/0.5.5)
- [Server-API und Quelltext](https://docs.rs/webauthn-rs/0.5.5/src/webauthn_rs/lib.rs.html)
- [Serverseitiger Zustand und Credential-Aktualisierung](https://docs.rs/webauthn-rs/0.5.5/src/webauthn_rs/interface.rs.html)
- [Test-Authentikator 0.5.5](https://crates.io/crates/webauthn-authenticator-rs/0.5.5)
- [W3C WebAuthn Level 3](https://www.w3.org/TR/webauthn-3/)
