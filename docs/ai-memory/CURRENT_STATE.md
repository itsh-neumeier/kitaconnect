# Projektstatus und Fortsetzung

Aktualisiert: 20.10.2026, Europe/Berlin. Grundlage: Spezifikation v5.9.
Ausdrückliche Antwort der Projektleitung: „P0 abnehmen, und P1 freigeben“.
P0 ist abgenommen und abgeschlossen. P1 ist beauftragt. Diese Entscheidungen nicht erneut abfragen.

## Arbeitsstand

- issue_id: [KC-006B / #63](https://github.com/itsh-neumeier/kitaconnect/issues/63), Teil von KC-006 / #6.
- branch: codex/kc-006b-sitzungsspeicher auf PR #68 / c7711e8a7eba7941bdf270e5a5429e1b3059e29e.
- pull_request: [PR #69](https://github.com/itsh-neumeier/kitaconnect/pull/69), Basis codex/kc-006a-sitzungsmodell.
- head_commit: geprüfte Implementierung 6d7425ea316e533496f1f5597c0845fda94c352d. Vertrag und ADR-030 zuvor in 2487f86 versioniert. Dieser Dokumentationscheckpoint folgt gesondert; aktuellen Kopf und CI im PR prüfen.
- objective: Getrennter Plattformbestand mit abgeleiteten Sitzungsnachweisen, aktueller Prüfung und atomarem Widerruf.
- completed: NOLOGIN-Rollen, Migration 0001, Betriebssystem-Zufallsnachweise, SHA-256-Ableitung und SQLx-Adapter eingebaut. Echte Datenbanktests einschließlich beobachteter Zeilensperrkonkurrenz und Neustart erfolgreich.
- remaining: Persönliche/sprachliche Durchsicht und gesonderter Integrationsauftrag. #63 bleibt bis Integration offen. Als Nächstes KC-006C / #64 mit begrenztem Argon2id-Vertrag vorbereiten.
- migration_state: Neue dienstbezogene Provisionierung und identity-access-Migration 0001. Keine vorhandene Organisationsmigration geändert. Runtime bleibt durch FORCE RLS ohne erlaubende Policy gesperrt.
- known_failures: Keine bekannten Fehler; fünf Pflichtprüfungen des ersten vollständigen CI-Laufs erfolgreich. Windows-Linker und Docker fehlen lokal.
- dependencies: #53, #54, #55, #59, #60, #61 und #68 offen und grün. Keine ungefragte Integration.
- security_notes: Keine Anmeldung, Cookies, Mitgliedschaft oder fachliche Rechtefreigabe. Positive Speichertests verwenden nur die Test-Betriebsidentität. Offene MFA-, Sitzungsdauer-, Wiederherstellungs- und produktive Rollenbindungsfragen bleiben unbeantwortet.

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

## Nächste drei Aktionen

1. KC-006C / #64 mit begrenztem Vertrag für Argon2id, Bibliotheks-/Parameterprüfung und Anfragemengenbegrenzung vorbereiten. Keine vollständige Anmeldung in einem PR.
2. Antworten zu MFA, Sitzungswerten, Wiederherstellung und produktiver Identitäts-/Poolbindung berücksichtigen. Ohne Antwort keine abhängigen Regeln, Cookies oder Freigaben einbauen.
3. PR #69 persönlich prüfen lassen. Integrationsfolge #53, #54, #55, #59, #60, #61, #68, #69; nach Basisintegration neu prüfen. Keine ungefragte Integration.

Vor Fortsetzung AGENTS, CLAUDE, ADR-014/019/029/030, KC-006-Plan, Speichervertrag und aktuellen GitHub-Stand lesen.
