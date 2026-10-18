# ADR-021 – Aktiver Main-Regelsatz ohne Merge-Ausnahmen

Datum: 18.10.2026. Status: ursprüngliche Einrichtung; durch ADR-022 fortgeschrieben.
Grundlage: ADR-003, ADR-013 und das Test- und Release-Konzept v5.9.

## Entscheidung

Ein GitHub-Regelsatz schützt ausschließlich `main`.
Er verlangt ein genehmigendes Review, CODEOWNER-Prüfung, Genehmigung des letzten Pushs durch eine andere Person,
Verwerfen veralteter Reviews und geklärte Review-Gespräche.
Die fünf bestehenden Prüfungen sind an GitHub Actions gebunden und müssen zur aktuellen Basis passen.
Der Regelsatz verhindert Löschen und Force-Push. Seine Ausnahmeliste ist leer.
Der Plan wird versioniert und mit negativen Tests gegen Abschwächungen geprüft.
Die CI verändert keine Verwaltungseinstellungen.

## Alternative und Folgen

Fortschreibung: [ADR-022](ADR-022_Reviewausnahme_eigene_PRs.md) beschreibt die später ausdrücklich gewünschte
Konto-Ausnahme mit verbindlicher Nutzung ausschließlich für eigene PRs von `@itsh-neumeier`.
Die Projektleitung hat die technisch weitergehende Konto-Ausnahme ausdrücklich gewählt.
Die aktuelle Einrichtung trennt Basisschutz ohne Ausnahme und Review-Regeln mit Konto-Ausnahme.
Die folgenden ursprünglichen Aussagen dokumentieren den Stand vor dieser bestätigten Änderung.

CODEOWNERS allein erzwingt kein Review. Ein bloßer lokaler Plan belegt keinen Schutz.
Eine Ausnahme für Administratoren würde die verlangten Prüfungen umgehen und wird nicht eingerichtet.
Die [API-Momentaufnahme](../ai-memory/BRANCH_PROTECTION_STATE.json) belegt Regelsatz `25341109`.
Administratoren können Regeln weiterhin verwalten. Änderungen benötigen einen neuen Prüf- und API-Nachweis.

Die bestätigte CODEOWNER-Datei fehlt bis zur Integration von PR #42 auf `main`.
Ein Autor kann seine eigene CODEOWNER-Prüfung nicht erfüllen.
Ohne weitere bestätigte Zuständigkeit bleiben solche Merges gesperrt.
Die Projektleitung muss die unabhängigen Reviewer benennen. Diese Entscheidung ernennt keine Fachprüfer.
Auf Arbeitsbranches gerichtete Folge-PRs müssen vor Integration auf `main` umgestellt und erneut geprüft werden.
Die [Betriebsanleitung](../18_Branchschutz_und_Reviews.md) beschreibt Nachweise, Sperren und Integrationsfolge.
Es gibt keine Veröffentlichung und keine Umsetzung einer weiteren Entwicklungsphase.
