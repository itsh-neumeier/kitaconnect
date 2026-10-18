# Projektstatus und Fortsetzung

Aktualisiert: 18.10.2026, Europe/Berlin. Grundlage: Spezifikation v5.9.
Ausdrückliche Antwort der Projektleitung: „P0 abnehmen, und P1 freigeben“.
P0 ist abgenommen und abgeschlossen. P1 ist beauftragt. Diese Entscheidungen nicht erneut abfragen.

## Arbeitsstand

- issue_id: [KC-004A / #50](https://github.com/itsh-neumeier/kitaconnect/issues/50), Teil von [KC-004 / #4](https://github.com/itsh-neumeier/kitaconnect/issues/4).
- branch: codex/kc-004a-organisationsmodell auf Main eaca2cb56a0e54a732ec17986f7c3ccb854c3986.
- head_commit: Vertrag und ADR-023 in 847517b vor Implementierung versioniert. Implementierungscommit folgt; aktuellen Kopf mit git rev-parse HEAD prüfen.
- objective: Kleines erstes P1-Paket mit internem Rust-Modell, ohne Netzwerk oder Persistenz.
- completed: Modell mit getrennten ID-Typen, optionalem registriertem Träger und unveränderlicher Zuordnung; acht Integrationstests und ein negativer Dokumentationstest vorbereitet.
- remaining: Neue Linux-CI für Tests und Build prüfen; eigenen PR und Issue mit Nachweisen aktualisieren.
- migration_state: Keine Datenbankmigrationen, Laufzeitkonten oder Zertifikate angelegt.
- known_failures: Keine bekannten fachlichen Fehler. Lokaler C++-Linker fehlt; ausführbare Rust-Tests und Build deshalb in Linux-CI prüfen. Windows meldete beim Clippy-Cache Ersatzkopien statt Hardlinks; Befehl erfolgreich.

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
- Struktur-/Verweisprüfung und git diff --check erfolgreich; nach finalem Checkpoint erneut prüfen.
- cargo test --workspace --locked und cargo build --workspace --locked: neue Linux-CI vor Abnahme erforderlich.
- Gitleaks auf vorgemerkten Änderungen vor Commit ausführen. Ergebnis im PR nachführen.

## Nächste drei Aktionen

1. Aktuellen PR und CI für KC-004A prüfen; fehlgeschlagene Prüfungen beheben. Keine ungeprüfte Integration.
2. KC-004A mit konkreten Prüfnachweisen zur Durchsicht bereitstellen. Gesamtissue KC-004 offen lassen.
3. KC-004B mit KC-005 abstimmen: Plattformmigration, Datenbankrechte und reproduzierbare Datenbanktests vorbereiten.
   Vor Aufnahme externer Pakete/Images offizielle Quellen, Versionen und Digests prüfen.

Vor Fortsetzung AGENTS, CLAUDE, ADR-014/019/023, aktuellen PR und GitHub-Stand lesen.
