# KC-006: Kleine Arbeitspakete für die Authentisierung

Stand: 20.10.2026. P1 ist ausdrücklich beauftragt. Grundlage: Spezifikation v5.9.
Gesamtissue: [KC-006 / #6](https://github.com/itsh-neumeier/kitaconnect/issues/6).

## Verbindlicher Umfang

Argon2id, Passkeys über WebAuthn, zeitbasierte Einmalpasswörter (TOTP) und Kontowiederherstellung bleiben verbindlich.
Die gesamte Abnahme benötigt positive und negative Anmeldefälle, sofort wirksamen Sitzungswiderruf und verweigerte Wiederverwendung von Wiederherstellungsnachweisen.
Die Aufteilung streicht keine ursprüngliche Bedingung. KC-006 bleibt bis zu allen Bedingungen offen.
OIDC-/SAML-Föderation bleibt in KC-007. Objekt- und Feldberechtigungen bleiben in KC-008.
ADR-019 gilt für Dienstidentitäten und aktuelle Rechteprüfung weiter.

## Reihenfolge und Abhängigkeiten

| Paket | GitHub-Issue | Ergebnis | Voraussetzung |
|---|---|---|---|
| KC-006A | [#62](https://github.com/itsh-neumeier/kitaconnect/issues/62) | Internes Identitäts-/Sitzungsmodell ohne Anmeldung oder Speicherung | KC-004A/B |
| KC-006B | [#63](https://github.com/itsh-neumeier/kitaconnect/issues/63) | Geschützter Sitzungsbestand, aktuelle Abfrage und atomarer Widerruf | KC-006A |
| KC-006C | [#64](https://github.com/itsh-neumeier/kitaconnect/issues/64) | Argon2id-Passwortprüfung und Anfragemengenbegrenzung | KC-006B |
| KC-006D | [#65](https://github.com/itsh-neumeier/kitaconnect/issues/65) | WebAuthn-Registrierung und Anmeldung mit Negativfällen | KC-006B |
| KC-006E | [#66](https://github.com/itsh-neumeier/kitaconnect/issues/66) | TOTP und atomar einmalige Wiederherstellung | KC-006B; bestätigte MFA-/Wiederherstellungsregeln |
| KC-006F | [#67](https://github.com/itsh-neumeier/kitaconnect/issues/67) | Dienstverträge, Gateway-Anbindung und Gesamtnachweise | KC-006C/D/E |

Nur KC-006A ist in diesem Arbeitsschritt aktiv. Jedes folgende Paket erhält einen begrenzten Vertrag vor Implementierung.
KC-006D setzt kein Passwort voraus. Die drei Nachweisverfahren dürfen nach KC-006B unabhängig vorbereitet werden.
KC-006F prüft aktuelle Authentisierung und Mitgliedschaft. Es gewährt keine pauschalen Objekt- oder Feldrechte.
Eine Mitgliedschaft ist allein keine Fachberechtigung. Die spätere KC-008-Prüfung bleibt erforderlich.

## Sicherheitsumfang von KC-006A

Die neue Rust-Bibliothek enthält nur interne Kennungen sowie Zeit- und Zustandsregeln.
Der [Modellvertrag](../contracts/identity-access/session-model-v1.md) und [ADR-029](architecture/ADR-029_Identitaets_und_Sitzungsmodell.md) begrenzen die Implementierung.
Eine Datensatzkennung ist kein Browsernachweis. Ein Modellzustand ist keine aktuelle Authentisierungsentscheidung.
Es werden keine produktiven Konten, Schlüssel, Tokens, Cookies oder Datenbankrollen angelegt.
Es gibt keine Login-API, Passwortprüfung oder selbst entwickelte Kryptografie.

## Offene Sicherheitsentscheidungen

Die [Entscheidungsliste](ai-memory/DECISIONS_PENDING.md) nennt konkrete Vorschläge und den jeweiligen Antwortstand.
Offen sind MFA-Zuordnung, Browser-Sitzungsgrenzen und Wiederherstellungsablauf.
Eine ausbleibende Antwort ist keine Freigabe. Abhängige Laufzeitfunktionen bleiben bis zur Entscheidung gesperrt.
KC-006A kann unabhängig davon mit ausdrücklich übergebenen Grenzen und synthetischen Testwerten umgesetzt werden.
Die bereits angefragte produktive Datenbankidentitäts-/Poolbindung bleibt offen und wird nicht erneut als entschieden behandelt.

## Nachweise und Fortsetzung

Die bestehende Workspace-CI soll Formatierung, Clippy, Tests und Build der neuen Bibliothek erfassen.
Rust-Tests müssen Fristgrenzen, unwirksame Verlängerungsversuche, Widerruf und Zeitfehler prüfen.
Sie sind kein Nachweis einer Anmeldung oder sofortigen verteilten Widerrufsprüfung.
KC-006B muss danach Speicherung und atomare aktuelle Prüfung gegen echte PostgreSQL-Verbindungen nachweisen.

## Ausgeführter erster Modellnachweis

Stand 5abb46e6345e4d0e5417ead35e9f96d12deea947: fünf Pflichtprüfungen des ersten vollständigen CI-Laufs erfolgreich.
[Build, Clippy und Modelltests](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39411531255),
[Secret- und Abhängigkeitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39411531380).
Elf Modelltests und der negative Kennungstypvertrag sind erfolgreich. Neue externe Abhängigkeiten waren nicht erforderlich.
[PR #68](https://github.com/itsh-neumeier/kitaconnect/pull/68) führt den Prüfstand des folgenden Dokumentationscheckpoints nach.
KC-006A bleibt bis Integration offen. Nächster Schritt ist KC-006B / #63; die Gesamtabnahme von KC-006 bleibt offen.

## Aktueller Sitzungsbestand aus KC-006B

[PR #69](https://github.com/itsh-neumeier/kitaconnect/pull/69) ergänzt den [Speichervertrag](../contracts/identity-access/session-storage-v1.md) und ADR-030.
Stand 6d7425ea316e533496f1f5597c0845fda94c352d: erster vollständiger CI-Lauf mit fünf erfolgreichen Pflichtprüfungen.
[Build und echte SQLx-/PostgreSQL-Nachweise](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39420733257),
[Secret- und Abhängigkeitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39420733188).
Aktuelle Zustandsprüfung und Widerruf sind im internen Speicheradapter vor und nach Server-Neustart geprüft.
Die Anwendung bleibt durch RLS gesperrt. Positive Tests verwenden ausschließlich die Test-Betriebsidentität.
Dies ist keine Anmeldung, aktuelle Mitgliedschaftsprüfung oder Benutzerberechtigung. Die offenen Interviewentscheidungen bleiben offen.
KC-006B bleibt bis Integration offen. Nächstes Teilpaket ist KC-006C / #64.

## Passwortfaktor aus KC-006C

[PR #70](https://github.com/itsh-neumeier/kitaconnect/pull/70) ergänzt den [Passwortvertrag](../contracts/identity-access/password-check-v1.md) und [ADR-031](architecture/ADR-031_Passwortpruefung.md).
Vertrag und ADR wurden in 3f964e6 vor der Implementierung versioniert.
Der Dienst verwendet RustCrypto argon2 0.6.0 mit einer festen Parameterliste.
Eine neue Migration 0002 ergänzt Passwortbestand und dauerhafte Versuchszähler; Migration 0001 bleibt unverändert.
Der vorhandene Datenbanktest für KC-006B läuft weiterhin gegen dessen eigenen Bestand mit Version 1.
Der neue Passworttest verwendet einen getrennten synthetischen Bestand mit Versionen 1 und 2.

Die Prüfung reserviert einen Versuch vor der Hasharbeit. Das gilt auch für unbekannte Konten und negative Fälle.
Grenzen werden ausdrücklich übergeben. Ein Erfolg setzt keine Zähler zurück und erzeugt keine Sitzung.
Die Hasharbeit ist pro Prozess begrenzt. Eine ältere freigegebene Parameterwahl wird erst nach richtiger Passwortprüfung aktualisiert.
Eine konkurrierende Änderung des Bestands verhindert ein positives Ergebnis aus dem vorherigen Stand.
Die interne Passwortpflege widerruft vorhandene Sitzungen derselben Identität atomar.
Dies ist kein produktiver Registrierungs- oder Wiederherstellungsablauf.

Produktive Grenzwerte, vertrauenswürdige Bildung der Prüfschlüssel und Messungen auf der Referenzhardware bleiben vor einer Laufzeitfreigabe erforderlich.
Die produktive Datenbankbindung und die bisherigen Interviewfragen bleiben offen.
KC-006C bleibt bis Integration offen. Als nächstes unabhängiges Teilpaket folgt KC-006D / #65 mit einem WebAuthn-Vertrag.
