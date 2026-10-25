# ADR-056: Rechtegrundlage vor Anmeldegesamtabnahme

- Datum: 25.10.2026
- Status: Technische Ablaufentscheidung; keine neue Fachregel
- Umfang: KC-008 und KC-006F innerhalb der beauftragten Phase P1

## Entscheidung

Der [P1-Plan](../28_P1_Rechtegrundlage_und_Anmeldeadapter.md) trennt KC-008 in einen technischen Vorlauf
und die vollständigen ursprünglichen Rechteprüfungen.
KC-008A benötigt KC-005 und die Faktorgrundlagen KC-006C/D/E.
KC-006F darf diese Grundlage verwenden. KC-008B und die Gesamtabnahme von KC-008 behalten KC-006 als Voraussetzung.
Der Backlog bildet diese Kanten ausdrücklich ab und prüft sie auf Kreise.

## Begründung und Grenzen

Die aktuelle Rechte-/MFA-Quelle wird für den vollständigen Anmeldeadapter benötigt.
Eine gegenseitige Abhängigkeit der beiden Gesamtabnahmen würde die Umsetzung verhindern.
Die Zerlegung erhält sämtliche ursprünglichen Akzeptanzkriterien.
Die bestätigte MFA-Zuordnung und die Datenhoheit bleiben unverändert.
Der konkrete Vertrag und die aktuelle Bestandsableitung von KC-008A folgen vor dessen Implementierung.
Eine bloße falsche MFA-Behauptung ersetzt keine unbekannte Einstufung.
Dieser Plan enthält keine Rollenvergabe, Migration oder neue Fachfreigabe.
