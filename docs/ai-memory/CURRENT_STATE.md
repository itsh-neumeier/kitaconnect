# Projektstatus und Fortsetzung

Aktualisiert: 18.10.2026, Europe/Berlin. Grundlage: Spezifikation v5.9.
Status: Die ausdrücklich gewählte Konto-Ausnahme ist eingerichtet und per API nachgewiesen.
P0 ist nicht insgesamt abgenommen. P1 bis P9 besitzen keinen Implementierungsauftrag.

## Arbeitsstand

- `issue_id`: [KC-038 / #38](https://github.com/itsh-neumeier/kitaconnect/issues/38), Nachtrag im KC-036-PR #44.
- `branch`: `codex/p0-documentation-review`.
- `head_commit`: Verifizierter Ausgangscommit `bad4415865c0c8a9be2249ff64909e0da0b5c8dc`.
  Konto-Ausnahme, Tests und dieser Checkpoint folgen als eigener Commit. Aktuellen Kopf mit `git rev-parse HEAD` prüfen.
- `objective`: Konto-Ausnahme ausschließlich von Review-Regeln; sämtliche Pflichtprüfungen ohne Ausnahme erhalten.
- `pull_request`: [PR #44](https://github.com/itsh-neumeier/kitaconnect/pull/44), Entwurf gegen `codex/p0-review-protection`.
- `migration_state`: Keine Datenbankrollen, Migrationen, Zertifikate oder Laufzeitdienste angelegt.
- `dependencies`: PRs #39 bis #43 zuerst prüfen und integrieren. Kein Merge durch diese Sitzung.

## Verbindliche Antworten und neuester Vorrang

1. `1A, 2A, 3A, 4A` bestätigt ADR-014 bis ADR-017. Nicht erneut abfragen.
2. Das [Lastprofil](../17_P0_Entscheidungsregister.md) ist bestätigt: eine Einrichtung, 50 Sitzungen,
   10 Anfragen pro Sekunde, 80/20 Lesen/Schreiben, p95 500/1.000 ms, unter 1 % Fehler,
   4 vCPU, 8 GB RAM, 100 GB SSD, 5 Minuten Aufwärmen und 30 Minuten Messung.
   Medienverarbeitung bleibt gesondert. Repräsentative Messergebnisse fehlen noch.
3. Rechteinhaber, juristische Lizenzprüfung und unabhängige Wahlprüfung sollen ausdrücklich offen bleiben.
   Die Sperren und spätesten Prüftermine aus KC-037 bleiben verbindlich.
4. `@itsh-neumeier Timo Neumeier` ist CODEOWNER sowie benannter Sicherheits- und Datenschutzprüfer.
   Dies ist keine Ernennung zum Rechteinhaber und kein Nachweis einer erfolgten oder unabhängigen Fachprüfung.
5. Persönliche Durchsicht durch die Projektleitung und technische Vorprüfung durch den KI-Assistenten sind bestätigt.
6. F-001 ist mit „Ja“ entschieden. Unterlage 05 verlangt serverseitige Prüfung vor dem Speichern.
7. Neueste ausdrückliche Auswahl: **„Einfache Konto-Ausnahme; Nutzung auf eigene PRs beschränken“**.
   Die technische Verwendung auch bei fremden PRs wurde vorher erläutert und ist akzeptiert.
   „Nur eigene PRs“ ist eine Nutzungsregel, keine technisch erzwungene Autorbedingung. Nicht erneut abfragen.
8. npm audit ist lokal und in GitHub Actions erlaubt. Technische P0-Weiterarbeit ist delegiert.
   Keine zusätzliche App und keine weitere Entwicklungsphase sind dafür erforderlich oder beauftragt.

## Aktueller GitHub-Schutz

- Basisregelsatz `25341109`: aktiv, nur `main`, **keine Ausnahme**.
  Erzwingt `structure`, `rust`, `web`, `secrets`, `dependencies`, aktuelle Basis, PR-Zwang,
  geklärte Review-Gespräche sowie Schutz vor Löschen und Force-Push.
- Review-Regelsatz `25346711`: aktiv, nur `main`, ein Review, CODEOWNER-Prüfung,
  Verwerfen veralteter Genehmigungen und Genehmigung des letzten Pushs durch andere Person.
  **Einzige Ausnahme:** Benutzer-ID `77058548`, `@itsh-neumeier`, Modus `pull_request`.
- Konto-Ausnahme hebt keine Regel des getrennten Basisschutzes auf.
- Zuerst neuen Review-Regelsatz angelegt und geprüft; danach bisherigen Regelsatz zum Basisschutz geändert.
- Beide Antworten nach Einrichtung erneut über die GitHub-App gelesen; `main` weiterhin `protected: true`.
- [API-Nachweis](BRANCH_PROTECTION_STATE.json), [ADR-022](../architecture/ADR-022_Reviewausnahme_eigene_PRs.md)
  und [Betriebsanleitung](../18_Branchschutz_und_Reviews.md) aktualisiert.
- Vorhandener Git-Zugang nur im Speicher verwendet. Keine Zugangsdaten gespeichert oder ausgegeben.
- Kein PR integriert und kein automatischer Merge eingerichtet.

## Bisheriges P0-Ergebnis

PRs #39 bis #43 enthalten Grundgerüst, Dienstgrenzen, Testdaten, Entscheidungsregister und ursprünglichen Branchschutz.
KC-036 hat die ursprünglichen Unterlagen 01 bis 12 redaktionell geprüft und F-001 nach Bestätigung klargestellt.
Der [Redaktionsbericht](../19_Redaktionspruefung_KC-036.md) dokumentiert Vergleich und manuelle Eigenprüfung.
Entitäts-/Berechtigungstabellen, API-Tabelle, Ereignisnamen und Phasenabhängigkeiten bleiben erhalten.
Die ursprüngliche Branchschutzentscheidung ADR-021 wird durch die ausdrücklich gewählte Konto-Ausnahme fortgeschrieben.

## Prüfungen dieses Stands

| Prüfung | Ergebnis |
|---|---|
| `node scripts/check-github-rules.mjs` | Beide versionierten Pläne erfolgreich |
| Prüfung auf getrennten API-Antworten | Basis und Review-Regelsatz erfüllen dieselben Validatoren |
| `node --test scripts/*.test.mjs` | 40 von 40 Tests erfolgreich, davon 26 negative Branchschutzfälle |
| `node scripts/check-project.mjs` | 38 Issues, 31 Dokumente und lokale Verweise erfolgreich geprüft |
| `node scripts/fixtures.mjs --check` | Bytegenau erfolgreich |
| `git diff --check` | Erfolgreich |

Ausgangscommit `bad4415`: [P0 CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39135846179)
und [P0 Security](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39135846189) erfolgreich.
Der neue Commit benötigt eigene CI-Nachweise. Keine Laufzeitdienste oder Workflows geändert.
Kein tatsächlicher Merge- oder negativer Push-Versuch zur Schutzprüfung durchgeführt.
API-Nachweise und Konfigurationstests sind keine bereits erfolgte persönliche Prüfung oder Fachfreigabe.

## Grenzen und nächste drei Aktionen

1. Konto-Ausnahme in PR #44 sichern, aktuelle CI nachweisen und Issues #36/#38 nachführen.
2. Persönliche sprachliche Prüfung am konkreten Commit dokumentieren lassen.
   Review-Ausnahme nur für eigene PRs nutzen; bei fremden PRs Review verlangen.
   Ein Merge braucht weiterhin einen gesonderten Auftrag. Die Einrichtung allein erteilt keine Merge-Freigabe.
3. Nach Integration P0-Abnahme nachführen. Offene juristische und unabhängige Wahlprüfungen vor ihren Sperrterminen veranlassen.

Integrationsfolge: #39, #40, #41, #42, #43, #44.
Nach Integration einer Basis den nächsten PR auf `main` umstellen und aktuelle CI erneut prüfen.
Die Regelsätze schützen nur `main`, nicht die bisherigen Arbeitsbranches als PR-Ziel.
CODEOWNERS liegt bis zur Integration von PR #42 noch nicht auf `main`.
Beim Neustart AGENTS, CLAUDE, diesen Status, ADR-022, offene Entscheidungen und GitHub-Stand lesen.
