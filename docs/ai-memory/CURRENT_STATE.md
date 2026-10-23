# Projektstatus und Fortsetzung

Aktualisiert: 20.10.2026, Europe/Berlin. Grundlage: Spezifikation v5.9.
Ausdrückliche Antwort der Projektleitung: „P0 abnehmen, und P1 freigeben“.
P0 ist abgenommen und abgeschlossen. P1 ist beauftragt. Diese Entscheidungen nicht erneut abfragen.

## Arbeitsstand

- issue_id: [KC-006C / #64](https://github.com/itsh-neumeier/kitaconnect/issues/64), Teil von KC-006 / #6.
- branch: codex/kc-006c-passwortpruefung auf PR #69 / e0e1214fd88e65ebc34c2db81286990388b59571.
- pull_request: [PR #70](https://github.com/itsh-neumeier/kitaconnect/pull/70), Basis codex/kc-006b-sitzungsspeicher.
- head_commit: cd39fe8 mit vollständigem CI-Nachweis einschließlich der Ergänzungen aus 404c04c. Vertrag und ADR-031 zuvor in 3f964e6 versioniert. Dieser Abschlusscheckpoint und eine Klarstellung zur offenen MFA-Regel folgen gesondert; aktuellen Kopf und dessen CI im PR prüfen.
- objective: Begrenzter Argon2id-Passwortfaktor mit dauerhafter Reservierung von Versuchen, ohne Anmeldefreigabe.
- completed: Feste Profile, begrenzte Hasharbeit, einheitliche Fehler, Parameteraktualisierung und Migration 0002 eingebaut. Alle Nachweise einschließlich echter Sperrkonkurrenz und Server-Neustart erfolgreich. Auch Hashänderung ohne Versionswechsel, Eigentümer-RLS und ausbleibende Sitzungserzeugung sind geprüft.
- remaining: CI des Abschlusskopfes im PR prüfen; bei Fehler korrigieren. Persönliche/sprachliche Durchsicht und gesonderter Integrationsauftrag. #64 bleibt bis Integration offen. Danach KC-006D / #65 mit WebAuthn-Vertrag vorbereiten.
- migration_state: Neue identity-access-Migration 0002. Migration 0001 und Organisationsmigrationen bleiben unverändert. FORCE RLS ohne erlaubende Policy sperrt weiterhin die Runtime. Getrennter synthetischer Passwort-Testbestand mit Versionen 1 und 2.
- known_failures: Erste SQLx-Typanforderung im Test in 2ec3983 behoben. Fünf Pflichtprüfungen des ersten vollständigen Laufs erfolgreich. Windows-Linker und Docker fehlen lokal. Keine unbelegten lokalen Rust-/Datenbankerfolge behaupten.
- dependencies: #53, #54, #55, #59, #60, #61, #68 und #69 offen und grün. Keine ungefragte Integration.
- security_notes: Ein Passwortergebnis ist nur ein Faktor. Keine Anmeldung, Cookies, Mitgliedschaft oder fachliche Rechtefreigabe. Positive Speichertests verwenden nur die Test-Betriebsidentität. MFA-, Sitzungsdauer-, Wiederherstellungs- und produktive Rollenbindungsfragen bleiben offen. Produktive Versuchslimits, Prüfschlüsselbindung und Argon2id-Lastmessungen sind vor Laufzeitfreigabe erforderlich.

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

## Nächste drei Aktionen

1. Aktuellen Kopf und CI von PR #70 prüfen. Bei grünen Ergänzungsnachweisen KC-006D / #65 mit WebAuthn-Vertrag und geprüfter Bibliothekswahl beginnen. Keine vollständige Anmeldung in einem PR.
2. Antworten zu MFA, Sitzungswerten, Wiederherstellung und produktiver Identitäts-/Poolbindung berücksichtigen. Vor Laufzeitfreigabe zusätzlich Versuchslimits und Prüfschlüsselbildung klären. Ohne Antwort keine abhängigen Regeln, Cookies oder Freigaben einbauen.
3. PR #70 persönlich prüfen lassen. Integrationsfolge #53, #54, #55, #59, #60, #61, #68, #69, #70; nach Basisintegration neu prüfen. Keine ungefragte Integration.

Vor Fortsetzung AGENTS, CLAUDE, ADR-014/019/029/030/031, KC-006-Plan, Passwortvertrag und aktuellen GitHub-Stand lesen.
