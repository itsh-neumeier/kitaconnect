# P0 – Branchschutz und Review-Ausnahme

Stand: 18.10.2026. Arbeitspaket: KC-038. Grundlage: Spezifikation v5.9 und ADR-022.

## Aktiver Schutz

Der [Basisregelsatz 25341109](https://github.com/itsh-neumeier/kitaconnect/rules/25341109) gilt ohne Ausnahme für `main`.
Er verlangt `structure`, `rust`, `web`, `secrets` und `dependencies` mit aktueller Basis.
Prüfungen müssen von GitHub Actions mit Integrationskennung `15368` stammen.
Er verlangt außerdem PRs und geklärte Review-Gespräche. Löschen und Force-Push bleiben verboten.

Der [Review-Regelsatz 25346711](https://github.com/itsh-neumeier/kitaconnect/rules/25346711) verlangt mindestens eine Genehmigung,
CODEOWNER-Prüfung, Verwerfen veralteter Genehmigungen und Prüfung des letzten Pushs durch eine andere Person.
Nur `@itsh-neumeier`, Benutzer-ID `77058548`, erhält hier eine Ausnahme im Modus `pull_request`.
Andere Benutzer und Administratorrollen erhalten keine Ausnahme.
Die Ausnahme erlaubt keine Umgehung des getrennten Basisschutzes.

## Ausdrücklich bestätigte Nutzungsregel

Die Projektleitung wählt: „Einfache Konto-Ausnahme; Nutzung auf eigene PRs beschränken“.
GitHub kann bei dieser Ausnahme nicht nach PR-Autor unterscheiden.
Die Ausnahme wäre technisch auch bei fremden PRs verwendbar.
Die verbindliche Nutzungsregel erlaubt dies ausschließlich bei eigenen PRs von `@itsh-neumeier`.
[ADR-022](architecture/ADR-022_Reviewausnahme_eigene_PRs.md) dokumentiert die akzeptierte technische Grenze.

Timo Neumeier übernimmt persönliche Prüfung sowie die benannten Sicherheits- und Datenschutzprüfungen.
Der KI-Assistent unterstützt mit technischer Vorprüfung und nachvollziehbaren Ergebnissen.
Die Zuständigkeit belegt keine bereits erfolgte Prüfung.
Die Ausnahme erzeugt keine Selbstgenehmigung; sie erlaubt den dokumentierten Verzicht auf das zusätzliche GitHub-Pflichtreview.
Sie ersetzt weder unabhängige Wahlprotokollprüfung noch juristische Lizenzprüfung oder andere erforderliche Fachfreigaben.

## CODEOWNERS und Integrationsfolge

`@itsh-neumeier` ist als vorläufiger CODEOWNER bestätigt.
Die Datei ist nach Integration von PR #42 auf `main` vorhanden.
GitHub verwendet die CODEOWNERS-Datei des Zielbranchs.
Die Konto-Ausnahme selbst bleibt aktiv; ihre Wirksamkeit ist von der CODEOWNERS-Datei unabhängig.

Die Integrationsfolge lautet #39, #40, #41, #42, #43, #44.
Diese sechs PRs sind nach ausdrücklicher Freigabe integriert.
Jeder Folge-PR wurde vorher auf `main` umgestellt, seine Basis aktualisiert und seine CI erneut geprüft.
Der [Integrationsnachweis](21_P0_Integration_und_Abnahmevorbereitung.md) enthält die Merge- und Main-Prüfstände.

1. Prüfen Sie PR-Autor, Zielbranch und aktuellen Commit.
2. Prüfen Sie persönliche Durchsicht, technische Vorprüfung und sämtliche Pflichtprüfungen.
3. Dokumentieren Sie bei eigenen PRs den Grund für die Review-Ausnahme und den geprüften Commit.
4. Verlangen Sie bei fremden PRs gemäß Nutzungsregel ein Review.
5. Integrieren Sie nur nach gesondertem Merge-Auftrag und erfülltem Basisschutz.

Die spätere freigegebene Integration umfasst ausschließlich PRs #39 bis #44. P0 ist noch nicht insgesamt abgenommen.

## Nachweis und Änderungen

Die [API-Momentaufnahme](ai-memory/BRANCH_PROTECTION_STATE.json) enthält beide aktiven Regelsätze.
`GET branches/main` bestätigte nach Einrichtung weiterhin `protected: true`.
Dies ist eine Momentaufnahme. Lesen Sie vor Merge beide Regelsätze und den aktuellen Prüfstand erneut.
Die lokale Prüfung lautet `node scripts/check-github-rules.mjs`.
Für separat gespeicherte API-Antworten lautet sie:

```sh
node scripts/check-github-rules.mjs BASISANTWORT.json REVIEWANTWORT.json
```

Die CI prüft die versionierten Pläne und hat keine Verwaltungsrechte.
HTTP 403, fehlende Antworten oder reine Konfigurationstests sind kein erfolgreicher Aktivierungsnachweis.
Administratoren können Regelsätze weiterhin verwalten. Änderungen benötigen dokumentierten Anlass und neuen API-Nachweis.
Zugangsdaten wurden bei der Einrichtung nur im Speicher verwendet und weder ausgegeben noch gespeichert.

Quelle: [GitHub-Regelsatz-API](https://docs.github.com/en/rest/repos/rules?apiVersion=2026-03-10).
