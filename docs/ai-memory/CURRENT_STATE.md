# Projektstatus und Fortsetzung

Aktualisiert: 20.10.2026, Europe/Berlin. Grundlage: Spezifikation v5.9.
Ausdrückliche Antwort der Projektleitung: „P0 abnehmen, und P1 freigeben“.
P0 ist abgenommen und abgeschlossen. P1 ist beauftragt. Diese Entscheidungen nicht erneut abfragen.

## Arbeitsstand

- issue_id: [KC-005B / #57](https://github.com/itsh-neumeier/kitaconnect/issues/57), Teil von KC-005 / #5.
- branch: codex/kc-005b-dienstrechte-rls auf PR #59 / 87cebbb6d9c6382bcfb1e382a731e9f6b4d4645c.
- pull_request: [PR #60](https://github.com/itsh-neumeier/kitaconnect/pull/60), Basis codex/kc-005a-schema-provisionierung.
- head_commit: geprüfter Implementierungsstand aedb4a075e39bacde800ae7e80c8a336b28df711. Vertrag und ADR-027 zuvor in 8af2253 versioniert. Dieser reine Dokumentationscheckpoint folgt gesondert; aktuellen Kopf und zugehörige CI im PR prüfen.
- objective: Dienstbezogene Objektrechte und FORCE RLS mit zwei Diensten und zwei Einrichtungen im Testcontainer nachweisen.
- completed: Festes SQL-Testartefakt und Rust-/SQLx-Test eingebaut; explizite Ausführung vor und nach Server-Neustart im vorhandenen Pflichtprüfschritt.
- remaining: Persönliche Durchsicht und gesonderter Integrationsauftrag. #57 bleibt bis Integration offen. Nächstes Arbeitspaket ist KC-005C / #58.
- migration_state: Keine neue ausgelieferte Migration. Neues Artefakt liegt ausschließlich unter database/tests/.
- known_failures: Keine bekannten Fehler; beide CI-Läufe von aedb4a0 haben fünf erfolgreiche Pflichtprüfungen. Docker und Windows-Linker fehlen lokal; ausführbare Datenbanknachweise stammen aus Linux-CI.
- dependencies: #53, #54, #55 und #59 offen, grün und zur Durchsicht bereit. Keine ungefragte Integration.
- security_notes: Test-Loginrollen und Testpolicies gelten nur im isolierten Container. Produktive Identitäts-/Poolbindung bleibt offen; vorhandene Runtime unverändert gesperrt.

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

## Nächste drei Aktionen

1. KC-005C / #58 vorbereiten: Vertrag vor Implementierung, wiederverwendete Verbindungen nach Erfolg/Fehler/Abbruch und konkurrierende Wartungsabläufe prüfen.
2. Offene Antwort zur produktiven Identitäts-/Poolbindung berücksichtigen. Keine produktive erlaubende RLS ohne freigegebenen Identitätsnachweis. Synthetische Tests können unabhängig fortgesetzt werden.
3. PR #60 persönlich prüfen lassen. Integrationsfolge #53, #54, #55, #59, #60; nach Basisintegration neu prüfen. Keine ungefragte Integration.

Vor Fortsetzung AGENTS, CLAUDE, ADR-014/019/025/026/027, Testvertrag und aktuellen GitHub-Stand lesen.
