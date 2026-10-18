# Projektstatus und Fortsetzung

Aktualisiert: 18.10.2026, Europe/Berlin. Grundlage: Spezifikation v5.9.
Status: Technische P0-Vorprüfung dokumentiert; alle sechs vorbereiteten PR-Stände sind grün.
Persönliche Durchsicht und gesonderter Integrationsauftrag stehen aus.
P0 ist nicht insgesamt abgenommen. P1 bis P9 besitzen keinen Implementierungsauftrag.

## Arbeitsstand

- `issue_id`: [KC-038 / #38](https://github.com/itsh-neumeier/kitaconnect/issues/38), Nachtrag im KC-036-PR #44.
- `branch`: `codex/p0-documentation-review`.
- `head_commit`: Geprüfter Gesamtstand `f4d8983ecf46ef61735ab3984435aaf037430a7f`.
  Prüfbericht und dieser Checkpoint folgen als Dokumentationscommit. Aktuellen Kopf mit `git rev-parse HEAD` prüfen.
- `objective`: P0-Nachweise zusammenführen und persönliche Durchsicht sowie spätere Integration vorbereiten.
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

## Technische Vorprüfung dieser Sitzung

Der [P0-Prüfbericht](../20_P0_Pruefung_und_Abnahme.md) dokumentiert Umfang und Ergebnis.
Alle sechs offenen PRs #39 bis #44 sind Entwürfe. Jeder aktuelle Kopf hat fünf erfolgreiche Pflichtprüfungen.
Quelltext, Konfiguration, Workflows, Tests, Infrastruktur und Repository-Vorlagen im P0-Umfang wurden durchgesehen.
Die P0-Basis ist Vorfahr des Gesamtstands. Es gibt keine neuen Fachfunktionen, Datenbankmigrationen oder Veröffentlichung.
Beide aktiven Regelsätze und den weiterhin unveränderten Initialstand von main erneut per API gelesen.
Keine neue blockierende technische Abweichung im dokumentierten Umfang festgestellt.
Dies ist keine unabhängige Fachprüfung und keine persönliche Freigabe durch die Projektleitung.
Die widersprüchliche allgemeine Reviewformulierung im P0-Plan wurde an die bereits bestätigte ADR-022 angepasst.

## Prüfungen dieses Stands

| Prüfung | Ergebnis |
|---|---|
| `node scripts/check-github-rules.mjs` | Beide versionierten Pläne erfolgreich |
| Prüfung auf getrennten API-Antworten | Basis und Review-Regelsatz erfüllen dieselben Validatoren |
| `node --test scripts/*.test.mjs` | 40 von 40 Tests erfolgreich, davon 26 negative Branchschutzfälle |
| `node scripts/check-project.mjs` | 38 Issues, 32 Dokumente und lokale Verweise erfolgreich geprüft |
| `node scripts/fixtures.mjs --check` | Bytegenau erfolgreich |
| `git diff --check` | Erfolgreich |

Ausgangscommit `bad4415`: [P0 CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39135846179)
und [P0 Security](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39135846189) erfolgreich.
Für Änderungscommit `3077cea` sind [P0 CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39138328760)
und [P0 Security](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39138328793) vollständig erfolgreich.
Für den geprüften Gesamtstand `f4d8983` sind [P0 CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39138506345)
und [P0 Security](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39138506397) vollständig erfolgreich.
Alle übrigen geprüften PR-Köpfe und ihre CI-Nachweise stehen im P0-Prüfbericht.
Actionlint und `git diff --check a6fae87..HEAD` waren in dieser Sitzung erfolgreich.
Der folgende Dokumentationscommit löst eigene Prüfungen aus. Keine Laufzeitdienste oder Workflows geändert.
Der vorgemerkte Diff wurde mit Gitleaks ohne Befund geprüft. Issues #36/#38 und PR #44 sind nachgeführt.
Kein tatsächlicher Merge- oder negativer Push-Versuch zur Schutzprüfung durchgeführt.
API-Nachweise und Konfigurationstests sind keine bereits erfolgte persönliche Prüfung oder Fachfreigabe.

## Grenzen und nächste drei Aktionen

1. Prüfbericht in PR #44 sichern und aktuelle CI nachweisen. P0-Issues mit dem gemeinsamen Nachweis nachführen.
2. Persönliche sprachliche Prüfung und gesonderten Auftrag zur Integration #39 bis #44 bestätigen lassen.
   Review-Ausnahme nur für eigene PRs nutzen; bei fremden PRs Review verlangen.
   Ein Merge braucht weiterhin einen gesonderten Auftrag. Die Einrichtung allein erteilt keine Merge-Freigabe.
3. Nach Integration P0-Abnahme nachführen. Offene juristische und unabhängige Wahlprüfungen vor ihren Sperrterminen veranlassen.

Integrationsfolge: #39, #40, #41, #42, #43, #44.
Nach Integration einer Basis den nächsten PR auf `main` umstellen und aktuelle CI erneut prüfen.
Merge-Commits erhalten die gemeinsame Historie. Bei neuer Main-Basis Branch aktualisieren und neue Pflichtprüfungen abwarten.
Die Regelsätze schützen nur `main`, nicht die bisherigen Arbeitsbranches als PR-Ziel.
CODEOWNERS liegt bis zur Integration von PR #42 noch nicht auf `main`.
Beim Neustart AGENTS, CLAUDE, diesen Status, ADR-022, offene Entscheidungen und GitHub-Stand lesen.
