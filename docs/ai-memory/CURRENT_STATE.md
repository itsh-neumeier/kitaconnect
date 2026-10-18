# Projektstatus und Fortsetzung

Aktualisiert: 18.10.2026, Europe/Berlin. Grundlage: Spezifikation v5.9.
Die Projektleitung hat ausdrücklich geantwortet: „P0 abnehmen, und P1 freigeben“.
P0 ist abgenommen. P1 ist beauftragt. Diese Entscheidungen nicht erneut abfragen.

## Arbeitsstand

- issue_id: [KC-038 / #38](https://github.com/itsh-neumeier/kitaconnect/issues/38), P0-Abschluss; danach KC-004 / #4.
- branch: codex/p0-abnahmevorbereitung, [PR #49](https://github.com/itsh-neumeier/kitaconnect/pull/49).
- head_commit: geprüfter Vorläufer 6e2e5db7efc00fafa21e6f1cbe9da7aac238e0c4; der Abnahme-Commit folgt. Aktuellen Kopf mit git rev-parse HEAD prüfen.
- objective: Abnahmeentscheidung sichern, PR #49 mit grüner CI integrieren, KC-004 in kleine P1-Arbeitspakete zerlegen und beginnen.
- completed: PRs #39 bis #44 integriert; KC-001, KC-002, KC-003, KC-036, KC-037 geschlossen. P0-Abnahme und gesonderter P1-Auftrag ausdrücklich erteilt.
- remaining: Abschlusscommit prüfen und integrieren; KC-038 mit Integrationsnachweis schließen. KC-004 beginnen.
- migration_state: Keine Datenbankmigrationen, Laufzeitkonten oder Zertifikate angelegt.
- known_failures: Keine bekannten fehlgeschlagenen Pflichtprüfungen. Windows-Rust benötigt zusätzliche C++-Buildwerkzeuge; Linux-CI liefert die Build-Nachweise.

## Bestätigte Entscheidungen und Sicherheitsumfang

- ADR-014 bis ADR-017 durch „1A, 2A, 3A, 4A“ bestätigt. Nicht erneut abfragen.
- Lastprofil: eine Einrichtung, 50 Sitzungen, 10 Anfragen pro Sekunde, 80/20 Lesen/Schreiben,
  p95 500/1.000 ms, unter 1 % Fehler, 4 vCPU, 8 GB RAM, 100 GB SSD, 5 Minuten Aufwärmen, 30 Minuten Messung.
  Dies sind Ziele. Medien gesondert messen; repräsentative Messungen stehen aus.
- CODEOWNER und benannter Sicherheits-/Datenschutzprüfer: @itsh-neumeier Timo Neumeier.
- Rechteinhaber, juristische Lizenzprüfung und unabhängige Wahlprüfung bleiben ausdrücklich offen.
  Die Sperren und Fristen aus [KC-037](../17_P0_Entscheidungsregister.md) gelten weiter.
- F-001: Server prüft Berechtigung vor Speichern. npm audit lokal und in CI erlaubt.
- Basisregelsatz 25341109: keine Ausnahmen von fünf Pflichtprüfungen, aktueller Basis, PR-Zwang und Lösch-/Force-Push-Schutz.
- Review-Regelsatz 25346711: bestätigte Konto-Ausnahme nur User 77058548 / pull_request.
  Nutzung nur für eigene PRs bleibt eine dokumentierte Regel; technische Grenze ist ausdrücklich akzeptiert.
- Keine Echtdaten, Secrets oder Veröffentlichung. Keine juristische oder unabhängige Fachfreigabe aus der P0-Abnahme ableiten.
- Dependabot-PRs #45 bis #48 bleiben unberührt; Hauptversionswechsel benötigen Prüfung.

## Tests und Nachweise

- Main 68f39f1: [P0 CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39142292618)
  und [P0 Security](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39142292652) erfolgreich.
- PR #49, 6e2e5db: [P0 CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39154982419)
  und [P0 Security](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39154982417) erfolgreich.
- 40 Tests in CI erfolgreich. Strukturprüfung: 38 Issues und 33 Dokumente. Beide Regelpläne validiert.
- Befehle für den Abnahme-Commit: node scripts/check-project.mjs, node scripts/check-github-rules.mjs,
  git diff --check; Gitleaks auf vorgemerkten Änderungen. Ergebnis im PR nachführen.
- [Integrationsbericht](../21_P0_Integration_und_Abnahmevorbereitung.md) und
  [maschinenlesbarer Nachweis](P0_INTEGRATION_STATE.json) dokumentieren die Merges #39 bis #44.

## Nächste drei Aktionen

1. PR #49 mit aktuellem Kopf und fünf grünen Pflichtprüfungen integrieren; Main-CI nachprüfen.
2. Issue #38 abschließen und KC-004 in kleine Issues mit Verträgen und negativen Tests zerlegen.
3. Erstes KC-004-Arbeitspaket auf eigenem Branch umsetzen. Kein vollständiges Modul auf einmal implementieren.

Vor Fortsetzung AGENTS, CLAUDE, aktuelle ADRs und GitHub-Stand lesen. Bestätigte Freigaben nicht erneut abfragen.
