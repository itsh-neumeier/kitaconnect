# Entwicklungsroadmap und Abhängigkeiten

**KitaConnect Spezifikation v5.9 · 17.10.2026 · Konsolidierter Planungsstand**

> Status: Anforderungsspezifikation, keine implementierte oder rechtsverbindlich geprüfte Software. Die neuesten ausdrücklichen Interviewentscheidungen gelten.

## Veröffentlichungsprinzip

**1.0 ist die erste öffentliche stabile Version und enthält den gesamten vereinbarten Umfang.**
Davor gibt es nur interne Entwicklungsmeilensteine. Öffentliche Beta- und nächtliche Entwicklungsversionen sind ausgeschlossen.
Danach folgen weitere stabile Versionen.
Jede Phase benötigt erfolgreiche Tests und fortlaufende Dokumentation.
Eine Funktion gilt ohne Berechtigungs-, Datenschutz- und Migrationstests nicht als fertig.

CI bezeichnet die automatisierte Integration. ADR bezeichnet eine dokumentierte Architekturentscheidung.
RLS bezeichnet Row-Level Security. MFA bezeichnet Mehrfaktor-Authentisierung.
PITR ist die Wiederherstellung zu einem Zeitpunkt. HA bezeichnet Hochverfügbarkeit.
E2E bezeichnet Ende-zu-Ende-Tests. SBOM bezeichnet die Software-Stückliste.

| Phase | Abhängigkeit | Ergebnis / Abnahmekriterium |
|---|---|---|
| P0 Grundlagen | keine | Monorepo, ADRs, Lizenzentwurf, CI, Docker-/Helm-Grundgerüst, Testdaten |
| P1 Plattformkern | P0 | Identität, MFA, Organisation/Träger, Schemas/RLS, Berechtigungsprüfung, Zugriffsprotokollierung, Modulkonfiguration |
| P2 Kita-Stammdaten | P1 | Kinder, Sorgeberechtigte, Gruppen, Aufnahme, Verträge, Anwesenheit, Rechte |
| P3 Personal & Tagesbetrieb | P2 | Dienstplan, Zeitkonten, Pflege, Übergaben, Notfälle, Elternkommunikation |
| P4 Pädagogik & Medien | P2/P3 | Portfolio, Entwicklung, lokale Gesichtsmaskierung, Medienfreigabe |
| P5 Elternbeirat & Finanzen | P1/P2 | Veranstaltungen, Basare, Kasse, QR, Wahlen, Sitzungen, Wissen, Inventar |
| P6 Speisen & Integrationen | P2/P3 | Menüs, Bestellungen ohne Abrechnung, Caterer, SMTP/Push/ICS/Webhooks |
| P7 Administration & Datenschutz | P1–P6 | Trägerkonfiguration, Arbeitsabläufe, Auskunft/Löschung, Import/Export, Migration |
| P8 Betrieb & Härtung | P0–P7 | Datensicherungen/PITR, Wiederherstellung, HA, Betriebsüberwachung, Last, Sicherheit, Barrierefreiheit, E2E |
| P9 Veröffentlichung 1.0 | P0–P8 | Vollständige Abnahme, CI-Prüfschritte, SBOM, signierte Images, Änderungsprotokoll, Versionsmarke der stabilen Version |

## Sequenzierung und Parallelität

Die P0-Reihenfolge, offene Entscheidungen und die Abnahme stehen im
[P0-Plan](15_P0_Initialisierung.md). Verfeinerte Issue-Abhängigkeiten ergänzen
die Phasenbedingungen dieser Tabelle. Sie ersetzen diese Bedingungen nicht.
P1 ist der kritische Pfad. Ab P2 können unabhängige Dienste parallel entwickelt werden.
Ihre Dienstverträge müssen dabei versioniert und testbar sein.
Zerlegen Sie jede Phase in kleine GitHub-Issues, möglichst mit 0,5–2 Entwicklungstagen je Issue.
Eine KI-Sitzung soll kein vollständiges Modul auf einmal implementieren müssen.

Besonders riskante Bereiche benötigen vorher technische Untersuchungsissues:

- Geheime Hybridwahl, Mandantenschema mit RLS und vollständige Protokollierung von Lesezugriffen.
- Gesichtsmaskierung und mandantenspezifische Wiederherstellung.
- Anmeldung über SAML beziehungsweise OIDC, lokaler oder S3-kompatibler Speicher und Microservice-Betrieb.

SAML und OIDC bezeichnen Verfahren für die angebundene Identitätsverwaltung.

## Abnahme der Version 1.0

Die Abnahme verlangt folgende Nachweise:

- 100 % der vereinbarten Pflichtfunktionen sind prüfbar. Alle kritischen Tests sind erfolgreich.
- Mandantenisolation, Datenschutz und Berechtigungen sind geprüft.
- Hauptprozesse sind barrierefrei. Wiederherstellung und Last werden mit definiertem Szenario getestet.
- Container sind signiert. Veröffentlichungshinweise liegen vor.

Automatische Versionsmarkierung und Veröffentlichung erfolgen ausschließlich nach einem durch Review geschützten Merge.
Alle verpflichtenden CI-Prüfschritte müssen erfolgreich sein.


## Dokumentationssprache (Ergänzung v5.9)
Für dieses Dokument gilt die deutsche Projektrichtlinie `13_Dokumentationsrichtlinie_DE_ASD_STE100.md` mit dem verbindlichen Fachwortverzeichnis `14_Fachwortverzeichnis_DE.md`. Die Richtlinie orientiert sich an ASD-STE100, stellt aber **keine offizielle deutsche ASD-STE100-Konformität** dar.
