# Dienstvertrag: Authentisierungsanforderung je Aktion v1

- Status: Technische Konkretisierung der ausdrücklich bestätigten MFA-Zuordnung vor Code
- Umfang: KC-006F; Grundlage für die spätere Rechteprüfung in KC-008
- Grundlage: Spezifikation v5.9, Entscheidung vom 21.10.2026 und ADR-043

## Zweck und Herkunft

Der zuständige Dienst bestimmt die Aktion und Datenklasse anhand seiner geprüften Route und des tatsächlichen Objekts.
Browserfelder und eine vom Aufrufer behauptete niedrigere Schutzklasse dürfen diese Einstufung nicht ersetzen.
Der interne Adapter bestimmt daraus die erforderliche aktuelle Authentisierung.
Er erteilt keine Fachberechtigung und ersetzt weder Mitgliedschaft noch Objekt-/Feldrechte.

## Bestätigte Zuordnung

| Aktion oder Datenklasse | Erforderliche Sitzungsprüfung |
|---|---|
| Rollen- und Rechteverwaltung | Aktuelle MFA nach `RecentMultiFactor` |
| Sicherheitsrelevante Einstellungen | Aktuelle MFA nach `RecentMultiFactor` |
| Gesundheitsdaten | Aktuelle MFA nach `RecentMultiFactor` |
| Sorgerechtsbeschränkungen | Aktuelle MFA nach `RecentMultiFactor` |
| Personalakten | Aktuelle MFA nach `RecentMultiFactor` |
| Interne Entwicklungsakten | Aktuelle MFA nach `RecentMultiFactor` |
| Bereits veröffentlichtes Portfolio | Normale aktuelle Sitzungsprüfung; gesonderte Objekt-/Feldrechte bleiben erforderlich |
| Einfache Kontaktdaten | Normale aktuelle Sitzungsprüfung; gesonderte Objekt-/Feldrechte bleiben erforderlich |
| Fehlende, unbekannte oder noch nicht zugeordnete Einstufung | Verweigerung vor jeder positiven Antwort |

Die beiden weniger strengen Einträge gelten ausschließlich für die genannten tatsächlichen Daten.
Ein zusammengefasstes Objekt mit Gesundheitsdaten wird nicht durch die Bezeichnung „Kontaktdaten“ herabgestuft.
Eine Portfolio-Veröffentlichung oder Freigabe wird durch diese Leseeinstufung nicht genehmigt.
Beschwerden und Wahlgeheimnis erhalten in diesem Teil keine neue Freigabe. Unzugeordnete Aktionen bleiben gesperrt.
Die gesonderte Wahlarchitektur und ihre unabhängige Prüfung bleiben erforderlich.

## Laufzeitgrenze

Die öffentliche interne Prüfvariante nimmt eine typisierte Einstufung entgegen.
Sie weist den unbekannten Zustand ab und ruft ansonsten die vorhandene aktuelle Sitzungsprüfung auf.
Sie bietet keinen zusätzlichen Parameter zur Herabstufung der abgeleiteten Anforderung.
MFA besteht aus tatsächlich geprüfter Passwort-/TOTP-Kombination oder Passkey mit Benutzerverifikation.
Die bestehende zusätzliche Prüfung und die aktuellen, gegebenenfalls verkürzten Einrichtungsfristen bleiben maßgeblich.
Eine Anmeldung allein gewährt kein Rollen-, Objekt- oder Feldrecht. Ein positives Ergebnis ist nur eine aktuelle Authentisierungsmomentaufnahme.

Jede geschützte Aktion benötigt weiterhin aktuelle Sitzung, Mitgliedschaft und ihren konkreten Berechtigungsumfang.
Fehler, Widerruf, Wiederherstellungssperre, unvollständige MFA und abgelaufene zusätzliche Authentisierung verweigern Zugriff.
Der interne Adapter erzeugt keinen dauerhaften Berechtigungsschein oder positiven Zwischenspeicher.

## Nachweise

Tests prüfen alle benannten Zuordnungen sowie den unbekannten Zustand.
Echte Datenbanktests binden die neue Prüfvariante an die bestehende aktuelle Sitzungsprüfung.
Sie prüfen normale gegenüber aktueller MFA, Ablauf, zusätzliche Faktorprüfung und Widerruf mit synthetischen Daten.
Die bisherigen tatsächlichen Faktor-/Rotations-/Neustartnachweise bleiben erforderlich.
Keine Migration, neue Rolle, erlaubende Policy oder neue Abhängigkeit entsteht durch die Zuordnung.
HTTP, mTLS und die vollständigen Berechtigungsnachweise aus KC-008 bleiben gesondert.
