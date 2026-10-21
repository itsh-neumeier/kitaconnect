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
| KC-006E | [#66](https://github.com/itsh-neumeier/kitaconnect/issues/66) | TOTP und atomar einmalige Wiederherstellung | KC-006B, KC-006C und KC-006D; bestätigte MFA-/Wiederherstellungsregeln |
| KC-006F | [#67](https://github.com/itsh-neumeier/kitaconnect/issues/67) | Dienstverträge, Gateway-Anbindung und Gesamtnachweise | KC-006C/D/E |

Die folgenden Abschnitte dokumentieren die einzelnen Arbeitsschritte. Den aktuellen Stand führt der [Checkpoint](ai-memory/CURRENT_STATE.md).
Jedes folgende Paket erhält einen begrenzten Vertrag vor Implementierung.
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
MFA-Zuordnung und Wiederherstellungsablauf sind inzwischen ausdrücklich bestätigt. Browser-Sitzungsgrenzen bleiben offen.
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

Prüfstand cd39fe8: alle fünf Pflichtprüfungen für Push und PR erfolgreich.
[Clippy, Rust-Tests, Build und echte Datenbanktests einschließlich Neustart](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39431069566),
[Secret- und Abhängigkeitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39431069880).
Der folgende Abschlusscheckpoint erhält eigene Prüfungen; den aktuellen Kopf weist PR #70 aus.

## WebAuthn-Vorbereitung für KC-006D

PR #70, Abschlusskopf e003eb055f4137ee66184152ce660e96d838be44, ist vollständig grün und zur Durchsicht bereit.
Der nächste Schritt bereitet [ADR-032](architecture/ADR-032_WebAuthn_Vorbereitung.md) und den [WebAuthn-Vertragsentwurf](../contracts/identity-access/webauthn-v1.md) vor.
Der Entwurf benennt Bibliothekskandidaten, serverseitige Einmaligkeit, aktuelle Credential-Prüfung und Negativfälle.
Ein getrennter lokaler Bibliotheks-Prüfgraph hat die aktuelle RustSec-Prüfung ohne Befund bestanden.
Dies ist kein ausführbarer WebAuthn- oder Projekt-Abhängigkeitsnachweis.

RP-ID/Ursprung, Benutzerverifikation und die Zulassung synchronisierter Passkeys wurden mit Vorschlägen angefragt.
Die Antworten stehen aus. Deshalb gibt es in diesem Schritt keine neue Laufzeitimplementierung, Abhängigkeit im Projekt-Lockfile oder Migration.
Nach der Entscheidung zuerst Vertrag und ADR aktualisieren und versionieren. Danach KC-006D weiter umsetzen.
Die Akzeptanzkriterien von #65 bleiben offen. Eine vorbereitete Dokumentation schließt das Teilpaket nicht ab.

## Bestätigter WebAuthn-Nachweis aus KC-006D

Die Projektleitung bestätigt „Alle drei Vorschläge bestätigen“: zunächst localhost im isolierten Test,
erforderliche Benutzerverifikation und erlaubte synchronisierte Passkeys. Die produktive Domain bleibt offen.
Der aktualisierte Vertrag und ADR-032 wurden in ae85a5f vor Code versioniert.
PR #71 implementiert den begrenzten Serveradapter und Migration 0003. Bestehende Migrationen bleiben unverändert.
Es gibt keine Passwortvoraussetzung, HTTP-Anmeldung, Sitzungserzeugung oder fachliche Rechtefreigabe.

Stand ff12df6: erster vollständiger Linux-Lauf erfolgreich.
[Build und echte WebAuthn-/PostgreSQL-Tests vor und nach Neustart](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39490754135),
[Secret- und Abhängigkeitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39490754221).
Der Test-Authentikator erzeugt Antworten mit bestehender Bibliotheks-Kryptografie.
Die Serverprüfung weist auch signierte Antworten für falsche Ursprünge, RP-IDs und fehlende Benutzerverifikation ab.
Die Prüfung belegt keine tatsächliche Geräte-PIN, Biometrie oder Synchronisierung mit einem bestimmten Passwortmanager.
Browser-/Gerätenachweise und Gateway-Anbindung folgen gesondert.

Neue RustSec-Hinweise für bestehende TLS-Pakete wurden durch rustls-webpki 0.103.16 und webpki-roots 1.0.10 behoben.
ADR-032 hält den tatsächlichen Projekt-Lockfile-Nachweis und die verbleibende Lizenzprüfung fest.
Der folgende Checkpoint ergänzt Negativtests und erhält eigene CI; den aktuellen Kopf weist PR #71 aus.
KC-006D bleibt bis Integration offen. Nächstes Teilpaket ist KC-006E / #66 mit weiterhin offenen MFA-/Wiederherstellungsregeln.

## Vorbereitung von KC-006E

Kopf 74c5e1731be0db55f1ef8698d7119f8fcd5cce7c von PR #71 besteht alle Pflichtprüfungen einschließlich ergänzter Negativtests.
Für KC-006E liegen ein [Vertragsentwurf](../contracts/identity-access/mfa-recovery-v1.md) und [ADR-033](architecture/ADR-033_TOTP_und_Wiederherstellung.md) vor.
Geprüfte Bibliothekskandidaten sind totp-rs 6.0.0 und aes-gcm 0.11.1.
Ein getrennter lokaler Prüfgraph besteht die aktuelle RustSec-Prüfung; dies ist kein Projekt-Lockfile- oder Buildnachweis.

Am 21.10.2026 bestätigt die Projektleitung alle vorgelegten MFA-, Wiederherstellungs- und Benachrichtigungsregeln.
Vertrag und ADR-033 wurden in cffb194 vor Implementierung aktualisiert. Diese Fragen nicht erneut stellen.
Der Adapter verwendet die gemeinsame Versuchsreservierung aus KC-006C und entwertet laufende Passkey-Vorgänge über KC-006D.
Diese technischen Abhängigkeiten verlangen kein vorhandenes Passwort oder Passkey als fachliche Voraussetzung für TOTP.
Migration 0004 ergänzt verschlüsselte TOTP-Faktoren, abgeleitete Offline-Codes, Wiederherstellungszustand und atomare Ereignisaufträge.
Stand ed44bb0 besteht den ersten vollständigen Linux-CI-Lauf einschließlich MFA-Datenbanktests vor und nach Neustart.
[CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39580138747),
[Sicherheitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39580138793).
Der folgende Dokumentationscheckpoint erhält eigene Prüfungen; aktuellen Kopf und Bereitschaft weist PR #72 aus.
Der produktive Schlüsselbetrieb bleibt offen. KC-006E bleibt bis Integration offen; nächstes Teilpaket ist KC-006F / #67.

## KC-006F: Vertragsvorbereitung

PR #72 ist auf cb3d8f62d1fde50dcb67b96468534cf8876fbe1a vollständig grün und zur Durchsicht bereit.
KC-006F beginnt mit [ADR-034](architecture/ADR-034_Browseranmeldung_und_Nachweisuebergabe.md)
und dem [Browser-Anmeldevertrag](../contracts/identity-access/browser-auth-v1.md).
Die bisherigen positiven Faktor-Ergebnisse sind keine atomaren oder einmaligen Sitzungsabschlussnachweise.
Der Entwurf beschreibt die erforderliche Übergabe, Bestandsversionen, gemeinsame lokale Sperrgrenze und konkrete negative Tests.
Er erhält die Organisationshoheit über Mitgliedschaften. Eine lokale Sitzungstransaktion ersetzt keine aktuelle Prüfung bei organization-config.

Reihenfolge innerhalb von #67: Antworten versionieren; konkreten Speicher-/Sperrvertrag festlegen; Identitätsdienst und aktuellen Mitgliedschafts-Prüfvertrag ergänzen;
Rust-Gateway mit bestätigtem Browser-Schutz anbinden; Browser-, TLS- und Dienstidentitätsnachweise gesondert führen.
Sitzungsfristen, Passkey-Gleichstellung bei MFA-Pflicht und Cookie-/CSRF-Regeln sind im Interview ausdrücklich angefragt.
Die Projektleitung bestätigt beide MFA-Varianten und den Browser-Schutz. Sitzungsfristen sind pro Einrichtung einstellbar.
Die anschließende Antwort bestätigt 900/28.800/300 Sekunden als Voreinstellungen und Obergrenzen.
Diese Antworten sind vor Code in 916109c versioniert. Nicht erneut fragen.

## KC-006F: Einstellungsspeicher als erster Umsetzungsschritt

[ADR-035](architecture/ADR-035_Sitzungsfristen_je_Einrichtung.md) und der [Speichervertrag](../contracts/organization-config/session-policy-v1.md) begrenzen diesen Schritt.
organization-config erhält einen aktuellen Einstellungssatz je Einrichtung mit atomarem Versionsvergleich.
Die neue Organisationsmigration 0003 ergänzt den Bestand und einen Versionsschutztrigger. Alte Migrationen bleiben unverändert.
Erzwungene RLS bleibt ohne erlaubende Runtime-Policy bestehen. Der Leseweg legt keine Voreinstellungen an.
Die neue Testdatenbank kc_session_policy_test bleibt von bestehenden Testbeständen getrennt.
Die CI muss Werteprüfung, Konkurrenz, Rücknahme, Rechte und Neustart nachweisen.
Gateway, Mitgliedschafts-Prüfvertrag, atomare Faktorübergabe und laufende Sitzungsprüfung folgen weiterhin. Die Gesamtkriterien von #67 bleiben offen.
