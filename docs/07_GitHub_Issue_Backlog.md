# GitHub-Issue-Backlog

Grundlage: Spezifikation v5.9. Prüfstand: 18.10.2026.

Vor P0 enthielt GitHub keine Issues. P0 hat die 35 CSV-Einträge in GitHub-Issues
mit stabiler KC-Kennung und prüfbaren Kriterien übertragen.
KC-036 übernimmt das bislang nicht nummerierte Dokumentationspaket.
KC-037 und KC-038 verfolgen offene P0-Entscheidungen und GitHub-Schutzregeln.

GitHub ist die verbindliche Quelle. Die [Issue-Zuordnung](../backlog/github-issues.json)
enthält Nummer und URL. [Verfeinerte Kriterien](../backlog/refined-issues.json)
sichern den Importstand. Der [ursprüngliche CSV-Backlog](../backlog/issues.csv)
bleibt als historische Importgrundlage erhalten. Er ist kein aktueller Statusbericht.

Die Reihenfolge und die P0-Abnahme stehen im [P0-Plan](15_P0_Initialisierung.md).
Die Phasen aus der [Roadmap](06_Entwicklungsroadmap.md) bleiben verbindlich.
Spätere Issues sind Planung. Sie erteilen keinen Implementierungsauftrag.
Vor Fachimplementierung MÜSSEN breite Module weiter zerlegt werden.
Ihre Sicherheitsgrenzen benötigen konkrete positive und negative Tests.

Das aktuelle Arbeitspaket und seine Fortsetzung stehen im [Projektstatus](ai-memory/CURRENT_STATE.md).
KC-002 hat Dienstgrenzen und Datenhoheit in PR #40 dokumentiert; dieser PR ist integriert.
P0 ist abgenommen. P1 ist ausdrücklich beauftragt. Review- und Integrationsnachweise bleiben erforderlich.

## Teilpakete und Gesamtabnahme

Die Präzisierung vom 20.10.2026 ergänzt KC-004A/B/C und KC-005A/B/C mit ihren GitHub-Zuordnungen.
`depends_on` benennt konkrete technische Voraussetzungen. `work_packages` benennt alle Teilpakete einer Gesamtabnahme.
Die Prüfung durchläuft beide Beziehungen und weist fehlende, doppelte, fremde oder phasenfremde Teilpakete sowie Zyklen ab.
KC-005 und KC-006 benötigen KC-004A/B. Der Abschluss von KC-004C wird dafür nicht vorweggenommen.
KC-004 und KC-005 bleiben bis zu sämtlichen ursprünglichen Kriterien offen.

## Dokumentationssprache

Es gelten die [deutsche Dokumentationsrichtlinie](13_Dokumentationsrichtlinie_DE_ASD_STE100.md)
und das [Fachwortverzeichnis](14_Fachwortverzeichnis_DE.md).
Dies ist eine deutsche Adaption der Sprachprinzipien und keine formelle ASD-STE100-Konformität.
