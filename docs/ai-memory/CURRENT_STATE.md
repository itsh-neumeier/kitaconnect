# Projektstatus und Fortsetzung

Aktualisiert: 19.10.2026, Europe/Berlin. Grundlage: Spezifikation v5.9.
Ausdrückliche Antwort der Projektleitung: „P0 abnehmen, und P1 freigeben“.
P0 ist abgenommen und abgeschlossen. P1 ist beauftragt. Diese Entscheidungen nicht erneut abfragen.

## Arbeitsstand

- issue_id: [KC-004B / #51](https://github.com/itsh-neumeier/kitaconnect/issues/51), SQL-Persistenz und echte Datenbanktests.
- branch: codex/kc-004b-postgres-persistenz; baut auf PR #53 / 5d42ef8 auf.
- head_commit: Speichervertrag und ADR-024 zuerst in a209a25 versioniert. Implementierungscommit folgt; aktuellen Kopf mit git rev-parse HEAD prüfen.
- objective: Rollen, transaktionssichere Erstmigration und synthetische PostgreSQL-Tests; keine vorgezogene Benutzerrechtefreigabe.
- completed: SQL-Speichervertrag, ADR-024, getrennte NOLOGIN-Rollen, restriktives Schema, ID-Domäne und Referenztabellen; Testläufer für echten Server-Neustart, Rechte und Rollback.
- remaining: Linux-CI für Datenbanktests ausführen und Fehler beheben. Rust-/SQLx-Anbindung bleibt als Folgeschritt von #51 offen.
- migration_state: Versionierte Erstmigration und gesonderte Provisionierung hinzugefügt. Noch keine produktive Datenbank oder Laufzeitidentität angelegt.
- known_failures: Lokal fehlen Docker und psql sowie der Windows-C++-Linker. Echte PostgreSQL-Ausführung wird deshalb in der Linux-CI geprüft; noch kein Datenbankerfolg behauptet.
- dependencies: PR #53 offen und vollständig grün. Folge-PR gegen dessen Branch öffnen; keinen ungefragten Merge ausführen. Nach Integration Basis auf main umstellen und erneut prüfen.

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

## Prüfung und nächste drei Aktionen für #51

- Lokal: JavaScript-Syntax, Struktur-/Verweisprüfung, Actionlint und git diff --check erfolgreich.
- PostgreSQL-Testimage 18.6-bookworm mit Digest afc7e2d441324c0388fa80c3d24f733b4194a4eb7f47dd8ee2b08eb1a24a647c aus offizieller Registry geprüft.
- Testcontainer ohne Netzwerk und Host-Port; lokale Testanmeldung nur im kurzlebigen Testcontainer.
- Pflichtprüfung rust umfasst jetzt auch Datenbanktests. Alle bisherigen Pflichtstatus bleiben unverändert geschützt.
- Runtime besitzt keine erlaubende RLS-Policy. Aktuelle Benutzerrechteprüfung bleibt #52 vorbehalten.
- Vor Commit Gitleaks auf vorgemerkten Änderungen ausführen; CI-Ergebnis im eigenen PR ergänzen.

1. Folge-PR zu #51 öffnen und echte Datenbanktests in CI ausführen; Fehler beheben.
2. Rechte-/Migrationsnachweise dokumentieren. #51 bis zur Rust-Anbindung und vollständigen Abnahme offen lassen.
3. SQLx-Version und Sicherheitsmeldungen für Rust-Persistenz prüfen; danach kleine Adapter- und Datenbanktests ergänzen.

Vor Fortsetzung AGENTS, CLAUDE, ADR-014/019/023/024, Speichervertrag und aktuellen GitHub-Stand lesen.
