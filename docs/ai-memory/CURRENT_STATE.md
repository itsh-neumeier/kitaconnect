# Projektstatus und Fortsetzung

Aktualisiert: 18.10.2026, Europe/Berlin. Grundlage: Spezifikation v5.9.
Status: KC-036 hat die ursprünglichen Unterlagen 01 bis 12 redaktionell überarbeitet.
F-001 ist bestätigt und klargestellt. Timo Neumeier ist als Prüfer benannt.
Die Projektleitung bestätigt anschließend eine Ausnahme ausschließlich für eigene PRs.
Die passende technische Durchsetzung ist noch offen; keine allgemeine Benutzerausnahme ist aktiviert.
KC-038 hat den Main-Branchschutz eingerichtet. P0 ist nicht insgesamt abgenommen.
P1 bis P9 besitzen keinen Implementierungsauftrag.

## Arbeitsstand

- `issue_id`: [KC-038 / #38](https://github.com/itsh-neumeier/kitaconnect/issues/38), Nachtrag im KC-036-PR #44.
- `branch`: `codex/p0-documentation-review`.
- `head_commit`: Verifizierter Ausgangscommit `8499669818c1cb4dce14841d0cb541fb84c77f83`.
  ADR-022 und dieser Checkpoint folgen als eigener Commit. Aktuellen Kopf mit `git rev-parse HEAD` prüfen.
- `objective`: Bestätigte Reviewausnahme ausschließlich für eigene PRs technisch eingrenzen.
- `pull_request`: [PR #44](https://github.com/itsh-neumeier/kitaconnect/pull/44), Entwurf gegen `codex/p0-review-protection`.
- `migration_state`: Keine Datenbankrollen, Migrationen, Zertifikate oder Laufzeitdienste angelegt.
- `dependencies`: PRs #39 bis #43 zuerst prüfen und integrieren. Kein Merge durch diese Sitzung.

## Bestätigte Antworten

Quelle: ausdrückliche Antworten im Projektchat am 18.10.2026.

1. `1A, 2A, 3A, 4A` bestätigt ADR-014 bis ADR-017. Nicht erneut abfragen.
2. Das [Lastprofil](../17_P0_Entscheidungsregister.md) ist bestätigt: eine Einrichtung, 50 Sitzungen,
   10 Anfragen pro Sekunde, 80/20 Lesen/Schreiben, p95 500/1.000 ms und unter 1 % Fehler.
   Referenz: 4 vCPU, 8 GB RAM, 100 GB SSD, 5 Minuten Aufwärmen und 30 Minuten Messung.
   Medienverarbeitung bleibt gesondert. Es gibt noch keine repräsentativen Messergebnisse.
3. Rechteinhaber, juristische Lizenzprüfung und unabhängige Wahlprüfung sollen ausdrücklich offen bleiben.
   Projektleitung bleibt für Besetzung vor den gesperrten Aktionen zuständig.
4. `@itsh-neumeier Timo Neumeier` bestätigt den vorläufigen CODEOWNER.
   Dies ernennt keinen Rechteinhaber oder unabhängigen Fachprüfer.
5. npm audit ist lokal und in GitHub Actions ausdrücklich erlaubt.
6. Technische Weiterarbeit innerhalb der Anforderungen ist delegiert. Keine weitere Entwicklungsphase beginnen.
7. Interviewantwort „1. Ja“ bestätigt die Klarstellung F-001: Eingabe, serverseitige Prüfung vor Speichern,
   Speichern, gegebenenfalls Freigabe für Eltern und Zugriffsprotokollierung.
8. Interviewantwort „2 person steht fest itsh-neumeier“ benennt den PR-Prüfer.
   Der Benutzer ist zugleich Autor der aktuellen PRs. Er kann diese nicht selbst genehmigen.
9. Interviewantwort „3. Ich auch als Prüfer“ benennt Timo Neumeier für Sicherheits- und Datenschutzprüfungen.
   Sie belegt keine erfolgte Fachprüfung oder unabhängige Gegenprüfung eigener Änderungen.
10. „Ich prüfe die auch sowie du als KI-Assistent“ bestätigt persönliche Prüfung durch die Projektleitung
    und technische Vorprüfung durch den Assistenten. Die GitHub-Pflichtgenehmigung ist damit nicht erfüllt.
    Die PRs wurden über den Zugang der Projektleitung erstellt. Keine abgeschlossene menschliche Prüfung behaupten.
11. Die spätere Antwort „Ja genau“ bestätigt die Rückfrage: Ausnahme nur für eigene PRs von `@itsh-neumeier`.
    Fremde PRs brauchen Review. Alle fünf Pflichtprüfungen bleiben verbindlich. Umfang nicht erneut abfragen.

## Erledigt in KC-038

1. AGENTS, CLAUDE, Fortsetzungsanleitung, Checkpoint, Entscheidungsprotokoll und relevante Unterlagen gelesen.
2. Sauberen Ausgangsbranch und erfolgreiche fünf Prüfungen von PR #42 geprüft.
3. `GET branches/main` zeigte zunächst `protected: false`; die Regelsatzliste war leer.
4. Die App erhielt beim klassischen Schutzendpunkt HTTP 403. Ihr fehlen dort Verwaltungsrechte.
5. Der vorhandene Git-Zugang bestätigte Administrationsrechte. Er wurde nur im Speicher verwendet.
   Keine Zugangsdaten ausgegeben oder gespeichert. Keine zusätzlichen Zugangsdaten angelegt.
6. Aktiven Regelsatz `25341109` ausschließlich für `refs/heads/main` erstellt.
7. Reviews, CODEOWNER-Prüfung, aktuelle Basis und alle fünf GitHub-Actions-Prüfungen sind erforderlich.
   Löschen und Force-Push sind gesperrt. Es gibt keine eingetragenen Merge-Ausnahmen.
8. Regelsatz und `protected: true` nach Einrichtung erneut über die GitHub-App gelesen.
9. Plan, API-Momentaufnahme, [ADR-021](../architecture/ADR-021_Branchschutz_und_Review.md)
   und [Betriebsanleitung](../18_Branchschutz_und_Reviews.md) erstellt.
10. Lokale Planprüfung und negative Tests in den bestehenden CI-Prüfschritt `structure` aufgenommen.
11. Spätere Zielregel in [ADR-022](../architecture/ADR-022_Reviewausnahme_eigene_PRs.md) dokumentiert.
    Eingebaute Benutzerausnahme kann nicht nach PR-Autor unterscheiden und wurde nicht aktiviert.
    Konkreten App-Vorschlag und Abnahmefälle vorbereitet. Zusätzliche App und Betriebsweg sind noch nicht entschieden.

## Erledigt in KC-036

1. Sauberen Ausgangsstand `bdb3f9f`, PR #43, Issue #36, ADRs und alle fünf erfolgreichen GitHub-Prüfungen gelesen.
2. Aktiven Regelsatz ohne Merge-Ausnahmen erneut gelesen. Keine GitHub-Verwaltungseinstellung geändert.
3. Alle zwölf ursprünglichen Unterlagen gelesen und in drei Redaktionsgruppen überarbeitet.
4. Stichwortketten aufgeteilt, Begriffe erklärt und historische Abschnitte im Fachkonzept eindeutig nummeriert.
5. Historische Ausschlüsse unter der bestehenden Vorrangregel erhalten. Kein Ausschluss eigenmächtig aufgehoben.
6. Fachwortverzeichnis ergänzt. API-Pfade, Ereignisnamen und Berechtigungstabellen unverändert erhalten.
7. [Prüfbericht](../19_Redaktionspruefung_KC-036.md) mit Nachweis je Unterlage und offenen Formulierungen erstellt.
8. Keine neue Architekturentscheidung getroffen. ADR-010 bleibt die redaktionelle Grundlage.
9. F-001 nach ausdrücklichem „Ja“ klargestellt: serverseitige Berechtigungsprüfung vor Speichern.
   ADR-019 bleibt verbindlich. Keine Fachfunktion implementiert.
10. Benannte Prüfzuständigkeit und Konflikt mit der GitHub-PR-Autorschaft dokumentiert.
    Gewünschte persönliche Prüfung plus KI-Vorprüfung erfasst. Technische Pflichtgenehmigung bleibt unerfüllt.

## Grenzen und offene Rollen

- Für fremde PRs bleibt unabhängige Genehmigung erforderlich. Für eigene PRs ist die begrenzte Ausnahme bestätigt,
  aber technisch noch nicht eingerichtet. Keine weitergehende Benutzerausnahme aktivieren.
- Persönliche Prüfung plus KI-Vorprüfung ist bestätigt. Nicht erneut nach dieser gewünschten Aufteilung fragen.
  Keine Pflichtgenehmigung daraus ableiten; bestehende Sperre bleibt bestehen.
- F-001 ist entschieden und in Unterlage 05 umgesetzt. Nicht erneut abfragen.
- Redaktionelle Eigenprüfung ist dokumentiert. Unabhängige sprachliche PR-Prüfung und Integration fehlen.
- Timo Neumeier ist für Sicherheits- und Datenschutzprüfungen benannt. Unabhängige Gegenprüfung eigener Änderungen
  sowie konkrete Prüfberichte fehlen. Fachfreigaben aus KC-037 bleiben erforderlich.
- CODEOWNERS liegt bisher nur auf dem Arbeitsbranch von PR #42. Die Datei fehlt auf `main`.
- Ein Autor kann eigene Änderungen nicht genehmigen. Mit nur einem CODEOWNER bleiben dessen eigene Merges
  unter dem bestehenden Schutz ohne weiteren bestätigten CODEOWNER gesperrt.
  ADR-022 dokumentiert die bestätigte Zielregel, ohne einen bereits aktiven Bypass zu behaupten.
- Der Regelsatz schützt nur `main`. Auf Arbeitsbranches gerichtete Folge-PRs sind noch nicht durch ihn geschützt.
- Administratoren können Regelsätze weiterhin verwalten. Keine Unveränderlichkeit behaupten.
- [API-Nachweis](BRANCH_PROTECTION_STATE.json) ist eine Momentaufnahme, vor Merge erneut prüfen.

## Prüfungen

| Prüfung | Ergebnis |
|---|---|
| `node scripts/check-project.mjs` | 38 Issues, 31 Dokumente und lokale Verweise geprüft |
| `node scripts/check-github-rules.mjs` | Versionierter Plan erfüllt Schutzanforderungen |
| Planprüfung auf ausgelesener API-Antwort | Erfolgreich |
| `node scripts/fixtures.mjs --check` | Bytegenau erfolgreich |
| `node --test scripts/*.test.mjs` | 28 von 28 Tests erfolgreich, davon 14 negative Branchschutzfälle |
| actionlint 1.7.12 | Erfolgreich |
| `git diff --check` | Erfolgreich |
| Vergleich mit `bdb3f9f` | Tabellen 03/04, Ereignisnamen, Phasenabhängigkeiten, ADR-/Interviewkennungen und Werkzeugversionen unverändert |

Die Prüfungen des Ausgangscommits `bdb3f9f` sind vollständig erfolgreich:
[P0 CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39123817480) und
[P0 Security](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39123817474).
Für KC-036-Commit `733f7a5` sind [P0 CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39132777909)
und [P0 Security](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39132777911) vollständig erfolgreich.
Auch der Ausgangscommit `fc60092` ist vollständig grün:
[P0 CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39132856831),
[P0 Security](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39132856848).
Für Interviewcommit `2617c38` sind [P0 CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39134454271)
und [P0 Security](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39134454419) vollständig erfolgreich.
Für Ausgangscommit `8499669` sind [P0 CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39134864033)
und [P0 Security](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39134864021) vollständig erfolgreich.
Der folgende Dokumentationscommit löst eigene Prüfungen aus. Vor Merge muss der aktuelle Kopf grün sein.
Alle vorgemerkten Gruppen und die vollständige Historie mit 16 Commits wurden mit Gitleaks ohne Befund geprüft.
Rust-, Frontend- und Workflow-Code wurden nicht geändert.
Keine neuen lokalen Testfehler. Keine implementierte Dienst-, Last- oder Wahlsicherheit behauptet.

## Nächste drei Aktionen

1. ADR-022 in PR #44 sichern, aktuelle CI prüfen und Issue #38 aktualisieren. F-001 und Ausnahmeumfang nicht erneut abfragen.
2. Zusätzliche GitHub-App und Betriebsweg entscheiden lassen. Bis zur passenden technischen Durchsetzung bleibt der alte Schutz aktiv.
   KC-036 und KC-038 bleiben offen. Keine Ausnahme für fremde PRs oder Pflichtprüfungen einrichten.
3. Integrationsnachweise und P0-Abnahme vorbereiten. Keine neue Entwicklungsphase beginnen.

Integrationsfolge: #39, #40, #41, #42, #43, danach KC-036. Nach Integration der Basis den nächsten PR
auf `main` umstellen und aktuelle CI sowie Reviews erneut prüfen. Keine eigenständige Merge-Freigabe.
Beim Neustart AGENTS, CLAUDE, diesen Status, offene Entscheidungen und den GitHub-Stand lesen.
