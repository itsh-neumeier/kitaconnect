# UX-/UI-Konzept

**KitaConnect Spezifikation v5.9 · 17.10.2026 · Konsolidierter Planungsstand**

> Status: Anforderungsspezifikation, keine implementierte oder rechtsverbindlich geprüfte Software. Die neuesten ausdrücklichen Interviewentscheidungen gelten.

## Nutzererlebnis
Familienfreundliches, sachliches Design, mobil zuerst, SvelteKit/Svelte 5/TypeScript/Tailwind, wiederverwendbare UI-Komponenten, PWA für Android/iOS, Deutsch und Englisch, WCAG 2.2 AA. Rollenbezogene Startseiten und personalisierbare Widgets; klare Trennung von Eltern-, Mitarbeitenden-, Beirats-, Leitungs- und Trägerbereich. Konfigurierbare Navigation und Branding ohne unzugängliche Farbkombinationen.

## Informationsarchitektur
Eltern: Start, Nachrichten, Kalender, Kinder, Formulare, Portfolio, Speiseplan, Termine, Elternbeirat. Mitarbeitende: Tagesübersicht, Gruppe, Anwesenheit, Übergabe, Portfolio, Personal. Leitung: Aufnahme, Gruppen, Dienstplanung, Freigaben, Datenschutz, Berichte. Elternbeirat: Events, Helfende, Basar, Kasse, Sitzungen, Wahl, Wissen. Träger: Einrichtungen, Personal, Vorgaben, Berichte, Audit. Admin: Module, Rollen, Domains, Integrationen, Backups, Monitoring.

## Interaktionsmuster
Wizards für Einrichtung, Aufnahme, Eingewöhnung, Gruppenwechsel, Austritt, Mandantenmigration und Restore. Autosave mit Konflikterkennung in Rich-Text/Portfolio; mobile touchoptimierte Foto-Maskierung mit verpflichtender Vorschau. Sichtbarer aktiver Organisationskontext, deutliche Warnung beim Rollenwechsel, sichere Vorschau vor Veröffentlichung. Formulare mit verständlichen Fehlermeldungen, Tastaturbedienbarkeit, Screenreader-Labels, Kontrast- und Zoomtests. Offline nur bei ausdrücklich freigegebenen, konfliktbeherrschbaren Abläufen; keine stille Offline-Freigabe sensibler Vorgänge.

## Kernjourneys
1. Eltern registrieren -> verifizieren -> Einrichtung zuordnen -> Einwilligungen -> Elternstartseite.
2. Leitung Kind aufnehmen -> Sorgeberechtigte prüfen -> Vertrag -> Gruppe -> Einladung -> Eingewöhnung.
3. Fachkraft Tagesereignis dokumentieren -> Berechtigung prüfen -> optional Elternfreigabe -> Audit.
4. Portfolioseite bearbeiten -> Gesichter maskieren -> Einwilligungen prüfen -> Freigabe -> Veröffentlichung.
5. Elternbeirat Basar organisieren -> Plätze vergeben -> Zahlungen manuell bestätigen -> QR ausgeben.
6. Träger Springer zuweisen -> begrenzten Einrichtungszugriff gewähren -> automatisch entziehen.


## Dokumentationssprache (Ergänzung v5.9)
Für dieses Dokument gilt die deutsche Projektrichtlinie `13_Dokumentationsrichtlinie_DE_ASD_STE100.md` mit dem verbindlichen Fachwortverzeichnis `14_Fachwortverzeichnis_DE.md`. Die Richtlinie orientiert sich an ASD-STE100, stellt aber **keine offizielle deutsche ASD-STE100-Konformität** dar.
