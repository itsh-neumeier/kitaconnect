# Interner Vertrag: Sitzungsfristen je Einrichtung v1

Stand: 21.10.2026. KC-006F / #67. Grundlage: Spezifikation v5.9, ADR-014/034/035.
Die Projektleitung bestätigt die Einstellbarkeit pro Einrichtung und anschließend ausdrücklich die vorgeschlagenen Voreinstellungen und Obergrenzen.

## Umfang und Datenhoheit

organization-config speichert die Einrichtungseinstellungen. identity-access bleibt für Sitzungen und Anmeldenachweise zuständig.
Das Gateway erhält keine Datenbankrolle. Ein gespeicherter Konfigurationswert erteilt keine Berechtigung.
Der interne Speicheradapter erhält eine bereits geprüfte Verbindung und eine typisierte Einrichtungskennung.
Er öffnet keine Verbindung, ändert keine Rolle und startet keine Migration.
Eine öffentliche Einstellungs-API und die Verwaltungsberechtigungen folgen gesondert über die bestätigten Dienstgrenzen und KC-008.

## Bestätigte Werte

| Feld | Einheit | Voreinstellung und Obergrenze |
|---|---|---|
| Inaktivität | Sekunden | 900 |
| Gesamtlaufzeit | Sekunden | 28.800 |
| Zusätzliche Authentisierung | Sekunden | 300 |

Jeder Wert muss eine positive ganze Zahl sein. Eine Einrichtung darf kleinere Werte wählen.
Inaktivität und Gültigkeit der zusätzlichen Authentisierung dürfen die Gesamtlaufzeit nicht überschreiten.
Der Adapter stellt einen ausdrücklich wählbaren Voreinstellungssatz bereit. Er legt keine Konfiguration beim Lesen an.
Fehlende oder wegen RLS unsichtbare Einstellungen ergeben keinen konfigurierten Satz. Datenbankfehler bleiben Fehler.
Die spätere geschützte Anmeldung muss bei fehlender erforderlicher Konfiguration Zugriff verweigern.

## Versionierter Bestand und Konkurrenz

Ein aktueller Satz gehört genau zu einer vorhandenen Einrichtung und besitzt eine positive Versionsnummer.
Die Erstfassung erhält Version 1. Der Adapter unterscheidet ausdrücklich Neuanlage und Änderung mit erwarteter Version.
Eine Änderung schreibt alle drei Werte gemeinsam und erhöht die Version genau um eins.
Bei zwei konkurrierenden Änderungen derselben erwarteten Version darf höchstens eine Änderung erfolgreich sein.
Veraltete Versionen, doppelte Neuanlage und Versionsüberlauf werden abgewiesen.
Ein Datenbanktrigger verhindert Kennungswechsel, Versionsrücksetzung und Änderungen ohne Versionsfortschritt.
Diese Versionskennung ist kein vollständiges Änderungsprotokoll und keine Benutzerfreigabe.

Der Adapter arbeitet außerhalb aufrufender Transaktionen. Auch rohe äußere SQL-Transaktionen sind kein zulässiger Aufrufkontext.
Ein erfolgreicher Schreibaufruf benötigt eine bestätigte eigene Operation mit genau einer betroffenen Zeile.
Ein unterdrücktes Update darf keinen positiven Rückgabewert erzeugen.
Werte und Kennungen werden als Parameter gebunden. Tabellen sind vollständig qualifiziert.
Die Fehlerdarstellung enthält keine rohen Datenbankdetails.

## Migration und Sicherheitsumfang

Eine neue Organisationsmigration 0003 ergänzt nur den aktuellen Einstellungssatz und den Schutztrigger.
Sie benötigt Versionen 1 und 2 und verwendet deren gemeinsame Migrationssperre.
Bestehende Migrationen und Identitätsbestände bleiben unverändert.
Der Bestand erhält FORCE RLS ohne erlaubende Laufzeit-Policy. Laufzeitrechte erlauben keinen Kennungswechsel und kein Löschen.
Positive Tests verwenden ausschließlich den isolierten Test-Betriebszugang und synthetische Einrichtungen.
Die allgemeine Migrationsteuerung und produktive Rollen-/Poolbindung werden dadurch nicht freigegeben.

## Wirkung auf spätere Sitzungsprüfung

Der Speicheradapter selbst ändert keine Sitzung und stellt keinen Anmeldenachweis aus.
Der spätere Anmeldeabschluss muss die aktuelle Einrichtungskonfiguration über den geprüften Organisationsvertrag beziehen.
Eine Verschärfung muss bei der nächsten geschützten Prüfung berücksichtigt werden.
Eine nachträgliche Lockerung darf keine bei Ausstellung festgelegte Frist einer bestehenden Sitzung oder Zusatzprüfung verlängern.
Der spätere Sitzungsvertrag muss dafür ursprüngliche Frist, aktuellen Satz, Einrichtung und Version binden und konkurrierende Änderungen prüfen.
Der Konfigurationsspeichertest ersetzt diesen späteren Integrationsnachweis nicht.

## Nachweise

- Positive und negative Werte, bestätigte Grenzen, Einheiten und Voreinstellungssatz.
- Getrennte Sätze für verschiedene Einrichtungen, unveränderliche Zuordnung und unbekannte Einrichtung.
- Konkurrenz mit tatsächlich wartenden Datenbankverbindungen, veraltete Version und genau ein Erfolg.
- Atomare Rücknahme, unterdrücktes Update, Versionsüberlauf sowie Versions- und Fristerhalt nach Neustart.
- Rücknahme der neuen Migration, saubere Wiederholung, Runtime-/Eigentümer-RLS und begrenzte Fehlerdarstellung.

Die Gesamtkriterien von KC-006F bleiben bis zur Anmelde-, Gateway- und Dienstintegration offen.
