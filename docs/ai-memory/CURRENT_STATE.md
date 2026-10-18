# Projektstatus und Fortsetzung

Aktualisiert: 18.10.2026, Europe/Berlin. Grundlage: Spezifikation v5.9.
Status: KC-002 besitzt prüfbare Architekturvorschläge. Erforderliche Entscheidungen stehen aus.
P0 ist nicht abgenommen. P1 bis P9 sind nicht implementiert und haben keinen Auftrag.

## Arbeitsstand

- `issue_id`: [KC-002 / #2](https://github.com/itsh-neumeier/kitaconnect/issues/2).
- `branch`: `codex/p0-service-boundaries`.
- `head_commit`: Vorschlagscommit `11776f4`; der nachfolgende Dokumentationscommit ergänzt Quellen und diesen Checkpoint.
  Den aktuellen Branchkopf zeigt `git rev-parse HEAD`.
- `objective`: Dienstgrenzen, Datenhoheit, erlaubte Aufrufe und Architekturvarianten zur Entscheidung vorlegen.
- `migration_state`: Keine Migrationen, Datenbankrollen, Tabellen oder Fachdatenspeicherung vorhanden.
- `pull_request`: [Entwurf #40](https://github.com/itsh-neumeier/kitaconnect/pull/40).
- `base`: `codex/p0-project-initialization`, Commit `1d5066d38b5629070dad0e9eb5719deb2c4fc69b`.
  [Grundgerüst-PR #39](https://github.com/itsh-neumeier/kitaconnect/pull/39) ist weiterhin offen.

## Prüfung beim Wiederbeginn

1. AGENTS, CLAUDE, Checkpoint, offene Entscheidungen, ADRs und relevante Spezifikationsstellen gelesen.
2. Arbeitsbaum war sauber. Der unterbrochene Checkpoint-Commit `1d5066d` war bereits auf GitHub gesichert.
3. PR #39 blieb ein Entwurf. Seine CI- und Sicherheitsworkflows waren für `1d5066d` erfolgreich.
4. KC-002 war offen. Keine Migrationen oder fremden lokalen Änderungen festgestellt.
5. Für KC-002 einen getrennten Branch und einen darauf begrenzten Dokumentations-PR erstellt.

## Erledigt

Das Grundgerüst aus KC-001 bleibt unverändert: Rust-Workspace, SvelteKit/Svelte 5,
TypeScript, Tailwind, Lockfiles, CI, Testdaten und Infrastrukturstruktur.
38 GitHub-Issues bilden den Importstand ab. GitHub bleibt die verbindliche Quelle.

KC-002 ergänzt:

1. [Dienstmatrix](../architecture/KC-002_Dienstgrenzen.md) für alle Dienste aus `docs/02`.
2. Zuordnung der konzeptionellen Entitäten aus `docs/03` und Abgrenzung gemeinsamer Veranstaltungen und Basare.
3. Erlaubte Kommunikationszwecke mit erneuter Rechteprüfung beim empfangenden Dienst.
4. [ADR-014](../architecture/ADR-014_Datenhoheit_und_Migrationen.md) als Vorschlag für Dienstrollen und Migrationszuständigkeit.
5. [ADR-015](../architecture/ADR-015_Gateway_Grenze.md) als Vergleich von Rust-Gateway und SvelteKit-Backend.
6. Negative Testanforderungen für spätere Implementierungsissues. Es sind keine ausgeführten Diensttests.
7. Offene Teilfragen O-001-HR und O-001-WAHL. Keine stillschweigende Entscheidung getroffen.
8. Entscheidung zu den allgemeinen Varianten von ADR-014 und ADR-015 bei der Projektleitung angefragt.

## Prüfungen und Grenzen

| Befehl / Nachweis | Ergebnis |
|---|---|
| `node scripts/check-project.mjs` | 38 Issues, 22 Dokumente, lokale Verweise und synthetische Daten erfolgreich geprüft |
| `node --test scripts/*.test.mjs` | Vier bestehende Tests erfolgreich |
| `git diff --check` | Erfolgreich |
| Gitleaks 8.30.1 auf vorgemerktem Diff | Vorschlagscommit ohne Befund; Checkpoint wird vor Commit erneut geprüft |
| Basiscommit `1d5066d` | P0 CI und P0 Security erfolgreich |
| Vorschlagscommit `11776f4` | P0 CI und P0 Security erfolgreich; alle fünf Prüfschritte grün |
| Quelltext, Abhängigkeiten und Workflows | In KC-002 unverändert; kein zusätzlicher lokaler Frontend-/Rust-Build erforderlich |

Basisnachweise:
[P0 CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39111808331),
[P0 Security](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39111808335).
Für jeden neuen Commit lösen die vorhandenen Workflows eigene Prüfungen aus.
Vorschlagsnachweise:
[P0 CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39114052078),
[P0 Security](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39114052098).
Vor Merge MUSS der aktuelle PR-Commit grün sein.
Die früheren lokalen Rust-Tests benötigten den fehlenden Windows-Linker `link.exe`.
Rust-Build und Tests waren in der Linux-CI erfolgreich.

## Offene Entscheidungen und Fehler

Es sind keine Codefehler festgestellt. Die wesentliche Einschränkung ist die ausstehende Entscheidung.

- O-001: Gemeinsames Einrichtungsschema mit getrennten Dienstrechten oder getrennte Dienstdatenbanken.
- O-002: Eigenständiges Rust-Gateway oder SvelteKit als Backend für das Frontend.
- O-001-HR: Trägerweite Personalakten brauchen einen ausdrücklich geprüften Speicher- und Rechtebereich.
- O-001-WAHL: Wahlberechtigung und Stimme brauchen eine unabhängig geprüfte physische und organisatorische Trennung.
- O-003: Konkrete Dienstidentitäten und sichere Weitergabe des Benutzerkontexts fehlen weiterhin.
- O-004 bis O-008 bleiben nach [P0-Plan](../15_P0_Initialisierung.md) offen.
- CODEOWNERS hat keine aktive Zuordnung. Die zuvor gestellte Rückfrage ist unbeantwortet.
- Bisheriger Branchschutz ist unbekannt: Connector-Abfrage lieferte HTTP 403; Regelsatzliste war leer.
- Die Lastprofilangabe `1` wurde als eine Einrichtung dokumentiert. Weitere Lastwerte fehlen.

Die [Entscheidungsliste](DECISIONS_PENDING.md) enthält Antworten und offene Fragen.
Eine ausbleibende Antwort gilt nicht als Zustimmung. Die ADRs bleiben bis zur ausdrücklichen Entscheidung vorgeschlagen.
KC-002 bleibt offen; ein grüner Dokumentationsprüfschritt ersetzt keine Sicherheitsabnahme.

## Sicherheitsumfang

Nur Dokumentation. Keine neuen Laufzeitdienste, Zugangsdaten, Migrationen oder Echtdaten.
Die Projektleitung hat npm audit lokal und in GitHub Actions bereits ausdrücklich erlaubt.
Es übermittelt Paketnamen und Versionen an registry.npmjs.org.
Es gibt weiterhin keine Nachweise für implementierte Mandantenisolation oder geheime Wahlen.
Die deutsche Dokumentation braucht vor Merge eine unabhängige manuelle sprachliche Prüfung.

## Nächste drei Aktionen

1. Antworten zu ADR-014 und ADR-015 prüfen. Entscheidung mit Datum und Quelle dokumentieren.
   Bei fehlender Antwort keine Rollen, Tabellen, APIs oder Migrationen implementieren.
2. O-001-HR und O-001-WAHL mit der zuständigen Fachperson klären.
   Erst danach die betreffenden KC-002-Akzeptanzkriterien als erfüllt markieren.
3. Zuerst PR #39 prüfen und integrieren. Danach PR #40 auf `main` umstellen und erneut prüfen.
   Keine eigenständige Merge-Freigabe aus diesem Checkpoint ableiten.

## Fortsetzungsanleitung

Lesen Sie AGENTS, CLAUDE, diesen Status, die Entscheidungsliste und die KC-002-Vorschläge.
Prüfen Sie Branch, Arbeitsbaum, `git log`, Issue #2 und PRs #39/#40 einschließlich aktueller CI.
Verwenden Sie [github-issues.json](../../backlog/github-issues.json) für die KC-Zuordnung.
Wiederholen Sie das fertige Grundgerüst nicht. Setzen Sie die offene Entscheidung fort.
P1 braucht weiterhin P0-Abnahme und einen gesonderten Auftrag.
