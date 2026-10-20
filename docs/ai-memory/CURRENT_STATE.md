# Projektstatus und Fortsetzung

Aktualisiert: 20.10.2026, Europe/Berlin. Grundlage: Spezifikation v5.9.
Ausdrückliche Antwort der Projektleitung: „P0 abnehmen, und P1 freigeben“.
P0 ist abgenommen und abgeschlossen. P1 ist beauftragt. Diese Entscheidungen nicht erneut abfragen.

## Arbeitsstand

- issue_id: [KC-006A / #62](https://github.com/itsh-neumeier/kitaconnect/issues/62), Teil von KC-006 / #6.
- branch: codex/kc-006a-sitzungsmodell auf PR #61 / 4fa55b0ea37bd9e359dd7ccd82a5dc11ed2a0c81.
- pull_request: [PR #68](https://github.com/itsh-neumeier/kitaconnect/pull/68), Basis codex/kc-005c-pool-wartung.
- head_commit: geprüfte Implementierung 5abb46e6345e4d0e5417ead35e9f96d12deea947. Vertrag und ADR-029 zuvor in 85fed18 versioniert. Dieser Dokumentationscheckpoint folgt gesondert; aktuellen Kopf und CI im PR prüfen.
- objective: KC-006 in kleine Pakete zerlegen und nur interne Kennungen sowie lokale Sitzungszustände ohne Anmeldung implementieren.
- completed: Issues #62–#67 mit Abhängigkeiten und Kriterien angelegt. Rust-Bibliothek identity-access ohne externe Abhängigkeiten; elf Modelltests und negativer Typvertragstest in Linux-CI erfolgreich.
- remaining: Persönliche/sprachliche Durchsicht und gesonderter Integrationsauftrag. #62 bleibt bis Integration offen. Als Nächstes KC-006B / #63 mit Speichervertrag und atomaren PostgreSQL-Nachweisen vorbereiten.
- migration_state: Keine neue Migration, Datenbankrolle oder Laufzeitfreigabe. Cargo.lock ergänzt nur das neue Workspace-Paket.
- known_failures: Keine bekannten Fehler; fünf Pflichtprüfungen des ersten vollständigen CI-Laufs erfolgreich. Windows-Linker und Docker fehlen lokal.
- dependencies: #53, #54, #55, #59, #60 und #61 offen und grün. Keine ungefragte Integration.
- security_notes: Datensatzkennungen sind keine Sitzungsnachweise; lokaler Modellzustand ist keine aktuelle Authentisierungsentscheidung. Drei neue Sicherheitsvorschläge sowie produktive Identitäts-/Poolbindung sind angefragt und offen.

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

## Nächste drei Aktionen

1. KC-006B / #63 mit begrenztem Speichervertrag vorbereiten. Aktuelle Abfrage, atomaren Widerruf und Konkurrenzfälle gegen PostgreSQL benennen; keine vollständige Anmeldung in einem PR.
2. Antworten zu MFA, Sitzungswerten, Wiederherstellung und produktiver Identitäts-/Poolbindung berücksichtigen. Ohne ausdrückliche Entscheidung keine abhängigen Freigaben oder Vorgabewerte einbauen.
3. PR #68 persönlich prüfen lassen. Integrationsfolge #53, #54, #55, #59, #60, #61, #68; nach Basisintegration neu prüfen. Keine ungefragte Integration.

Vor Fortsetzung AGENTS, CLAUDE, ADR-014/019/029, KC-006-Plan, Modellvertrag und aktuellen GitHub-Stand lesen.
