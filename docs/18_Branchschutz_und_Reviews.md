# P0 – Branchschutz und unabhängige Reviews

Stand: 18.10.2026. Arbeitspaket: KC-038. Grundlage: Spezifikation v5.9 und ADR-013.

## Nachgewiesener Zustand

Der Regelsatz [25341109](https://github.com/itsh-neumeier/kitaconnect/rules/25341109) ist aktiv.
Er gilt ausschließlich für `refs/heads/main`.
Die API meldet `protected: true`. Der Hauptbranch steht weiterhin auf `a6fae87`.
Diese Sitzung hat keinen Pull Request integriert.

Der [versionierte Plan](../.github/rulesets/main.json) enthält folgende Regeln:

- Mindestens ein genehmigendes Review. Neue Änderungen verwerfen frühere Genehmigungen.
- Erforderliches CODEOWNER-Review. Die letzte Änderung benötigt die Genehmigung einer anderen Person.
- Alle Review-Gespräche müssen geklärt sein.
- `structure`, `rust`, `web`, `secrets` und `dependencies` müssen erfolgreich sein.
- Der geprüfte Stand muss den aktuellen Hauptbranch enthalten.
- Die Pflichtprüfungen sind an GitHub Actions mit Integrationskennung `15368` gebunden.
- Löschen und Force-Push sind verboten. Es gibt keine eingetragenen Ausnahmen.

Die Integrationskennung wurde an den erfolgreichen Prüfungen von Commit `a893f76` abgelesen.
Der [API-Nachweis](ai-memory/BRANCH_PROTECTION_STATE.json) enthält den ausgelesenen Regelsatz.
Er ist eine Momentaufnahme. Er ersetzt keine erneute Prüfung vor einem Merge.
Der allgemeine Branch-Endpunkt und der Regelsatz-Endpunkt wurden nach der Einrichtung erneut gelesen.
Die verbundene App darf den klassischen Verwaltungsendpunkt nicht lesen und lieferte dort HTTP 403.
Der vorhandene Git-Zugang erlaubte die Einrichtung. Zugangsdaten wurden nur im Speicher verwendet.
Sie wurden weder in Dateien gespeichert noch ausgegeben.

## Review-Zuständigkeit und Sperren

`@itsh-neumeier`, Timo Neumeier, ist ausdrücklich als vorläufiger CODEOWNER bestätigt.
Die Datei liegt bisher auf dem Arbeitsbranch aus PR #42. Sie fehlt noch auf `main`.
GitHub verwendet die CODEOWNERS-Datei des Zielbranchs.
Bis zu ihrer Integration besteht deshalb noch keine wirksame CODEOWNER-Zuordnung für `main`.
Die allgemeine Pflicht zu einem genehmigenden Review gilt bereits.

Der Autor darf eigene Änderungen nicht genehmigen.
Ein einzelner CODEOWNER kann deshalb seine eigenen Änderungen nach Integration der Datei nicht selbst freigeben.
Die Projektleitung MUSS eine unabhängige Person benennen und deren erforderlichen Repository-Zugriff bestätigen.
Diese Person benötigt eine bestätigte CODEOWNER-Zuordnung, wenn sie die einzige erforderliche Eigentümerprüfung leisten soll.
Bis dahin bleiben betroffene Merges gesperrt. Die Regeln werden dafür nicht abgeschwächt.
Die Rückfrage zum GitHub-Benutzernamen ist offen. Es wurde niemand eigenmächtig ernannt oder angefragt.

Die unabhängigen Sicherheits- und Datenschutzprüfer bleiben ebenfalls unbesetzt.
Die Projektleitung benennt sie vor der jeweiligen Freigabe.
Ein allgemeines PR-Review ersetzt weder Datenschutzprüfung noch juristische Lizenzprüfung oder Wahlprotokollprüfung.
Die ausdrücklich zurückgestellten Fachrollen und ihre Sperren aus dem KC-037-Register gelten weiter.
P0 ist nicht insgesamt abgenommen.

## Integration der aufeinander aufbauenden Pull Requests

Die Reihenfolge lautet: #39, #40, #41, #42, danach der KC-038-PR.
Die Folge-PRs zielen derzeit auf Arbeitsbranches. Der Main-Regelsatz schützt diese Zielbranches nicht.
Eine grüne CI auf einem Arbeitsbranch ist daher noch keine durchgesetzte Main-Freigabe.

1. Prüfen Sie den ersten PR gegen `main`. Fordern Sie das unabhängige Review an.
2. Prüfen Sie alle fünf Pflichtprüfungen für den aktuellen Commit und die aktuelle Basis.
3. Lassen Sie die deutsche Dokumentation manuell sprachlich prüfen.
4. Integrieren Sie nur nach ausdrücklicher Merge-Freigabe und erfüllten Regeln.
5. Stellen Sie den nächsten PR nach Integration seiner Basis auf `main` um.
6. Prüfen Sie Änderungen, CI und Reviews erneut. Wiederholen Sie diese Schritte bis zum letzten PR.

## Schutz prüfen und ändern

1. Führen Sie `node scripts/check-github-rules.mjs` aus. Dies prüft nur den lokalen Plan.
2. Lesen Sie `GET /repos/itsh-neumeier/kitaconnect/rulesets/25341109` mit einem berechtigten GitHub-Zugang.
3. Speichern Sie ausschließlich die Regelsatzantwort, ohne Header oder Zugangsdaten.
4. Prüfen Sie diese Datei mit `node scripts/check-github-rules.mjs PFAD_ZUR_ANTWORT.json`.
5. Lesen Sie zusätzlich `GET /repos/itsh-neumeier/kitaconnect/branches/main`. Prüfen Sie `protected: true`.
6. Vergleichen Sie den aktuellen PR-Commit mit seinen Prüfungen und Reviews.

Fehlende Berechtigung, HTTP 403 oder eine leere Antwort gelten nicht als erfolgreicher Schutznachweis.
Bei Abweichung bleibt der Merge gesperrt. Aktualisieren Sie Issue und Checkpoint.
Die CI prüft den lokalen Plan und negative Testfälle. Sie hat keine Verwaltungsrechte.
Sie liest oder verändert deshalb keine GitHub-Verwaltungseinstellung.

GitHub-Administratoren können Regelsätze weiterhin verwalten.
Eine leere Ausnahmeliste verhindert den Merge-Bypass unter dem aktiven Regelsatz.
Sie macht Verwaltungsrechte nicht unveränderlich.
Änderungen an Regeln benötigen einen dokumentierten Anlass, Prüfung und einen neuen API-Nachweis in KC-038.
Kein Prüfer darf die Regeln zur Selbstfreigabe deaktivieren.

Quellen: [GitHub-Regelsatz-API](https://docs.github.com/en/rest/repos/rules?apiVersion=2022-11-28),
[geschützte Branches](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches),
[CODEOWNERS](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners).
