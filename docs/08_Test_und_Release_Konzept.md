# Test- und Release-Konzept

**KitaConnect Spezifikation v5.9 · 17.10.2026 · Konsolidierter Planungsstand**

> Status: Anforderungsspezifikation, keine implementierte oder rechtsverbindlich geprüfte Software. Die neuesten ausdrücklichen Interviewentscheidungen gelten.

## Testpyramide und verpflichtende Prüfschritte

Die automatisierte Integration (CI) und die späteren Abnahmen umfassen:

- Formatierung, statische Codeprüfung, Rust-Clippy, Rust-Tests sowie TypeScript- und Svelte-Prüfungen.
- Komponenten- und Integrationstests sowie Datenbankmigrationstests je Mandantenschema.
- Dienstvertragstests und Ende-zu-Ende-Tests (E2E).
- Rollenbasierte (RBAC), attributbasierte (ABAC) und feldbezogene Berechtigungstests sowie Sorgerechtsfälle.
- Isolationstests zwischen Mandanten und zwischen Trägern.
- Statische Quelltextanalyse (SAST), Abhängigkeitsanalyse (SCA), Secret- und Containerprüfung.
- Software-Stückliste (SBOM) und Prüfung der Lizenzanforderungen.
- Leistungs- und Lasttests sowie Barrierefreiheit nach WCAG 2.2 AA.
- Mobile Nutzung der Progressive Web App (PWA).
- Datensicherung, Wiederherstellung, Wiederherstellung zu einem Zeitpunkt (PITR) und Rücksetzverfahren.

WCAG bezeichnet die Web Content Accessibility Guidelines.
CI verwendet ausschließlich synthetische Testdaten und keine echten Kinderdaten.

## Sicherheitskritische Prüfungen

Folgende Testbereiche blockieren bei Fehlern die Freigabe:

- Mandantenisolation und Wahlgeheimnis.
- Zugriff auf Gesundheitsdaten und Personalakten sowie Widerruf von Einwilligungen.
- Notfallzugriff und Finanztransaktionen.
- Passwort- und Sitzungssicherheit sowie unsignierte oder verwundbare Images.

Kritische Änderungen benötigen genehmigende Pull-Request-Reviews vor einer automatischen Veröffentlichung.

## Automatisierte Veröffentlichung stabiler Versionen

1. Schützen Sie `main`. Verlangen Sie mindestens ein Review und CODEOWNERS für Sicherheit und Datenschutz.
2. Prüfen Sie CI, Integration, E2E, Dienstverträge, Mandantenisolation, Lastgrenzen, Sicherheit und Barrierefreiheit.
3. Prüfen Sie Migrationskompatibilität und Wiederherstellung. Prüfen Sie die Veröffentlichungscheckliste maschinell.
4. Berechnen Sie die Version und setzen Sie ihre Versionsmarke.
   Erstellen Sie Änderungsprotokoll und SBOM. Signieren Sie Images und veröffentlichen Sie diese in der GitHub Container Registry (GHCR).
   Erstellen Sie Veröffentlichungshinweise.
5. Bei einem fehlgeschlagenen Prüfschritt erfolgt weder Versionsmarkierung noch Veröffentlichung.
   Öffentliche Vorschauversionen und nächtliche Entwicklungsartefakte sind ausgeschlossen.

## Qualitätskennzahlen (initiale Zielwerte, vor Release festzulegen)

Die Projektleitung hat das [Einrichtungsprofil in KC-037](17_P0_Entscheidungsregister.md) ausdrücklich bestätigt.
Es legt 50 Sitzungen, 10 Anfragen pro Sekunde, p95-Ziele, Fehlerrate und Referenzressourcen fest.
KC-034 prüft diese Ziele nach Implementierung repräsentativer Abläufe.
Die unabhängige Prüfung des Wahlprotokolls bleibt gemäß ADR-017 vor dessen Implementierung erforderlich.
Zur Abnahme dürfen keine bekannten kritischen Sicherheitslücken oder ungeklärten kritischen Fehler vorliegen.
Alle verbindlichen Testfälle müssen erfolgreich sein.
P0 verlangt messbare Leistungsziele und Lastszenarien statt unbelegter Zahlen.
Das verknüpfte Einrichtungsprofil dokumentiert die inzwischen bestätigten Zielwerte.


## Dokumentationssprache (Ergänzung v5.9)

Es gelten die [deutsche Dokumentationsrichtlinie](13_Dokumentationsrichtlinie_DE_ASD_STE100.md)
und das [Fachwortverzeichnis](14_Fachwortverzeichnis_DE.md).
Die Richtlinie adaptiert Sprachprinzipien von ASD-STE100.
Sie ist keine offizielle deutsche Übersetzung und belegt keine formelle ASD-STE100-Konformität.
