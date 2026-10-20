# Projektstatus und Fortsetzung

Aktualisiert: 20.10.2026, Europe/Berlin. Grundlage: Spezifikation v5.9.
Ausdrückliche Antwort der Projektleitung: „P0 abnehmen, und P1 freigeben“.
P0 ist abgenommen und abgeschlossen. P1 ist beauftragt. Diese Entscheidungen nicht erneut abfragen.

## Arbeitsstand

- issue_id: [KC-005A / #56](https://github.com/itsh-neumeier/kitaconnect/issues/56), Teil von KC-005 / #5.
- branch: codex/kc-005a-schema-provisionierung auf PR #55 / a8db9ca.
- head_commit: Wartungsvertrag und ADR-026 in 0011c36 vor Implementierung versioniert; Implementierungscommit folgt. Aktuellen Kopf mit git rev-parse HEAD prüfen.
- objective: Leere Einrichtungsschemas mit Registry atomar provisionieren und prüfen; keine Laufzeitfreigabe.
- completed: Neue NOLOGIN-Wartungs-/Schemaeigentümerrollen, Migration 0002, Rust-Wartungs-API und echte PostgreSQL-Testfälle vorbereitet. Backlog bildet Grundmodelle und Gesamtabnahmen ohne verdeckten Zyklus ab.
- remaining: Neue Linux-CI und Datenbanktests ausführen, Fehler beheben, eigenen Folge-PR gegen #55 öffnen.
- migration_state: Neue gesonderte Rollen-/Datenbank-Provisionierung und Migration 0002 nach 0001. Keine produktive Datenbank oder Loginidentität angelegt.
- known_failures: Keine lokalen Syntax-/Strukturfehler bekannt. Windows-C++-Linker und Docker/psql fehlen; ausführbare Nachweise müssen aus Linux-CI kommen.
- dependencies: #53, #54, #55 offen und grün. Keine ungefragte Integration; Folge-PR gegen codex/kc-004b-sqlx-adapter. Nach Basisintegration retargeten und erneut prüfen.

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
- Rustfmt vor Commit prüfen. Gitleaks auf vorgemerkten Änderungen ausführen. Neue Rust-/PostgreSQL-Tests sind noch nicht erfolgreich ausgeführt.
- Neue Migration verändert keine vorhandene Datei 0001. Registry-/Kennungslese-Policies erlauben ausschließlich der Wartungsrolle Metadatenzugriff.
- Bestehende Runtime-Rolle erhält keine Registry- oder Einrichtungsschema-Rechte. Keine Fachobjekte, Personalbereiche oder Wahldatenbanken angelegt.

## Nächste drei Aktionen

1. Folge-PR zu #56 öffnen und Linux-CI vollständig prüfen; Fehler beheben.
2. Erfolgreiche Rollen-, Registry-, Rollback- und Neustartnachweise in Issue, PR und Checkpoint sichern.
3. KC-005B mit expliziter Dienst-/Objektliste und begründetem RLS-Kontext vorbereiten. Keine selbst gesetzte Kontextvariable allein als Identitätsnachweis zulassen.

Vor Fortsetzung AGENTS, CLAUDE, ADR-014/019/025/026, Wartungsvertrag und aktuellen GitHub-Stand lesen.
