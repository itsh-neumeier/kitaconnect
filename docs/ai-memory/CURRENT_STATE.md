# Projektstatus und Fortsetzung

Aktualisiert: 19.10.2026, Europe/Berlin. Grundlage: Spezifikation v5.9.
Ausdrückliche Antwort der Projektleitung: „P0 abnehmen, und P1 freigeben“.
P0 ist abgenommen und abgeschlossen. P1 ist beauftragt. Diese Entscheidungen nicht erneut abfragen.

## Arbeitsstand

- issue_id: [KC-004B / #51](https://github.com/itsh-neumeier/kitaconnect/issues/51), Rust-/SQLx-Anbindung.
- branch: codex/kc-004b-sqlx-adapter auf PR #54 / 16cd295.
- pull_request: [PR #55](https://github.com/itsh-neumeier/kitaconnect/pull/55), Basis codex/kc-004b-postgres-persistenz.
- head_commit: Implementierung 9a3ca3291b06c653c16ba8eb5e635053e06c9107; Vertrag und ADR-025 zuvor in d3dbb96 versioniert. Folgecommit sichert die Socket-Bereinigung und diesen Checkpoint; aktuellen Kopf mit git rev-parse HEAD prüfen.
- objective: SQLx-Adapter mit echten PostgreSQL-Tests; keine neue Berechtigungsregel oder Laufzeitfreigabe.
- completed: Adapter für Anlage und Lesen, gebundene Parameter und exakte u128-Kennungen, typisierte Fehler, callergeführte Transaktionen; Testläufer um Rust-Prüfung vor/nach Neustart erweitert.
- remaining: Linux-Build, Clippy und echte Adaptertests für PR #55 vollständig prüfen; Fehler beheben.
- migration_state: Migration 0001 und produktionsbezogene NOLOGIN-Provisionierung unverändert. Nur im isolierten Testcontainer entsteht eine kurzlebige Loginrolle ohne Passwort.
- known_failures: Lokal scheitert cargo check an fehlendem link.exe für Abhängigkeits-Buildskripte. Docker/psql fehlen ebenfalls. Ausführbare Nachweise müssen aus Linux-CI kommen.
- dependencies: PR #53 und #54 bleiben offen und grün. Neuer Folge-PR gegen codex/kc-004b-postgres-persistenz. Keine ungefragte Integration; nach Basis-Merges retargeten und neu prüfen.

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

- Vorgänger PR #54, 16cd295: alle fünf Pflichtprüfungen erfolgreich, einschließlich echter PostgreSQL-Migrations-/Rechte-/Neustarttests.
- Neu: SQLx 0.9.0 und Tokio 1.54.0 anhand offizieller Paketmetadaten geprüft; Versionen exakt festgelegt, Cargo.lock aktualisiert.
- Aktueller lokaler RustSec-Abgleich mit aktualisierter Datenbank und Paketindex erfolgreich: 155 Lockfile-Pakete, keine blockierenden Befunde.
  Advisory-DB: 999ec89d5d3f10f20be14750ec8c2d8b44e0ac9b.
- Lokal erfolgreich: Rustfmt, 40 Node-Tests, JavaScript-Syntax, Struktur-/Verweisprüfung, Actionlint und git diff --check.
- Adapter-Build/Tests noch nicht erfolgreich ausgeführt. cargo check scheitert am fehlenden Windows-C++-Linker, nicht als bestandene Prüfung ausweisen.
- Der echte PostgreSQL-Test läuft weiterhin im Pflichtstatus rust. Ein temporärer Unix-Socket ermöglicht die Rust-Verbindung ohne Container-Netzwerk oder Host-Port.
- Runtime-RLS bleibt ohne erlaubende Policy. Aktuelle Benutzerrechteprüfung und Dienstfreigabe bleiben #52 vorbehalten.
- Vor Commit Gitleaks auf vorgemerkten Änderungen ausführen; neue CI-Nachweise im Folge-PR nachführen.

1. Aktuellen Kopf von PR #55 und Linux-CI vollständig prüfen; Build-/Testfehler beheben.
2. #51 mit Adapter- und Datenbanknachweisen aktualisieren. Erst nach Integration und vollständigen Kriterien abschließen.
3. KC-005 mit dem Organisationsmodell abstimmen; anschließend Identitäts-/Policy-Grundlagen für #52 bearbeiten. Keine Fachrechte erfinden.

Vor Fortsetzung AGENTS, CLAUDE, ADR-014/019/023/024/025, Adaptervertrag und aktuellen GitHub-Stand lesen.
