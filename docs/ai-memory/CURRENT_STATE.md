# Projektstatus und Fortsetzung

Aktualisiert: 18.10.2026, Europe/Berlin. Grundlage: Spezifikation v5.9.
Status: Die Projektleitung hat die vier Architekturvarianten für KC-002 ausdrücklich entschieden.
Die Dokumentation ist zur Prüfung in PR #40 vorbereitet. P0 ist insgesamt noch nicht abgenommen.
P1 bis P9 sind nicht implementiert und haben keinen Auftrag.

## Arbeitsstand

- `issue_id`: [KC-002 / #2](https://github.com/itsh-neumeier/kitaconnect/issues/2).
- `branch`: `codex/p0-service-boundaries`.
- `head_commit`: Ausgangsstand dieser Entscheidungsfortsetzung ist `6cc722e6de6ef29cf1437f864e5a039b32546261`.
  Die bestätigten Entscheidungen folgen als eigener Dokumentationscommit. Aktuellen Kopf mit `git rev-parse HEAD` prüfen.
- `objective`: Antwort `1A, 2A, 3A, 4A` dokumentieren und die KC-002-Dienstgrenzen entsprechend konkretisieren.
- `migration_state`: Keine Datenbankrollen, Tabellen, Migrationen oder Fachdatenspeicherung angelegt.
- `pull_request`: [PR #40](https://github.com/itsh-neumeier/kitaconnect/pull/40).
- `base`: `codex/p0-project-initialization`; [PR #39](https://github.com/itsh-neumeier/kitaconnect/pull/39) bleibt Voraussetzung.

## Bestätigte Entscheidungen

Quelle: ausdrückliche Antwort der Projektleitung `1A, 2A, 3A, 4A` im Projektchat am 18.10.2026.

| Auswahl | Entscheidung | Nachweis |
|---|---|---|
| `1A` | Ein PostgreSQL-Schema je Einrichtung mit getrennten Dienstrechten und Migrationsrollen | [ADR-014](../architecture/ADR-014_Datenhoheit_und_Migrationen.md) |
| `2A` | Eigenständiges Rust-Gateway; Fachdienste prüfen Rechte zusätzlich selbst | [ADR-015](../architecture/ADR-015_Gateway_Grenze.md) |
| `3A` | Gesonderter Personalbereich je Träger; eigene Bereiche für trägerlose Einrichtungen; nur erlaubte Einsatzdaten für Einrichtungen | [ADR-016](../architecture/ADR-016_Personalbereich.md) |
| `4A` | Getrennte Datenbanken und Dienstidentitäten für Wahlberechtigung und Stimme; unabhängige Protokollprüfung vor Implementierung | [ADR-017](../architecture/ADR-017_Wahltrennung.md) |

O-001, O-002 sowie O-001-HR und O-001-WAHL sind damit auf Architekturebene entschieden.
Die unabhängige Prüfung des Wahlprotokolls bleibt verpflichtend und ist noch nicht durchgeführt.
Sie wird nicht durch die Antwort, Datenbanktrennung oder grüne Dokumentationsprüfungen ersetzt.

## Erledigt

1. Anweisungen, Checkpoint, ADRs, Dokumentationsrichtlinie sowie Issue #2 und PR #40 geprüft.
   Arbeitsbaum war sauber; Basis-CI für `6cc722e` war erfolgreich.
2. ADR-014 und ADR-015 von Vorschlägen auf bestätigte Entscheidungen umgestellt.
3. ADR-016 für Datenhoheit, Rechte und Migrationen der Personalbereiche ergänzt.
4. ADR-017 für Wahlgrenzen und verpflichtende unabhängige Prüfung ergänzt.
5. [Dienstmatrix](../architecture/KC-002_Dienstgrenzen.md), Architekturübersicht, Datenmodell und P0-Plan abgeglichen.
6. Dienstrollen und Migrationszuständigkeit erhalten ihre Trennung. Gateway und Frontend haben keine Fachdatenbankrolle.
7. Negative Testanforderungen für Personal- und Wahlgrenzen ergänzt. Sie sind noch keine ausgeführten Diensttests.
8. Entscheidungsliste und Fortsetzungsanleitung aktualisiert. GitHub-Issue und PR führen den Review-Stand.

## Prüfungen

| Prüfung | Ergebnis |
|---|---|
| `node scripts/check-project.mjs` | 38 Issues, 24 Dokumente, lokale Verweise und synthetische Daten geprüft |
| `node --test scripts/*.test.mjs` | Vier bestehende Tests erfolgreich |
| `git diff --check` | Erfolgreich |
| Gitleaks auf vorgemerktem Diff | Vollständiger Entscheidungsdiff ohne Befund; finalen Commit zusätzlich in CI prüfen |
| Ausgangscommit `6cc722e` | [P0 CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39114152989) und [P0 Security](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39114153001) erfolgreich |

Nur Dokumentation geändert. Lokale Frontend-/Rust-Builds werden deshalb nicht wiederholt.
Der neue Commit löst eigene vollständige CI-Prüfungen aus. Vor Merge MUSS der aktuelle Commit grün sein.
Die früher fehlenden Windows-Buildwerkzeuge sind unverändert; Rust-Build und Tests sind in Linux-CI nachgewiesen.
Es gibt keine bekannten neuen Testfehler. Die Implementierungsnachweise folgen erst in späteren Phasen.

## Verbleibende Aufgaben

- Review und Integration von PR #39, danach PR #40 auf `main` umstellen und erneut prüfen.
- KC-002 erst nach Integration schließen. Architekturentscheidungen allein sind keine P0-Gesamtabnahme.
- KC-003: Test- und Sicherheitsgrundlagen weiterführen; vor externen Rust-Abhängigkeiten `cargo audit` einrichten.
- O-003 bis O-008 über KC-037 klären: Dienstidentitäten, Zustellung, Lastwerte, Versionen, Lizenz und Fachprüfer.
- KC-038: Zuständige CODEOWNERS und nachgewiesenen Branchschutz klären.
- KC-036: Bestandsdokumentation sprachlich prüfen. Unabhängige sprachliche Prüfung vor Merge bleibt erforderlich.

Die frühere Lastangabe `1` bezeichnet das Referenzprofil einer Einrichtung. Weitere Lastwerte fehlen.
Die frühere CODEOWNER-Rückfrage ist nicht beantwortet. Bisheriger Branchschutz bleibt wegen HTTP 403 unbekannt.
Die neue Variantenentscheidung beantwortet diese anderen Fragen nicht.

## Nächste drei Aktionen

1. Aktuelle CI und Dokumentationsreview von PR #40 prüfen; Integration in Abhängigkeit von PR #39 vorbereiten.
2. [KC-003 / #3](https://github.com/itsh-neumeier/kitaconnect/issues/3) als nächstes P0-Arbeitspaket auswählen.
3. KC-037/KC-038 weiter klären. Vor Umsetzung des Wahlprotokolls die unabhängige Prüfung aus KC-025 durchführen lassen.

## Sicherheitsumfang und Fortsetzung

Nur Architektur- und Projektdokumentation. Keine Echtdaten, Secrets, Laufzeitdienste oder Migrationen.
Die bestehende Erlaubnis für npm audit lokal und in GitHub Actions gilt weiter.
GitHub ist die verbindliche Quelle. Die JSON-Dateien unter `backlog/` sind Importnachweise.
Lesen Sie beim Neustart AGENTS, CLAUDE, diesen Checkpoint, die ADRs und den aktuellen GitHub-Stand.
Wiederholen Sie die beantworteten Variantenfragen nicht. Beginnen Sie keine weitere Phase ohne Auftrag.
