# ADR-022 – Konto-Ausnahme für Reviews mit Nutzung nur für eigene PRs

Datum: 18.10.2026. Bezug: KC-038 und ADR-021.
Status: ausdrücklich bestätigt, eingerichtet und per API nachgewiesen.

## Entscheidung der Projektleitung

Die Projektleitung wählt ausdrücklich: „Einfache Konto-Ausnahme; Nutzung auf eigene PRs beschränken“.
Diese Antwort konkretisiert die frühere Antwort „Ja genau“ zum gewünschten Umfang.
`@itsh-neumeier` darf die zusätzliche genehmigende Reviewpflicht bei eigenen PRs ausnehmen.
Persönliche Prüfung durch die Projektleitung und technische KI-Vorprüfung bleiben vorgesehen.
Für fremde PRs verlangt die Nutzungsregel weiterhin ein Review.

Die technische Konto-Ausnahme unterscheidet nicht nach PR-Autor.
GitHub würde ihre Verwendung durch dieses Konto auch bei fremden PRs zulassen.
Die Projektleitung hat diese Grenze der eingebauten Ausnahme nach ausdrücklichem Hinweis akzeptiert.
Die Beschränkung auf eigene PRs wird deshalb als Nutzungsregel dokumentiert und nicht als technische Sperre behauptet.
Dies erlaubt keine Nutzung der Ausnahme durch den Assistenten ohne gesonderten Merge-Auftrag.

## Getrennte aktive Regelsätze

Beide Regelsätze gelten nur für `refs/heads/main`.

| Regelsatz | Inhalt | Ausnahme |
|---|---|---|
| `25341109` – Basisschutz | Fünf aktuelle Pflichtprüfungen; PR-Zwang; geklärte Review-Gespräche; kein Löschen oder Force-Push | Keine |
| `25346711` – Reviews | Mindestens ein Review; CODEOWNER-Prüfung; Verwerfen alter Genehmigungen; Genehmigung des letzten Pushs durch andere Person | Nur Benutzer-ID `77058548`, `@itsh-neumeier`, im Modus `pull_request` |

Der Basisregelsatz enthält eine PR-Regel ohne eigene Reviewanzahl.
Die genehmigenden Reviewanforderungen werden im zusätzlichen Regelsatz erzwungen.
Damit hebt die Konto-Ausnahme keine Pflichtprüfung und keinen sonstigen Basisschutz auf.
Die Pflichtprüfungen bleiben `structure`, `rust`, `web`, `secrets` und `dependencies`.
Sie sind an GitHub Actions mit Integrationskennung `15368` gebunden und verlangen die aktuelle Basis.
Es gibt keine Ausnahme für eine Administratorrolle, App, Team, Deploy-Schlüssel oder weitere Benutzer.

## Einrichtung und Nachweis

Zuerst wurde der zusätzliche Review-Regelsatz aktiv angelegt und geprüft.
Danach wurde der bisherige gemeinsame Regelsatz zum Basisregelsatz ohne Ausnahme angepasst.
Die erforderlichen Prüfungen blieben während der Umstellung verpflichtend.
Beide Regelsätze wurden nach Einrichtung über die GitHub-API erneut gelesen.
Der Hauptbranch meldet weiterhin `protected: true`.
Die [API-Momentaufnahme](../ai-memory/BRANCH_PROTECTION_STATE.json) dokumentiert beide Antworten.

Die [Basisdatei](../../.github/rulesets/main.json) und die [Reviewdatei](../../.github/rulesets/main-reviews.json) sind versioniert.
Die CI prüft beide Dateien. Negative Tests weisen fremde Konten, pauschale Rollen, direkte Push-Ausnahmen
und Ausnahmen von Pflichtprüfungen ab.
Konfigurationstests und API-Nachweis ersetzen keinen Nachweis einer persönlichen Prüfung oder eines tatsächlich versuchten Merges.
Es wurde kein Merge durchgeführt.

## Verbindliche Nutzung

1. Prüfen Sie Repository, Zielbranch, PR-Autor und exakten Commit.
2. Verwenden Sie die Review-Ausnahme ausschließlich für eigene PRs von `@itsh-neumeier`.
3. Prüfen Sie persönliche Durchsicht, technische Vorprüfung und alle aktuellen Pflichtprüfungen.
4. Dokumentieren Sie den Grund der Ausnahme und den geprüften Commit im PR.
5. Integrieren Sie nur mit gesondertem Merge-Auftrag und erfülltem Basisschutz.

Bei fremden PRs gilt die Ausnahme nach der Nutzungsregel nicht.
Unabhängige Wahlprotokollprüfung, juristische Lizenzprüfung und weitere Fachfreigaben bleiben gesondert erforderlich.
Es wird keine Fachprüfung allein aus Kontoinhaberschaft oder KI-Unterstützung abgeleitet.

## Nicht gewählte Alternative

Eine gesonderte vertrauenswürdige GitHub-App könnte vor einer Ausnahme zusätzlich den PR-Autor prüfen.
Die Projektleitung hat stattdessen die einfachere Konto-Ausnahme mit dokumentierter Nutzungsbeschränkung gewählt.
Es werden keine zusätzliche App, Hostingressourcen, Schlüssel oder Secrets angelegt.
Der ursprüngliche App-Vorschlag bleibt im Git-Verlauf bei Commit `bad4415` nachvollziehbar.
P0 ist nicht insgesamt abgenommen. Keine spätere Fachphase beginnt durch diese Entscheidung.

Quelle: [GitHub-Regelsatz-API: Akteure, Ausnahmemodus und Bedingungen](https://docs.github.com/en/rest/repos/rules?apiVersion=2026-03-10).
