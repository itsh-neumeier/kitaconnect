# Projektstatus und Fortsetzung

Aktualisiert: 21.10.2026, Europe/Berlin. Grundlage: Spezifikation v5.9.
Ausdrückliche Antwort der Projektleitung: „P0 abnehmen, und P1 freigeben“.
P0 ist abgenommen und abgeschlossen. P1 ist beauftragt. Diese Entscheidungen nicht erneut abfragen.

## Arbeitsstand

- issue_id: [KC-006F / #67](https://github.com/itsh-neumeier/kitaconnect/issues/67), Teil von KC-006 / #6.
- branch: codex/kc-006f-transaktionsabschluss auf PR #77 / 5927631.
- pull_request: Eigener Folge-PR auf codex/kc-006f-sitzungsrotation; Nummer und aktuellen Kopf in GitHub prüfen.
- head_commit: Abschlussvertrag und ADR-040 in 456fa51 vor Code versioniert. Aktueller Implementierungskopf steht im PR.
- objective: Offenen Backlog nach ausdrücklichem Folgeauftrag automatisch in kleinen abhängigen Paketen weiterbearbeiten. Aktuell eigene Transaktionen im Sitzungsbestand, Passwortadapter und Einstellungsspeicher bereinigen.
- completed: PR #77 / 5927631 besteht alle zehn aktuellen Prüfungen. Rotation, Fehler, Konkurrenz und Abmeldung sind vor/nach Neustart erfolgreich geprüft. PRs #73 bis #77 sind zur gesammelten Durchsicht bereit. Der Folgecode wartet Rollback in drei älteren Adaptern ab. Neue getrennte Datenbankzeugen prüfen offen gehaltene Fehlerverbindungen.
- remaining: Native CI des Folgecodes bestätigen. Danach TOTP, WebAuthn und Anmeldeabschluss auf dieselben Fehlerpfade prüfen. Zweckgebundene Zusatzauthentisierung ist nur lokal in 3358864 auf codex/kc-006f-zusatzauthentisierung vorbereitet; vor Weiterarbeit auf aktuellen Folgekopf rebasen. Die zusätzliche HTTPS-Test-Origin ist angefragt und noch unbestätigt. Organisationsadapter, Gateway und Browser-/TLS-/mTLS-Nachweise bleiben offen. Alle 44 offenen Issues bleiben nach WORK_QUEUE geordnet.
- migration_state: Keine neue Migration oder Abhängigkeit in diesem Korrekturschritt. Identitätsmigrationen 0001 bis 0007 und vorhandene getrennte Testdatenbanken bleiben unverändert.
- known_failures: Das Rotationszeitlimit aus 53610f4/cdd8b02 ist mit 5927631 behoben und nativ nachgewiesen. Weitere ältere Adapter haben noch nicht ausdrücklich abgeschlossene Fehlerpfade; Umfang siehe ADR-040. Windows-Linker/Docker fehlen lokal; #47/#48 behalten die Ablehnungsempfehlung.
- test_commands: cargo fmt --all --check; node --test scripts/*.test.mjs; node scripts/check-project.mjs; node --check scripts/test-organization-db.mjs; git diff --check; Gitleaks. Linux-CI: Clippy, Rust-Tests/Build, neuer expliziter SQLx-Test session_rotation/rotation_preserves_login_and_logout_lineage vor/nach Neustart sowie alle bisherigen zwölf getrennten Datenbank-Testgruppen.
- dependencies: #53, #54, #55, #59, #60, #61, #68, #69, #70, #71, #72 und #73 offen. PR #73 am Abschlusskopf vollständig grün. Keine ungefragte Integration.
- security_notes: MFA-Pflicht, Offline-Wiederherstellung, Benachrichtigungsaufträge, WebAuthn-Testprofil, beide MFA-Kombinationen und Cookie-/CSRF-Schutz sind bestätigt. Fristen sind pro Einrichtung einstellbar; 900/28.800/300 Sekunden sind bestätigte Voreinstellungen und Obergrenzen. Diese Fragen nicht erneut stellen. Bestehende positive Faktor-Ergebnisse sind weiterhin keine einmaligen Sitzungsabschlussnachweise. Produktiver Schlüsselbetrieb, Domain, Versuchslimits und Rollen-/Poolbindung bleiben gesondert.

## P0-Abschluss

- PRs #39 bis #44 integriert. KC-001, KC-002, KC-003, KC-036 und KC-037 abgeschlossen.
- PR #49 integriert über eaca2cb56a0e54a732ec17986f7c3ccb854c3986. KC-038 ist geschlossen.
- Alle fünf Main-Pflichtprüfungen nach Abschluss erfolgreich:
  [Build/Tests](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39155487071),
  [Sicherheitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39155487089).
- Keine weitere Bestätigung für P0 oder Beginn von P1 anfordern. Die ausdrückliche Abnahme ist dokumentiert.

## P1-Reihenfolge und Abnahmegrenze

Der [P1-Plan](../22_P1_Start_KC-004.md) zerlegt KC-004 in drei kleine Arbeitspakete:

1. #50 KC-004A: interner Modellvertrag und Rust-Bibliothek im Dienst organization-config.
2. #51 KC-004B: Persistenz und Migrationstests, abgestimmt mit KC-005.
3. #52 KC-004C: authentisierte Dienstzugriffe und negative Tests fremder Trägerzuordnung;
   benötigt Identitäts-/Policy-Grundlagen aus KC-006/KC-008.

KC-004 bleibt bis zu den ursprünglichen Persistenz-/Rechtekriterien offen.
Modelltests sind keine Benutzerberechtigungs- oder Datenbankisolationstests.
Die Bibliothek ist kein Laufzeitdienst. Es gibt keine öffentliche Erstellungs-/Transfer-API.
Vor weiteren Organisationshierarchien oder Transferregeln Fachumfang klären.

## Bestätigte Entscheidungen und Sicherheitsumfang

- ADR-014 bis ADR-017 durch „1A, 2A, 3A, 4A“ bestätigt. Nicht erneut abfragen.
- ADR-023 begründet das interne Organisationsmodell; keine neue Berechtigungsregel.
- Lastprofil und Messverfahren bleiben bestätigt; repräsentative Lastmessungen stehen aus.
- CODEOWNER und benannter Sicherheits-/Datenschutzprüfer: @itsh-neumeier Timo Neumeier.
- Rechteinhaber, juristische Lizenzprüfung und unabhängige Wahlprüfung bleiben ausdrücklich offen.
  Sperren und Fristen aus [KC-037](../17_P0_Entscheidungsregister.md) gelten weiter.
- F-001: Server prüft Berechtigung vor Speichern. npm audit lokal und in CI erlaubt.
- Basisregelsatz 25341109 bleibt ohne Ausnahme von Pflichtprüfungen, aktueller Basis und PR-Zwang.
- Review-Regelsatz 25346711: bestätigte Konto-Ausnahme nur User 77058548 / pull_request.
  Nutzung nur für eigene PRs bleibt dokumentierte Regel. Persönliche Prüfung und gesonderter Merge-Auftrag bleiben vorgesehen.
- Keine Echtdaten, Secrets oder Veröffentlichung. Keine juristische oder unabhängige Fachfreigabe aus P0 ableiten.
- Dependabot-PRs #45 bis #48 bleiben unberührt.

## Prüfungen des ersten P1-Pakets

- Lokal erfolgreich: cargo generate-lockfile --offline; cargo check --workspace --all-targets --locked --offline;
  cargo fmt --all --check; cargo clippy --workspace --all-targets --locked --offline -- -D warnings.
- Struktur-/Verweisprüfung für 38 Issues und 35 Dokumente, beide Regelsatzvalidatoren, 40 Node-Tests und git diff --check erfolgreich.
- Implementierung 1332bd7: alle fünf Pflichtprüfungen erfolgreich. Linux-CI führt Rust-Integrationstests, negativen Dokumentationstest und Build erfolgreich aus.
  Nachweise: [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39155761533),
  [Sicherheitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39155761404).
  Der folgende Checkpoint-Commit erhält eigene Prüfungen.
- Gitleaks auf vorgemerkter Implementierung ohne Befund. Checkpoint ebenfalls vor Commit prüfen. Ergebnis im PR nachführen.

## KC-005-Reihenfolge und Prüfstand

- KC-005A / #56: leere Schemata und Registry atomar anlegen; Runtime-Konfiguration und physischen Zustand prüfen.
- KC-005B / #57: freigegebene Dienstobjekte, getrennte Rechte und erzwungene RLS je Einrichtung prüfen.
- KC-005C / #58: Poolwechsel nach Erfolg/Fehler/Abbruch sowie konkurrierende Migrationsabläufe nachweisen.
- KC-004A/B sind konkrete Voraussetzungen von KC-005 und KC-006; KC-004C bleibt von KC-008 abhängig.
  Die gesamte KC-004-Abnahme bleibt offen. Keine fachliche Bedingung wurde gestrichen.
- Backlog-Prüfung berücksichtigt Teilpakete als Voraussetzungen der Gesamtabnahme und erkennt dadurch verdeckte Zyklen.
- Lokal bisher erfolgreich: 44 Node-Tests, Struktur-/Verweisprüfung für 44 Issues und 38 Dokumente, JavaScript-Syntax und git diff --check.
- Implementierung c3369d6: erster vollständiger CI-Lauf mit fünf erfolgreichen Pflichtprüfungen, einschließlich echter PostgreSQL-Provisionierungs-, Rollen-, Rollback- und Neustarttests.
  [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39368424819),
  [Sicherheitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39368424845).
- Stand 518415e: alle fünf Pflichtprüfungen des neuesten PR-Laufs erfolgreich. Ergänzte Rollenattribute, zurückgenommene Rechtevergaben und der negative Rust-Dokumentationstest zum freien Client-Schemanamen sind erfolgreich.
  [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39368712802),
  [Sicherheitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39368712808).
- Rustfmt und Gitleaks waren für Implementierung und Ergänzungen erfolgreich. Der folgende reine Dokumentationscommit erhält eigene CI; Nachweise im PR nachführen.
- Neue Migration verändert keine vorhandene Datei 0001. Registry-/Kennungslese-Policies erlauben ausschließlich der Wartungsrolle Metadatenzugriff.
- Bestehende Runtime-Rolle erhält keine Registry- oder Einrichtungsschema-Rechte. Keine Fachobjekte, Personalbereiche oder Wahldatenbanken angelegt.

## KC-005B: Prüfstand und Grenzen

- PR #60 ergänzt ausschließlich Testartefakte; die Produktionsprovisionierung bleibt unverändert.
- Implementierung 0b8a20b: alle fünf Pflichtprüfungen des ersten vollständigen Laufs erfolgreich.
  [Build und echte Datenbanktests](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39388995659),
  [Sicherheitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39388996011).
- SQLx-Nachweise vor und nach Neustart: Dienst-/Einrichtungsgrenzen, positive Schreibpfade, FORCE RLS für Eigentümer,
  Namensverdeckung, Funktionsrechte, Rollen-/Eigentümerwechsel und atomare Rücknahme des Testartefakts.
- Lokal: 44 Node-Tests, Struktur-/Verweisprüfung für 44 Issues und 39 Dokumente, rustfmt, JavaScript-Syntax und Gitleaks erfolgreich.
- Ergänzungsstand aedb4a0 setzt gezielt die Kennung der anderen vorhandenen Einrichtung. Alle fünf Pflichtprüfungen des PR-Laufs sind erfolgreich.
  [CI mit echten Datenbanktests](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39389278957),
  [Sicherheitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39389279112).
  Dieser reine Dokumentationscheckpoint erhält eigene CI; Nachweise zum aktuellen Kopf werden im PR nachgeführt.
- Die Frage nach eigener produktiver Identität und eigenem Pool je Dienst/Einrichtung wurde gestellt; Antwort noch offen.
  Keine solche Produktionsarchitektur ohne ausdrückliche Entscheidung implementieren. Technische Negativtests bleiben möglich.

## KC-005C: Prüfstand und Gesamtabnahmegrenzen

- Vertrag und ADR-028 wurden in d32879a vor dem Testcode versioniert.
- Implementierung e4c1e00: fünf Pflichtprüfungen des ersten vollständigen CI-Laufs erfolgreich.
  [Build und echte Datenbanktests](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39406137461),
  [Sicherheitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39406137330).
- Poolnachweise: gleicher Datenbankprozess nach Erfolg, SQL-Fehler, verworfener Rust-Operation und Anweisungszeitlimit;
  Rücknahme abgebrochener Schreiboperationen, Rechte-/Policyentzug auf offener Verbindung und Verwerfen bei fehlerhaftem Rückgabe-Hook.
- Wartungsnachweise: doppelte Provisionierung, doppelte Testmigration und gemischte Wartung warten tatsächlich an derselben Sperre.
  Die fehlerhafte Testwelle stoppt, nimmt nur den betroffenen Schritt zurück und bewahrt frühere bestätigte Änderungen.
- Der folgende reine Dokumentationscheckpoint erhält eigene CI. Nachweise zum aktuellen Kopf im PR nachführen.
- Lokal: 44 Node-Tests, Struktur-/Verweisprüfung, rustfmt, JavaScript-Syntax, git diff --check und Gitleaks erfolgreich.
- Der [KC-005-Prüfstand](../23_KC-005_Pruefstand_und_Abnahmegrenzen.md) trennt technische Teilnachweise und offene Gesamtabnahme.
  Produktive Identitätsbindung, erlaubter produktiver Einrichtungswechsel, allgemeine Migrationssteuerung, Benutzerrechte und Wiederherstellung sind nicht abgenommen.

## KC-006A: Prüfstand und Fortsetzung

- [KC-006-Plan](../24_KC-006_Authentisierung.md) zerlegt #6 in KC-006A bis KC-006F / #62 bis #67.
  Alle ursprünglichen Passwort-, Passkey-, TOTP-, Wiederherstellungs- und Widerrufskriterien bleiben bestehen.
- Vertrag und ADR-029 wurden in 85fed18 vor dem Modellcode versioniert.
- Implementierung 5abb46e: erster vollständiger CI-Lauf mit fünf erfolgreichen Pflichtprüfungen.
  [Build, Clippy, elf Modelltests und negativer Typvertragstest](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39411531255),
  [Secret- und Abhängigkeitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39411531380).
- Lokal erfolgreich: paketbezogene Typprüfung und Clippy mit --all-targets --locked --offline, rustfmt,
  44 Node-Tests, Struktur-/Verweisprüfung für 50 Issues und 43 Dokumente, git diff --check und Gitleaks.
- Cargo.lock ergänzt ausschließlich das neue Paket; keine neue externe Abhängigkeit oder Werkzeugversion.
- Der folgende reine Dokumentationscheckpoint erhält eigene CI. Nachweise zum aktuellen Kopf werden im PR nachgeführt.
- Keine eingebauten Projektwerte, Passwortprüfung, Cookies, MFA, Speicherung, Tokenausstellung oder Berechtigung implementiert.
- Rückfragen zu MFA-Zuordnung, Sitzungswerten und Wiederherstellung wurden mit konkreten Vorschlägen gestellt.
  Es liegt noch keine ausdrückliche Antwort vor. Keine abhängigen Regeln als bestätigt behandeln.
  Details stehen in [DECISIONS_PENDING](DECISIONS_PENDING.md).

## KC-006B: Prüfstand und Speichergrenzen

- Vertrag und ADR-030 wurden in 2487f86 vor Implementierung versioniert.
- Implementierung 6d7425e: erster vollständiger CI-Lauf mit fünf erfolgreichen Pflichtprüfungen.
  [Build, Clippy und echte PostgreSQL-/SQLx-Tests](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39420733257),
  [Secret- und Abhängigkeitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39420733188).
- Der neue Datenbanktest wird ausdrücklich vor und nach Server-Neustart ausgeführt. Rohe Sitzungsnachweise bleiben nur im Arbeitsspeicher.
- Geprüft: gespeicherte Ableitung, numerische Grenzen, doppelte und fremde Kennungen, Fristen, Widerruf, aktuelle Fehlerverweigerung,
  tatsächliche Zeilensperrwartende, Ablauf während Wartezeit, unterdrücktes Update, äußere Transaktionen und Verbindungsverlust.
- Interne Operationen verlangen eine Verbindung außerhalb aufrufender Transaktionen. Eigene mehrteilige Vorgänge verwenden READ COMMITTED.
  Auch manuell per Roh-SQL geöffnete äußere Transaktionen sind unzulässig. Spätere zusammengesetzte Abläufe benötigen einen eigenen Vertrag.
- Direkte Bibliotheksabhängigkeiten sind exakt festgelegt: getrandom 0.4.3, sha2 0.11.1, zeroize 1.9.1, SQLx 0.9.0; Tokio 1.54.0 für Tests.
  Alle Versionen waren bereits im Lockfile vorhanden. SHA-2-Standardfeatures bleiben deaktiviert; keine zusätzliche OID-Abhängigkeit benötigt.
- Lokal erfolgreich: rustfmt, JavaScript-Syntax, 44 Node-Tests, Struktur-/Verweisprüfung für 50 Issues und 44 Dokumente,
  git diff --check und Gitleaks. Der ausführbare Rust-/Datenbanknachweis stammt aus Linux-CI.
- Der folgende reine Dokumentationscheckpoint erhält eigene CI. Nachweise zum aktuellen Kopf werden im PR nachgeführt.
- Das Modell ist keine aktuelle Benutzerberechtigung. Jeder geschützte Vorgang muss aktuell prüfen; keine positive Ergebniszwischenspeicherung.
  Die Anwendungsrolle bleibt gesperrt. Die drei Interviewvorschläge und die produktive Rollenbindung sind nicht bestätigt.

## KC-006C: Prüfstand und Passwortgrenzen

- Vertrag und ADR-031 wurden in 3f964e6 vor Implementierung versioniert.
- Kernstand 2ec3983: erster vollständiger CI-Lauf mit fünf erfolgreichen Pflichtprüfungen.
  [Clippy, Rust-Tests, Build und echte Datenbanktests](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39430379027),
  [Secret- und Abhängigkeitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39430379035).
- Der neue Datenbanktest ist vor und nach Neustart ausdrücklich erfolgreich ausgeführt.
  Geprüft: richtige und falsche Passwörter, unbekannte Kennungen, ungültige Formate, begrenzte Parameter und Aktualisierung,
  konkurrierender Versionswechsel, atomare Sitzungswiderrufe bei Passwortpflege und reservierte Versuche über Neustart.
  Schlüssellimit, Gesamtlimit, Mengenlimit, Konfigurationsabweichung und rückläufige Datenbankzeit werden negativ geprüft.
- Ein weiterer Rust-Test belegt, dass ein Aufrufabbruch die Begrenzung laufender Hasharbeit nicht vorzeitig freigibt.
- 404c04c ergänzt die konkurrierende Hashänderung bei unveränderter Versionskennung, Eigentümer-RLS und fehlende Sitzungserzeugung.
  Abschlussstand cd39fe8: alle fünf Pflichtprüfungen von Push und PR erfolgreich, einschließlich dieser Ergänzungen.
  [CI mit ergänzten Datenbanktests vor und nach Neustart](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39431069566),
  [Secret- und Abhängigkeitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39431069880).
  Der folgende Abschlusscheckpoint erhält eigene CI; Kopf und Nachweise im PR nachführen.
- Fünf neue Lockfile-Pakete: argon2 0.6.0, base64ct 1.8.3, blake2 0.11.0, password-hash 0.6.1 und phc 0.6.1.
  Bestehende Paketversionen wurden nicht aktualisiert. Tokio wird auch für begrenzte Hintergrundarbeit verwendet.
- Lokal erfolgreich: 44 Node-Tests, Struktur-/Verweisprüfung für 50 Issues und 45 Dokumente, Rustfmt,
  JavaScript-Syntax, git diff --check und Gitleaks. Rust- und PostgreSQL-Ausführung stammen aus Linux-CI.
- Keine produktiven Vorgabewerte, erlaubenden RLS-Policies oder HTTP-Anmeldung. Offene Interviewentscheidungen bleiben offen.

## KC-006D: Vertragsvorbereitung und Entscheidungssperre

- [ADR-032](../architecture/ADR-032_WebAuthn_Vorbereitung.md) und der [Vertragsentwurf](../../contracts/identity-access/webauthn-v1.md) dokumentieren den nächsten begrenzten Umfang.
- Bibliothekskandidaten webauthn-rs und webauthn-authenticator-rs 0.5.5: Rust 1.88, MPL-2.0 laut Paketmetadaten.
  Der Passkey-Ablauf verlangt Benutzerverifikation und lässt synchronisierte Schlüssel zu. Beides ist angefragt, noch nicht als Projektregel bestätigt.
- Ein getrennter ignorierter Prüfgraph mit 143 Paketen einschließlich Prüfpaket besteht cargo-audit 0.22.2 mit --deny warnings.
  Frisch aktualisierte RustSec-Datenbank: b19b2c83c2493961ccb12f9fc4699a00fb2c6c44, 1.345 Hinweise.
  Kein Anwendungscode gebaut. Kein Projekt-Lockfile-Nachweis, keine Lizenzfreigabe und kein OpenSSL-Build daraus ableiten.
- Lokal erfolgreich: Struktur-/Verweisprüfung für 50 Issues und 46 Dokumente, 44 Node-Tests, git diff --check und Gitleaks.
- Keine neue Laufzeitfunktion, Abhängigkeit im Projekt-Lockfile oder Migration. #65 bleibt vollständig offen.
- Die Fragen stehen in [DECISIONS_PENDING](DECISIONS_PENDING.md). Eine allgemeine Aufforderung zur Weiterarbeit beantwortet sie nicht.
  Unabhängige Vertrags-/Bibliotheksvorbereitung ist erfolgt. Abhängige Implementierung erst nach Antwort fortsetzen.

## KC-006D: Bestätigtes Profil und Implementierungsnachweis

- Die Projektleitung bestätigt ausdrücklich alle drei WebAuthn-Vorschläge. Diese Antwort ersetzt die frühere Entscheidungssperre für das Testprofil.
  Allgemeine MFA-Zuordnung, Wiederherstellung und produktive Domain werden damit nicht entschieden.
- Vertrag und ADR-032 wurden in ae85a5f vor Implementierung aktualisiert. Abhängigkeiten in 6ca7c40; Kerncode in ff12df6.
- Erster vollständiger Linux-CI-Lauf erfolgreich:
  [Clippy, Rust-Tests, Build und echte Datenbanktests](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39490754135),
  [Secret- und Abhängigkeitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39490754221).
  WebAuthn-Test ausdrücklich vor und nach Neustart erfolgreich. Native OpenSSL-Anbindung baut erfolgreich.
- Geprüft: unabhängige Registrierung/Anmeldung ohne Passwortbestand und Sitzungserzeugung; einmalige Referenzen,
  falscher Ursprung, Port, Subdomain, RP-ID, Challenge, Zweck, Identität und Benutzerhandle; fehlende Benutzerverifikation;
  manipulierte Signatur, Authentikatordaten und Clientdaten; aktueller Widerruf, Bestandswechsel, reale Sperrwartende,
  Mengenlimits, Runtime-/Eigentümer-RLS und Migrationsrücknahme.
- Ergänzende Tests dieses Checkpoints prüfen doppelte Credential-Zuordnung, übergroße Antwort,
  Ablauf nach tatsächlicher Kontosperrwartezeit sowie Abbruch nach dauerhaftem Verbrauch. Deren CI im PR nachführen.
- 68 neue transitive Pakete für Server-/Testbibliotheken, bestehende Paketversionen bis auf die zwei TLS-Fixes unverändert.
  Tatsächlicher Projekt-Lockfile besteht cargo-audit 0.22.2 mit --deny warnings; 229 Pakete, 1.348 Hinweise,
  RustSec-Stand e6f94f3760d5e6c54c1b05f204002263bc025524. Dies ist keine juristische oder unabhängige Sicherheitsfreigabe.
- Lokal erfolgreich: Rustfmt, JavaScript-Syntax, 44 Node-Tests, Struktur-/Regelsatzprüfung,
  synthetische Fixtures, git diff --check und Gitleaks. Keine lokalen Rust-/PostgreSQL-Erfolge behaupten.

## KC-006E: Unabhängige Vorbereitung und offene Antworten

- [ADR-033](../architecture/ADR-033_TOTP_und_Wiederherstellung.md) und der [Vertragsentwurf](../../contracts/identity-access/mfa-recovery-v1.md) sind in 12e2cd7 versioniert.
- Bibliothekskandidaten: totp-rs 6.0.0, MIT, Rust 1.88; aes-gcm 0.11.1, Apache-2.0 oder MIT, Rust 1.85 laut Paketmetadaten.
  Keine eigene Kryptografie vorgesehen. Ein TOTP-Geheimnis benötigt geschützte entschlüsselbare Speicherung; dessen Hash genügt nicht.
  Ein passender TOTP-Zeitschritt muss im Dienst selbst dauerhaft und atomar verbraucht werden.
- Getrennter ignorierter Prüfgraph mit 32 Paketen einschließlich Prüfpaket besteht cargo-audit 0.22.2 mit --deny warnings.
  Frisch aktualisierte RustSec-Datenbank: e6f94f3760d5e6c54c1b05f204002263bc025524, 1.348 Hinweise.
  Kein Projekt-Lockfile- oder nativer Buildnachweis, keine juristische oder produktive Schlüsselbetriebsfreigabe.
- Lokal erfolgreich: Struktur-/Verweisprüfung für 50 Issues und 47 Dokumente, git diff --check und Gitleaks.
  Keine neuen Tests für die reine Dokumentationsänderung geschrieben. CI des aktuellen Kopfes im PR nachführen.
- Zwei Interviewfragen sind gestellt: gemeinsame Bestätigung von MFA-Zuordnung und Wiederherstellung;
  zusätzlich Sicherheitsprotokoll und Benachrichtigungsauftrag bei Einrichtung, Wechsel, Widerruf und Wiederherstellung.
  Beide Antworten stehen aus. Die bisherige WebAuthn-Antwort und „Weiter machen“ bestätigen diese Regeln nicht.
- Die frühere Fortsetzungsprüfung zu PR #71 ist abgeschlossen: Kopf 74c5e17 besteht alle fünf Pflichtprüfungen von Push und PR,
  einschließlich ergänzter Datenbanktests vor/nach Neustart. PR #71 ist kein Entwurf mehr.
  [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39491483136),
  [Sicherheitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39491483085).

## KC-006E: Bestätigter Vertrag und Implementierungsabschluss

- Die Antwort „Ja, alle genannten Regeln bestätigen“ bestätigt MFA-Zuordnung, Offline-Wiederherstellung und atomare Benachrichtigungsaufträge. Nicht erneut fragen. Vertrag und ADR-033 wurden in cffb194 vor Code aktualisiert; Zählerschutz und Code-Widerruf in 8550a01 vor der Ergänzung.
- Kerncode in 5c27f7a, Ergänzungen in 5d4b1ec und ed44bb0. Der interne Adapter erzeugt keine normale Sitzung oder Fachberechtigung und versendet keine Nachricht.
- Migration 0004 ergänzt sieben Tabellen, einen Nonce-Schutztrigger und Version 4. Migrationen 0001 bis 0003 bleiben unverändert. Der Test prüft Rücknahme, Wiederholung und sauberen Wiederanlauf in kc_mfa_test. Laufzeit- und Eigentümer-RLS bleiben geschlossen.
- TOTP: öffentliche RFC-6238-Vektoren, explizite Zeitfenster, Aktivierung, Ersetzen, Widerruf und dauerhafter Schrittverbrauch. Zwei tatsächlich an derselben Zeile wartende Prüfungen ergeben genau einen Erfolg.
- Geheimnisschutz: AES-GCM, dauerhafte Zählerreservierung, Rücksetzungsschutz und Nutzungslimit; Manipulation, falscher Schlüssel sowie vertauschte Faktor-/Identitätsbindung werden abgewiesen. Nur frische zufällige Testschlüssel im RAM; kein Import oder Export.
- Wiederherstellung: abgeleitete Offline-Codes, atomarer einmaliger Verbrauch auch bei realer Konkurrenz, begrenzte Nachweise, Ablauf, fremde Identität, ungültiger alter Satz und Sperre bis zur Neueinrichtung. Ein Faktorwiderruf entwertet auch den Offline-Code-Satz.
- Rücknahmetests: fehlender bestätigter Kontakt, unterdrückter Outbox-Eintrag und unterdrückter Sitzungswiderruf. Verbrauch, Faktorbestand und Sicherheitsprotokoll werden gemeinsam zurückgenommen. Versuchszählung und Nonce-Reservierung bleiben verbraucht.
- Bestehende Sitzungen werden bei Aktivierung, Widerruf und Wiederherstellung widerrufen. Die Passkey-Version wird entwertet. KC-006F muss die aktuelle Wiederherstellungssperre auch bei Passwort- und Passkey-Anmeldung beachten; einzelne alte positive Ergebnisse genügen nicht.
- Erster vollständiger Linux-Lauf auf ed44bb0 erfolgreich:
  [Clippy, Rust-Tests, Build und Datenbanktests vor/nach Neustart](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39580138747),
  [Secret- und Abhängigkeitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39580138793).
  Dieser reine Dokumentationscheckpoint erhält eigene CI; aktuelle Nachweise im PR nachführen.
- Tatsächlicher Projekt-Lockfile: 243 Pakete, cargo-audit 0.22.2 mit --deny warnings ohne Sicherheitsbefund. RustSec e6f94f3760d5e6c54c1b05f204002263bc025524, 1.348 Hinweise. 14 neue transitive Pakete, keine bestehende Paketversion aktualisiert. Keine juristische oder unabhängige Sicherheitsfreigabe daraus ableiten.
- Lokal erfolgreich: Rustfmt, 44 Node-Tests, Struktur-/Verweisprüfung für 50 Issues und 47 Dokumente, Regelsatzprüfung, synthetische Fixtures, JavaScript-Syntax, git diff --check und Gitleaks.
- Für KC-006F wurde die noch offene Frage zu 15 Minuten Inaktivität, 8 Stunden absoluter Laufzeit und 5 Minuten zusätzlicher Authentisierung erneut gestellt. Antwort steht aus. Produktive Domain, Versuchslimits, Prüfschlüsselbindung, Rollenbindung und Schlüsselbetrieb bleiben gesondert offen.

## KC-006F: Vorbereitung und offene Interviewantworten

- Fortsetzungsprüfung abgeschlossen: PR #72 ist offen und kein Entwurf mehr. Abschlusskopf cb3d8f62d1fde50dcb67b96468534cf8876fbe1a besteht alle zehn Pflichtprüfungen.
  [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39580909505),
  [Sicherheitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39580909278).
- [ADR-034](../architecture/ADR-034_Browseranmeldung_und_Nachweisuebergabe.md) und der [Browser-Anmeldevertrag](../../contracts/identity-access/browser-auth-v1.md) sind in 20a88bf versioniert. PR #73 bleibt ein Entwurf.
- Die drei Fragen wurden gemeinsam mit Vorschlägen gestellt. Eine Antwort steht aus. Die Aufforderung „Weiter“ bestätigt keine der Sicherheitsregeln.
- Technische Richtung: versionierte und einmalige Nachweisübergabe aus der tatsächlichen Faktorprüfung; atomarer lokaler Sitzungsabschluss; aktuelle Mitgliedschaft beim Organisationsdienst. Kein ungeprüftes create_session nach einem alten positiven Ergebnis.
- Kein Gateway-Dienst, keine Browseranmeldung, kein Mitgliedschaftsadapter, keine Migration und keine neue Abhängigkeit ergänzt. Keine bestehende Funktion aus KC-006E erneut implementiert.
- Lokal erfolgreich: 44 Node-Tests, Struktur-/Verweisprüfung für 50 Issues und 48 Dokumente, git diff --check und Gitleaks. Keine neuen Tests für die reine Dokumentationsänderung geschrieben. Aktuelle CI-Nachweise im PR nachführen.

## KC-006F: Bestätigte Regeln und erster Einstellungsspeicher

- Die Projektleitung antwortet „Einstellbar pro Einrichtung“, „Beide Varianten bestätigen“ und „Vorschlag bestätigen“. Die Nachfrage zu Voreinstellungen und Obergrenzen beantwortet sie mit „Diese Voreinstellungen und Obergrenzen bestätigen“. Damit sind alle vier Antworten eindeutig; frühere offene Fragezustände sind ersetzt.
- In 916109c werden die Antworten, [ADR-035](../architecture/ADR-035_Sitzungsfristen_je_Einrichtung.md) und der [Einstellungsspeichervertrag](../../contracts/organization-config/session-policy-v1.md) vor Implementierung versioniert.
- 19fc229 implementiert einen aktuellen Einstellungssatz pro Einrichtung im Organisationsdienst. Alle drei Werte sind positiv, innerhalb der bestätigten Obergrenzen und mit der Gesamtlaufzeit konsistent. Voreinstellungen werden ausdrücklich gewählt; Lesen legt keinen fehlenden Satz an.
- Ein erwarteter Versionsstand schützt konkurrierende Änderungen. Die neue Migration 0003 erzwingt Grenzen, unveränderliche Zuordnung und genau einen Versionsfortschritt. Sie enthält keine erlaubende RLS-Policy und keinen Sitzungszugriff.
- Neue Tests: Werte-/Voreinstellungsgrenzen, getrennte Einrichtungen, tatsächlich wartende konkurrierende Schreiber, unterdrücktes Update, verzögerter Commitfehler, Versionsexhaustion, äußere Transaktionen, verlorene Verbindung, Eigentümer-/Runtime-RLS und Erhalt nach Neustart. Der CI-Runner prüft Migrationsrücknahme und saubere Anwendung in einer eigenen Datenbank.
- Lokal erfolgreich: Rustfmt, 44 Node-Tests, Struktur-/Verweisprüfung für 50 Issues und 49 Dokumente, JavaScript-Syntax, git diff --check und Gitleaks. Die Linux-CI muss die neuen nativen Tests gesondert bestätigen; aktuelle Ergebnisse werden im PR nachgeführt.
- Diese Komponente erzwingt noch keine Frist bei einer Browseranfrage. Atomare Faktorübergabe, aktueller Mitgliedschafts-Prüfvertrag, Gateway, Cookie-/CSRF-Implementierung, Browser-/TLS-/mTLS-Nachweise und vollständige Anmeldeabnahme folgen weiter in #67. Eine Lockerung darf bestehende Nachweise nicht verlängern; eine Verschärfung muss später bei der nächsten geschützten Prüfung wirken.

## KC-006F: Einmalige Faktorübergabe und lokaler Abschluss

- Fortsetzungsprüfung von PR #73 abgeschlossen: f5f0586 besteht alle zehn Prüfungen einschließlich neuer Konfigurationstests vor/nach Neustart. Keine erledigte Konfiguration erneut implementieren.
- [ADR-036](../architecture/ADR-036_Einmaliger_Anmeldeabschluss.md) und der [Anmeldeabschlussvertrag](../../contracts/identity-access/login-completion-v1.md) wurden vor Code versioniert. PR #74 auf #73 begrenzt den nächsten Teil.
- Die neuen Varianten check_for_login, verify_for_login und finish_authentication_for_login merken ihren Erfolg unmittelbar in der echten Faktortransaktion vor. Alte positive Ergebnisobjekte erhalten keine Umwandlung. Einmaliger Verbrauch, neue Sitzung und Policy-/Einrichtungsbindung werden gemeinsam bestätigt.
- Ein gemeinsamer transaktionsgebundener Identitätslock serialisiert Faktorpflege und Abschluss. Eine begrenzte Bestandsableitung berücksichtigt auch PHC-Änderungen ohne Versionsfortschritt. Passkey-Entzug und Passwortpflege widerrufen bestehende Sitzungen atomar und prüfen die betroffene Zeilenzahl.
- Ursprüngliche und aktuelle Fristen werden durch das Minimum verbunden. Policy-Rücklauf, veränderte Werte derselben Version, unvollständige MFA und Wiederherstellungssperre verweigern den Abschluss. Die Datenbankzeit wird nach einer wartenden Organisationsantwort erneut geprüft.
- Organisationsinformationen stammen in neuen Tests ausschließlich aus dem benannten SyntheticOrganization-Testdoppel. Die interne Adaptergrenze ist kein mTLS-, Netzwerk- oder produktiver Mitgliedschaftsnachweis. Spätere geschützte Aktionen müssen selbst erneut aktuell prüfen.
- Neue Tests prüfen echte Passwort-/TOTP-/Passkey-Erfolge, fehlende Faktoren, falsche Bindung/Route, zwei tatsächlich wartende Abschlüsse, mit Abschluss konkurrierende Wiederherstellung und sofortigen Widerruf der gerade erzeugten Sitzung, veränderten Bestand, aktuelle Mitgliedschaftsverweigerung, Policy-Verschärfung/-Lockerung, Commitfehler, unterdrückten Eintrag, Ablauf nach Organisationswartezeit, Grenzen, RLS und Neustart.
- Kernstand: [erster vollständiger Push-Lauf](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39588080448), [Sicherheitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39588080343). Dieser Lauf umfasst noch nicht die neuen SQLx-Tests aus aafa802; aktuelle Nachweise im PR nachführen.
- Lokal sind Rustfmt, Struktur-/Verweisprüfung und Gitleaks erfolgreich. Keine neuen externen Bibliotheken oder Lockfile-Änderungen. Der aktuelle Checkpoint erhält eigene CI.

## KC-006F: Dauerhafte Ablaufsperre und ergänzte Nachweise

- bec4c91 besteht beide Rust-Läufe einschließlich echter Login-Tests vor und nach Neustart. Die Wiederherstellung wartet nachweisbar an derselben Identitätssperre wie der Abschluss. Nach dem Abschluss widerruft sie auch die gerade erzeugte Sitzung.
  [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39589510033),
  [Sicherheitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39589510090).
- df54e28 konkretisiert den endgültigen Ablauf im Vertrag und ADR vor der Korrektur d25ed9e. Ein beobachteter Ablauf sperrt den Vorgang dauerhaft. Ein späterer Zeitrücksprung darf ihn nicht reaktivieren.
- Neue Datenbanktests prüfen den Ablauf bei tatsächlicher Passwort-, TOTP- und Passkey-Prüfung sowie vor und nach der Organisationswartezeit. Geprüfte Faktorzustände dürfen dabei bestätigt werden; es entsteht kein Erfolgsmarker und keine Sitzung. Der Schutztrigger verhindert das Aufheben der Sperre.
- Die neue, noch nicht integrierte Migration 0005 ergänzt dafür invalidated. Migrationen 0001 bis 0004 bleiben unverändert. Keine neue Abhängigkeit, kein Gateway und kein produktiver Organisationsadapter.
- Die aktuelle CI muss die ergänzten Ablauftests bestätigen. Abschlussnachweise und Kopf werden in PR #74 und Issue #67 nachgeführt. KC-006F bleibt bis zur übrigen Integration offen.

## Nächste drei Aktionen

Aktueller Korrekturschritt nach PR #77:

1. Folge-PR und CI des Transaktionsabschlusses prüfen. Getrennte Zeugen müssen Fehlerverbindungen ohne Folgeabfrage im Zustand idle ohne Transaktion bestätigen.
2. Danach TOTP, WebAuthn und Anmeldeabschluss auf dieselben Fehlerpfade prüfen. Bestätigte Versuchszähler, Herausforderungverbrauch und dauerhafte Ablaufzustände erhalten. Erst danach den lokalen Zusatzprüfvertrag 3358864 auf den neuesten Kopf übernehmen und implementieren.
3. Antwort zum zusätzlichen HTTPS-Testprofil übernehmen, sobald sie vorliegt. Ohne Antwort dieses Profil nicht implementieren. Unabhängige Organisations-/Gateway-Vorbereitung gemäß WORK_QUEUE fortsetzen.

Der Folgeauftrag umfasst den gesamten offenen Backlog. PR #77 ist erfolgreich geprüft; nicht erneut implementieren.
Die [Arbeitsfolge](WORK_QUEUE.md) ist die Fortsetzungsgrundlage.
Die folgenden früheren Durchsichtsaktionen sind mit PR #75 technisch erledigt; keine erledigten Adapterkorrekturen erneut implementieren.

1. Aktuellen Kopf und CI von Durchsichts-/Korrektur-PR #75 prüfen. Die nativen Korrekturtests von 6a1ccff sind bereits vor/nach Neustart erfolgreich; sie nicht erneut implementieren. Die Empfehlungen aller 17 ursprünglichen PRs sind dokumentiert. Keine Integration vor persönlicher Durchsicht und gesondertem Auftrag.
2. Erst danach gesammelte persönliche/sprachliche Durchsicht vorbereiten. Der [Durchsichtsbericht](../25_P1_PR_Durchsicht.md) enthält die Integrationsfolge und die Ablehnungsempfehlungen für #47/#48. Ohne gesonderten Merge-Auftrag nichts integrieren. P0 und der P1-Auftrag sind bereits bestätigt.
3. Nach der PR-Bearbeitung #67 fortsetzen: Auf eigenem Folgebranch Vertrag für aktuelle Sitzungs-/Zusatzprüfung, Rotation und Abmeldung vor Code festlegen. Organisationsadapter, Gateway und Browser-/TLS-/mTLS-Nachweise bleiben gesondert. Alle bestätigten Interviewantworten bleiben verbindlich.

Vor Fortsetzung AGENTS, CLAUDE, ADR-014/015/019/029/030/031/032/033/034/035/036, KC-006-Plan, Browser-Anmeldevertrag, Anmeldeabschlussvertrag und aktuellen GitHub-Stand lesen.
