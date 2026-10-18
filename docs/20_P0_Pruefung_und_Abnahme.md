# P0 – Technische Vorprüfung und Abnahmevorbereitung

Stand: 18.10.2026, Europe/Berlin. Grundlage: Spezifikation v5.9.
Geprüfter Gesamtstand: `f4d8983ecf46ef61735ab3984435aaf037430a7f`.
Vergleichsbasis: `a6fae87c40f0e030dbd39913f0dc3f575a2b6c38`.
Status: technische Vorprüfung dokumentiert; persönliche Durchsicht und Integration stehen aus.

## Ergebnis und Umfang

Im unten beschriebenen Umfang wurden keine neuen blockierenden technischen Befunde festgestellt.
Geprüft wurden das Rust-Grundgerüst, sämtliche Frontend-Quelldateien und ihre Konfiguration,
die Prüfskripte und Tests, beide Workflows, die Infrastrukturdateien, Repository-Vorlagen,
Schutzregeln sowie die dokumentierten P0-Abhängigkeiten und Freigaben.
Die Prüfung ordnet vorhandene Nachweise der vorbereiteten Änderung zu.
Sie ist keine unabhängige Fachprüfung und kein Nachweis der Sicherheit noch nicht implementierter Dienste.

Rust enthält nur eine Konsolenausgabe und öffnet keinen Netzwerkport.
Das Frontend zeigt eine interne P0-Hinweisseite.
Compose definiert keine Dienste. Helm enthält keine installierbaren Laufzeitressourcen.
Es gibt keine Fachdatenbankmigration und keine produktive Datenverarbeitung.

## GitHub-Stand je Pull Request

Alle sechs PRs sind offen und als Entwurf gekennzeichnet.
Für jeden aufgeführten Commit sind `structure`, `rust`, `web`, `secrets` und `dependencies` erfolgreich.

| PR | Geprüfter Commit | Derzeitige Basis | GitHub-Nachweis |
|---|---|---|---|
| [#39](https://github.com/itsh-neumeier/kitaconnect/pull/39) | `1d5066d` | `main` | [Build/Tests](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39111808331), [Sicherheitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39111808335) erfolgreich |
| [#40](https://github.com/itsh-neumeier/kitaconnect/pull/40) | `d4de932` | `codex/p0-project-initialization` | [Build/Tests](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39116283113), [Sicherheitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39116283194) erfolgreich |
| [#41](https://github.com/itsh-neumeier/kitaconnect/pull/41) | `ec907b8` | `codex/p0-service-boundaries` | [Build/Tests](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39121874227), [Sicherheitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39121874219) erfolgreich |
| [#42](https://github.com/itsh-neumeier/kitaconnect/pull/42) | `a893f76` | `codex/p0-test-security-foundations` | [Build/Tests](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39122609613), [Sicherheitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39122609604) erfolgreich |
| [#43](https://github.com/itsh-neumeier/kitaconnect/pull/43) | `bdb3f9f` | `codex/p0-decision-register` | [Build/Tests](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39123817480), [Sicherheitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39123817474) erfolgreich |
| [#44](https://github.com/itsh-neumeier/kitaconnect/pull/44) | `f4d8983` | `codex/p0-review-protection` | [Build/Tests](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39138506345), [Sicherheitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39138506397) erfolgreich |

Die PR-Basis ist jeweils die vorherige Arbeitsstufe.
Die aufgeführten Ergebnisse gelten für diese Commits und diese Prüfzeit.
Der folgende Berichtscommit erhält eigene Prüfungen. Ein später geänderter Commit benötigt erneute Nachweise.

## Abgleich mit dem P0-Auftrag

| Anforderung | Ergebnis auf dem vorbereiteten Branch | Grenze |
|---|---|---|
| Fehlende technische Entscheidungen erfassen | O-001 bis O-008 im KC-037-Register; bestätigte ADRs und späteste Prüftermine | Offene juristische und unabhängige Fachfreigaben sind keine erteilten Freigaben |
| Monorepo entwerfen | Rust unter services/crates, Frontend unter apps/web, Verträge unter contracts, Betrieb unter infra | Keine Implementierung von P1 bis P9 |
| Entwicklungsreihenfolge mit Abhängigkeiten | Roadmap, P0-Plan und verfeinerte Issue-Abhängigkeiten | Spätere Module vor Implementierung weiter zerlegen |
| Issues mit Akzeptanzkriterien | 38 zugeordnete GitHub-Issues; P0-Issues #1, #2, #3, #36, #37 und #38 geprüft | Issues bleiben bis zu ihren Review-/Integrationsnachweisen offen |
| Actions, Tests und Sicherheitsprüfungen | Zwei Workflows, fünf Pflichtprüfungen, Lockfiles, 40 bestehende Tests | Quelltext-/Containerprüfungen, Fachtests und Veröffentlichung folgen ihren späteren Phasen |
| Persistentes Gedächtnis | Checkpoint, offene Entscheidungen und API-Momentaufnahme | Vor Fortsetzung GitHub erneut lesen |
| Architekturentscheidungen | ADR-011 bis ADR-022 beziehungsweise zugeordnete Entscheidungsabschnitte | ADR-022 schreibt den ursprünglichen Branchschutz ausdrücklich fort |
| Deutsche Dokumentationsrichtlinie | Unterlagen 01 bis 12 redaktionell geprüft; Fachwortverzeichnis und Prüfbericht | Persönliche manuelle sprachliche Bestätigung fehlt noch |

## Technische Nachweise und Grenzen

- GitHub Actions baut Rust aus Cargo.lock und das Frontend nach `npm ci --ignore-scripts` aus package-lock.json.
  Die Jobs laufen auf frischen Ubuntu-Runnern; es ist kein Wiederverwenden eines lokalen Buildverzeichnisses eingerichtet.
- Die 40 Tests umfassen Backlog-Abhängigkeiten, synthetische Testdaten und 26 negative Branchschutzfälle.
- Gitleaks prüft die Git-Historie. npm audit und cargo-audit prüfen die festgelegten Abhängigkeiten.
- Actionlint und die Prüfung des gesamten Diffs auf Formatfehler sind erfolgreich.
- Die Workflow-Tokens haben nur Leserechte. Checkouts speichern keine Git-Zugangsdaten.
  Actions sind auf Commit-IDs, die Scannerarchive auf Prüfsummen festgelegt.
- Es gibt keinen Workflow für Veröffentlichung, Deployment, Image-Erstellung oder automatische Merges.
- Lokale Zugangswerkzeuge unter `.local/` sind nicht versioniert.
  Die Testdaten enthalten ausschließlich synthetische Organisationskennungen.
- Die lokale Windows-Umgebung ersetzt den erfolgreichen Linux-Buildnachweis nicht.
  Frühere lokale Rust-Linkergrenzen sind dokumentiert; die Linux-CI enthält Rust-Build und Tests.

Die Vorprüfung testet keine Mandantenisolation, Rechtewiderrufe, Wahlgeheimnisse oder Wiederherstellung einer Fachanwendung.
Diese Funktionen existieren noch nicht. Sie dürfen aus grünen P0-Prüfungen nicht abgeleitet werden.

## Erneut geprüfter Main-Schutz

Die API bestätigt weiterhin `protected: true`; main steht auf dem Initialcommit `a6fae87`.
Beide Regelsätze sind aktiv:

- `25341109`: keine Ausnahme von fünf aktuellen Pflichtprüfungen, PR-Zwang, Gesprächsklärung, Lösch- oder Force-Push-Schutz.
- `25346711`: zusätzliche Reviewpflicht; einzige Konto-Ausnahme `User 77058548 / pull_request`.

Die Projektleitung hat die Konto-Ausnahme ausdrücklich gewählt.
Ihre Nutzung nur für eigene PRs ist eine dokumentierte Regel; GitHub erzwingt die Autorgrenze nicht.
Der Assistent verwendet die Ausnahme nur nach gesondertem Merge-Auftrag.
Die CODEOWNERS-Datei fehlt bis zur Integration von PR #42 noch auf main.
Die Schutzregeln gelten nur für main, nicht für die derzeitigen Arbeitsbranches als PR-Ziel.

## Konkrete noch fehlende Abnahmen

1. Timo Neumeier bestätigt die persönliche und sprachliche Durchsicht der vorbereiteten Änderungen.
2. Die Projektleitung erteilt einen gesonderten Auftrag zur Integration der PRs #39 bis #44.
3. Nach Integration werden Main-CI, Issue-Status und P0-Gesamtabnahme dokumentiert.

Die technische Konfiguration der Review-Ausnahme allein ist weder persönliche Durchsicht noch Merge-Freigabe.
Der Lizenzentwurf bleibt ein nicht freigegebener Prüfentwurf.
Rechteinhaber, juristische Lizenzprüfung und unabhängige Wahlprotokollprüfung sind ausdrücklich zurückgestellt.
Ihre Sperren und spätesten Prüftermine aus KC-037 bleiben bestehen.
Eine P0-Abnahme würde weder diese Fachfreigaben ersetzen noch einen Auftrag für P1 erteilen.

## Integrationsanleitung für einen späteren freigegebenen Auftrag

1. Lesen Sie PR, Zielbranch, aktuellen Commit und beide aktiven Regelsätze erneut.
2. Entfernen Sie nach bestätigter Durchsicht den Entwurfsstatus des nächsten PRs.
3. Integrieren Sie in der Reihenfolge #39, #40, #41, #42, #43, #44.
4. Verwenden Sie Merge-Commits, damit die gemeinsame Historie der aufeinander aufbauenden Branches erhalten bleibt.
5. Stellen Sie nach Integration einer Basis den nächsten PR auf main um.
6. Bringen Sie dessen Branch bei Bedarf auf die aktuelle Main-Basis. Warten Sie neue Pflichtprüfungen ab.
7. Bei Konflikt, fehlgeschlagener Prüfung oder ungeklärter Abweichung stoppen Sie die nächste Integration und dokumentieren den Zustand.
8. Prüfen Sie nach jedem Merge die Main-CI. Löschen Sie keine als Basis benötigten Arbeitsbranches vorzeitig.
9. Aktualisieren Sie Checkpoint und betroffene Issues. Schließen Sie ein Issue erst nach erfüllter Abnahme.

Diese Sitzung führt diese Integrationsschritte nicht aus.
