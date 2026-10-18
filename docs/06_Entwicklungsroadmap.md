# Entwicklungsroadmap und Abhängigkeiten

**KitaConnect Spezifikation v5.9 · 17.10.2026 · Konsolidierter Planungsstand**

> Status: Anforderungsspezifikation, keine implementierte oder rechtsverbindlich geprüfte Software. Die neuesten ausdrücklichen Interviewentscheidungen gelten.

## Releaseprinzip
**1.0 ist die erste öffentliche Stable und enthält den gesamten vereinbarten Umfang.** Davor nur interne Entwicklungsmeilensteine, keine öffentliche Beta/Nightly. Danach zusätzliche stabile Versionen. Alle Phasen mit grünen Tests und fortlaufender Dokumentation; kein Feature gilt ohne Berechtigungs-, Datenschutz- und Migrationstests als fertig.

| Phase | Abhängigkeit | Ergebnis / Exit-Gate |
|---|---|---|
| P0 Grundlagen | keine | Monorepo, ADRs, Lizenzentwurf, CI, Docker/Helm-Skeleton, Testdaten |
| P1 Plattformkern | P0 | Identity, MFA, Organisation/Träger, Schemas/RLS, Policy-Engine, Audit, Modulkonfiguration |
| P2 Kita-Stammdaten | P1 | Kinder, Sorgeberechtigte, Gruppen, Aufnahme, Verträge, Anwesenheit, Rechte |
| P3 Personal & Tagesbetrieb | P2 | Dienstplan, Zeitkonten, Pflege, Übergaben, Notfälle, Elternkommunikation |
| P4 Pädagogik & Medien | P2/P3 | Portfolio, Entwicklung, lokale Gesichtsmaskierung, Medienfreigabe |
| P5 Elternbeirat & Finanzen | P1/P2 | Events, Basare, Kasse, QR, Wahlen, Sitzungen, Wissen, Inventar |
| P6 Speisen & Integrationen | P2/P3 | Menüs, Bestellungen ohne Abrechnung, Caterer, SMTP/Push/ICS/Webhooks |
| P7 Administration & Datenschutz | P1–P6 | Trägerkonfiguration, Workflows, Auskunft/Löschung, Import/Export, Migration |
| P8 Betrieb & Härtung | P0–P7 | Backups/PITR, Restore, HA, Monitoring, Last, Security, Accessibility, E2E |
| P9 Release 1.0 | P0–P8 | Vollständige Abnahme, CI-Gates, SBOM, signierte Images, Changelog, Stable-Tag |

## Sequenzierung und Parallelität
P1 ist der kritische Pfad. Ab P2 können unabhängige Dienste parallel entwickelt werden, solange API-Verträge versioniert und testbar sind. Jede Phase in kleine GitHub-Issues zerlegen (idealerweise 0,5–2 Entwicklungstage pro Issue); keine KI-Sitzung soll ein ganzes Modul auf einmal implementieren müssen. Technische Spike-Issues vor besonders riskanten Bereichen: geheime Hybridwahl, Mandantenschema+RLS, vollständige Lesezugriff-Audits, Face-Masking, mandantenspezifischer Restore, SAML/OIDC, lokaler/S3-Speicher, Microservice-Betrieb.

## Release-1.0-Abnahme
100 % vereinbarte Must-Funktionen nachprüfbar; alle kritischen Tests grün; Mandantenisolation nachgewiesen; Datenschutz- und Rechteprüfung; barrierefreie Hauptprozesse; Restore-Test; Load-Test mit definiertem Szenario; signierte Container und Release Notes. Automatisches Tag/Publish ausschließlich nach Review-geschütztem Merge und vollständigem CI-Gate.


## Dokumentationssprache (Ergänzung v5.9)
Für dieses Dokument gilt die deutsche Projektrichtlinie `13_Dokumentationsrichtlinie_DE_ASD_STE100.md` mit dem verbindlichen Fachwortverzeichnis `14_Fachwortverzeichnis_DE.md`. Die Richtlinie orientiert sich an ASD-STE100, stellt aber **keine offizielle deutsche ASD-STE100-Konformität** dar.
