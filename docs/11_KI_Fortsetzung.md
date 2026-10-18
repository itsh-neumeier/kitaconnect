# KI-Entwicklungs- und Fortsetzungsanleitung

**KitaConnect Spezifikation v5.9 · 17.10.2026 · Konsolidierter Planungsstand**

> Status: Anforderungsspezifikation, keine implementierte oder rechtsverbindlich geprüfte Software. Die neuesten ausdrücklichen Interviewentscheidungen gelten.

## Prinzip
GitHub ist die einzige verbindliche Quelle für Code, Issues, PRs, Entscheidungen und KI-Checkpoints. Codex und Claude sind austauschbar. Usage-Limits werden nicht umgangen: Arbeit wird so zerlegt und dokumentiert, dass eine neue Sitzung nahtlos fortsetzt.

## Start jeder Sitzung
1. `AGENTS.md`, `CLAUDE.md`, `docs/ai-memory/CURRENT_STATE.md`, Architekturentscheidungen und relevante Issues lesen.
2. `git status`, Branch, offene PRs, letzte Commits, CI-Ergebnisse, migrationsbezogene Änderungen prüfen.
3. Nur ein kleines Issue auswählen, Scope/Tests/Abhängigkeiten verifizieren.
4. Vor Änderungen Ist-Zustand und erwartetes Ergebnis festhalten.

## Während der Arbeit
Kleine Commits; API-Contract zuerst; niemals ungetestete Änderung in Main; Tests fortlaufend; bei Scope-Änderung Issue aktualisieren. Keine Geheimnisse oder produktiven Kinderdaten in Prompts, Commits oder Checkpoints.

## Vor Kontext-/Usage-Limit
Commit/Branch sichern; `CURRENT_STATE.md` mit Issue, Branch, Commit, fertig, offen, Tests, Fehlern, Migrationen und nächsten drei Aktionen aktualisieren; Issue/PR kommentieren. Keine unbegründete Erfolgsmeldung bei fehlenden Tests.

## Neustart-Prompt
```text
Arbeite am GitHub-Repository kitaconnect weiter. Lies zuerst AGENTS.md, CLAUDE.md,
docs/ai-memory/CURRENT_STATE.md, die relevanten ADRs, Issues und PRs.
Prüfe Branch, git status, Commits, CI, offene Migrationen und Tests.
Fahre mit dem im Checkpoint genannten Issue fort; wiederhole keine erledigten
Arbeiten. Setze die Akzeptanzkriterien um, führe Tests aus und aktualisiere
Checkpoint, Issue und PR. Halte bei unklarer Sicherheits-/Fachentscheidung an.
```

## Handoff-Schema
`issue_id`, `branch`, `head_commit`, `objective`, `completed`, `remaining`, `test_commands`, `test_results`, `known_failures`, `migration_state`, `security_notes`, `next_steps`, `updated_at`.


## Dokumentationssprache (Ergänzung v5.9)
Für dieses Dokument gilt die deutsche Projektrichtlinie `13_Dokumentationsrichtlinie_DE_ASD_STE100.md` mit dem verbindlichen Fachwortverzeichnis `14_Fachwortverzeichnis_DE.md`. Die Richtlinie orientiert sich an ASD-STE100, stellt aber **keine offizielle deutsche ASD-STE100-Konformität** dar.
