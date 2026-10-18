# KI-Entwicklungs- und Fortsetzungsanleitung

**KitaConnect Spezifikation v5.9 · 17.10.2026 · Konsolidierter Planungsstand**

> Status: Anforderungsspezifikation, keine implementierte oder rechtsverbindlich geprüfte Software. Die neuesten ausdrücklichen Interviewentscheidungen gelten.

## Prinzip
GitHub ist die einzige verbindliche Quelle für Code, Issues, Pull Requests (PRs), Entscheidungen und KI-Checkpoints.
Ein Checkpoint dokumentiert den Arbeitsstand und die Fortsetzung.
Codex und Claude sind austauschbar. Nutzungsgrenzen werden nicht umgangen.
Die Arbeit wird so zerlegt und dokumentiert, dass eine neue Sitzung sie ohne Wiederholung fortsetzen kann.

## Start jeder Sitzung
1. Lesen Sie `AGENTS.md`, `CLAUDE.md`, `docs/ai-memory/CURRENT_STATE.md`, Architekturentscheidungen und relevante Issues.
2. Prüfen Sie `git status`, Branch, offene PRs, letzte Commits und Ergebnisse der automatisierten Integration (CI).
   Prüfen Sie außerdem migrationsbezogene Änderungen.
3. Wählen Sie nur ein kleines Issue aus. Prüfen Sie Umfang, Tests und Abhängigkeiten.
4. Halten Sie vor Änderungen den Istzustand und das erwartete Ergebnis fest.

## Während der Arbeit
Erstellen Sie kleine Commits. Beschreiben Sie den Dienstvertrag vor seiner Implementierung.
Integrieren Sie keine ungetestete Änderung in `main`. Führen Sie Tests fortlaufend aus.
Aktualisieren Sie das Issue, wenn sich der Umfang ändert.
Verwenden Sie keine Geheimnisse oder produktiven Kinderdaten in Prompts, Commits oder Checkpoints.

## Vor Kontext- oder Nutzungsgrenze

1. Sichern Sie Commit und Branch.
2. Aktualisieren Sie `CURRENT_STATE.md` mit Issue, Branch und Commit.
   Dokumentieren Sie erledigte und offene Aufgaben, Tests, Fehler, Migrationen und die nächsten drei Aktionen.
3. Aktualisieren Sie Issue und PR mit dem aktuellen Stand.

Behaupten Sie bei fehlenden Tests keinen unbelegten Erfolg.

## Neustart-Prompt

Der konkrete P0-Stand steht in [CURRENT_STATE](ai-memory/CURRENT_STATE.md).
Offene Rückfragen stehen in [DECISIONS_PENDING](ai-memory/DECISIONS_PENDING.md).
Die [Issue-Zuordnung](../backlog/github-issues.json) verbindet KC-Kennungen mit GitHub.
P0 ist vor der dokumentierten Abnahme nicht abgeschlossen. P1 braucht einen neuen Auftrag.
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
