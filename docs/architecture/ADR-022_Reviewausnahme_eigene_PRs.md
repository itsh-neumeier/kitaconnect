# ADR-022 – Reviewausnahme ausschließlich für eigene PRs

Datum: 18.10.2026. Bezug: KC-038 und ADR-021.
Status: Geltungsbereich ausdrücklich bestätigt; technische Umsetzung noch offen.

## Bestätigte Anforderung

Die Projektleitung bestätigt die Rückfrage mit „Ja genau“.
Nur `@itsh-neumeier` soll bei selbst erstellten Pull Requests auf ein weiteres genehmigendes Review verzichten können.
Fremde PRs benötigen weiterhin Review. CODEOWNER-Zugehörigkeit allein gewährt keine allgemeine Ausnahme.
Persönliche Prüfung durch die Projektleitung und technische KI-Vorprüfung bleiben vorgesehen.
Die Ausnahme betrifft die zusätzliche genehmigende PR-Prüfung.
Sie ersetzt weder unabhängige Wahlprotokollprüfung noch rechtliche Fachprüfungen.

Die fünf Pflichtprüfungen `structure`, `rust`, `web`, `secrets` und `dependencies` bleiben verbindlich.
Sie müssen sich auf den aktuellen Commit und die aktuelle Basis beziehen.
Direktes Schreiben auf `main`, Force-Push und Löschen werden durch die Reviewausnahme nicht erlaubt.
Review-Gespräche müssen weiterhin geklärt sein.
Ein Merge benötigt weiterhin einen gesonderten Auftrag.

## Grenze der eingebauten GitHub-Regeln

Die Regelsatz-API beschreibt Ausnahmen für Benutzer, Rollen und weitere Akteure.
Eine Ausnahme im Modus `pull_request` beschränkt die Nutzung auf PRs.
Die dokumentierten Bedingungen eines Branch-Regelsatzes wählen Zielbranches, nicht den PR-Autor.
Die Schnittstelle bietet keine Bedingung „handelnder Benutzer ist zugleich PR-Autor“.
Die Folgerung aus diesem Schema ist: Eine allgemeine Benutzerausnahme bildet die bestätigte Beschränkung nicht vollständig ab.

Eine Ausnahme im bisherigen gemeinsamen Regelsatz könnte außerdem Pflichtprüfungen betreffen.
Deshalb wird weder dort eine Benutzerausnahme eingetragen noch die Anzahl erforderlicher Reviews pauschal auf null gesetzt.
Der tatsächliche Regelsatz `25341109` bleibt bis zu einem passenden Umsetzungsnachweis unverändert.
ADR-021 dokumentiert damit weiterhin den aktiven Schutz. Dieses Dokument ergänzt die gewünschte Zielregel.

## Konkreter Umsetzungsvorschlag zur Entscheidung

Ein gesonderter, vertrauenswürdig betriebener GitHub-App-Dienst kann ausschließlich erlaubte Merges ausführen.
Der allgemeine Regelsatz behält Pflichtprüfungen, PR-Zwang sowie Lösch- und Force-Push-Schutz ohne Ausnahme.
Die Reviewanforderungen liegen in einem zusätzlichen Regelsatz.
Nur die App erhält dort eine Ausnahme im Modus `pull_request`. Der Benutzer selbst erhält keinen allgemeinen Bypass.

Die App prüft vor jeder Ausnahme die von GitHub bestätigte Repository-, Benutzer- und PR-Autoridentität.
Sie akzeptiert ausschließlich einen Auftrag von `@itsh-neumeier` für einen von demselben Benutzer erstellten PR.
Sie bindet die Aktion an PR-Nummer, Ziel `main` und exakten Commit.
Sie prüft CI-Stand, aktuelle Basis und erneute Änderungen. Bei Fehler oder unklarem Zustand verweigert sie den Merge.
Sie führt keinen Quelltext aus dem zu prüfenden PR aus und übernimmt keine Berechtigungsentscheidung aus dessen Dateien.
Jede Ausnahme erhält einen nachvollziehbaren Nachweis von Auftrag, Prüfungen und Merge.

Diese Variante benötigt eine zusätzliche GitHub-App, geschützten Betrieb und Zugangsdaten außerhalb des Repositorys.
Sie ist ein Vorschlag und noch keine bestätigte Betriebsentscheidung.
Es wurden keine App, Schlüssel, Secrets, Hostingressourcen oder neuen Schreibberechtigungen angelegt.
Eine normale CI-Erfolgsmeldung wird nicht als Nachweis einer solchen vertrauenswürdigen Merge-Steuerung ausgegeben.

## Erforderliche Abnahmefälle

| Fall | Erwartetes Ergebnis |
|---|---|
| Benutzer und PR-Autor sind `itsh-neumeier`; aktueller Stand vollständig geprüft | Ausnahme darf auf ausdrücklichen Merge-Auftrag angewendet werden |
| `itsh-neumeier` möchte fremden PR ohne Review integrieren | Keine Ausnahme |
| Andere Person möchte eigenen PR ohne Review integrieren | Keine Ausnahme |
| PR-Autor oder Auftraggeber fehlt beziehungsweise ist nicht überprüfbar | Keine Ausnahme |
| Mindestens eine Pflichtprüfung fehlt, läuft noch oder ist fehlgeschlagen | Merge gesperrt |
| Commit oder Basis ändern sich nach Prüfung | Erneute Prüfung; bisheriger Nachweis reicht nicht |
| Direkter Push, Force-Push oder Löschen | Keine Berechtigung durch diese Ausnahme |
| Fremder PR enthält Änderungen an der Prüfsteuerung | Keine Übernahme dieser Änderungen in die vertrauenswürdige Entscheidung |

## Fortsetzung

Vor Einrichtung muss die Projektleitung die zusätzliche App und den vorgesehenen Betriebsweg auswählen.
Danach werden Rechte, Prüfungen, Ausfallverhalten und API-Nachweise konkret umgesetzt und getestet.
Bis dahin bleibt die gewünschte Ausnahme technisch unerfüllt. Es gibt keine eigenmächtige Erweiterung auf fremde PRs.
P0 ist nicht insgesamt abgenommen. Keine spätere Fachphase beginnt durch diese Entscheidung.

Quelle: [GitHub-Regelsatz-API: Akteure, Ausnahmemodus und Bedingungen](https://docs.github.com/en/rest/repos/rules?apiVersion=2022-11-28).
Die App-Variante ist ein Projektvorschlag, keine Zusicherung der GitHub-Dokumentation.
