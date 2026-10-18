# UX-/UI-Konzept

**KitaConnect Spezifikation v5.9 · 17.10.2026 · Konsolidierter Planungsstand**

> Status: Anforderungsspezifikation, keine implementierte oder rechtsverbindlich geprüfte Software. Die neuesten ausdrücklichen Interviewentscheidungen gelten.

## Nutzererlebnis

Das Design ist familienfreundlich und sachlich. Die Gestaltung beginnt mit mobilen Geräten.
Die Benutzeroberfläche (UI) verwendet SvelteKit, Svelte 5, TypeScript und Tailwind sowie wiederverwendbare Komponenten.
Die Progressive Web App (PWA) unterstützt Android und iOS sowie Deutsch und Englisch.
Für Barrierefreiheit gelten die Web Content Accessibility Guidelines (WCAG) 2.2 auf Stufe AA.
Startseiten richten sich nach der Rolle. Benutzer können ihre Anzeigeelemente anpassen.
Eltern-, Mitarbeitenden-, Beirats-, Leitungs- und Trägerbereich bleiben klar getrennt.
Navigation und Gestaltung sind konfigurierbar. Unzugängliche Farbkombinationen sind ausgeschlossen.

## Informationsarchitektur

| Bereich | Navigation |
|---|---|
| Eltern | Start, Nachrichten, Kalender, Kinder, Formulare, Portfolio, Speiseplan, Termine, Elternbeirat |
| Mitarbeitende | Tagesübersicht, Gruppe, Anwesenheit, Übergabe, Portfolio, Personal |
| Leitung | Aufnahme, Gruppen, Dienstplanung, Freigaben, Datenschutz, Berichte |
| Elternbeirat | Veranstaltungen, Helfende, Basar, Kasse, Sitzungen, Wahl, Wissen |
| Träger | Einrichtungen, Personal, Vorgaben, Berichte, Prüfung von Zugriffsprotokollen |
| Administration | Module, Rollen, Domains, Integrationen, Datensicherungen, Betriebsüberwachung |

## Interaktionsmuster

Geführte Assistenten unterstützen Einrichtung, Aufnahme, Eingewöhnung, Gruppenwechsel und Austritt.
Sie unterstützen außerdem Mandantenmigration und Wiederherstellung.
Formatierte Texte und Portfolios werden automatisch gespeichert. Konflikte werden erkannt.
Die Foto-Maskierung ist für Berührungseingabe auf mobilen Geräten ausgelegt und verlangt eine Vorschau.
Der aktive Organisationskontext bleibt sichtbar. Ein Rollenwechsel zeigt eine deutliche Warnung.
Vor Veröffentlichung wird eine sichere Vorschau angezeigt.
Formulare bieten verständliche Fehlermeldungen, Tastaturbedienung und Beschriftungen für Vorleseprogramme.
Kontrast und Vergrößerung werden getestet.
Offline-Nutzung ist nur für ausdrücklich freigegebene Abläufe mit beherrschbaren Konflikten zulässig.
Sensible Vorgänge erhalten keine unbemerkte Offline-Freigabe.

## Zentrale Benutzerabläufe

Die folgenden Abläufe geben die vorhandene fachliche Reihenfolge wieder.
Die Formulierung zur Berechtigungsprüfung in Ablauf 3 ist als [F-001](19_Redaktionspruefung_KC-036.md) offen.
Sie hebt die serverseitige Prüfpflicht aus ADR-019 nicht auf.

1. Eltern registrieren sich und werden verifiziert. Danach folgen Einrichtungszuordnung, Einwilligungen und Elternstartseite.
2. Die Leitung nimmt ein Kind auf und prüft die sorgeberechtigten Personen.
   Danach folgen Vertrag, Gruppe, Einladung und Eingewöhnung.
3. Eine Fachkraft dokumentiert ein Tagesereignis. Danach folgen Berechtigungsprüfung, optionale Elternfreigabe und Zugriffsprotokollierung.
4. Eine Portfolioseite wird bearbeitet. Danach folgen Gesichtsmaskierung, Einwilligungsprüfung, Freigabe und Veröffentlichung.
5. Der Elternbeirat organisiert einen Basar und vergibt Plätze. Zahlungen werden manuell bestätigt. Danach werden QR-Codes ausgegeben.
6. Der Träger weist eine Springerkraft zu. Sie erhält begrenzten Einrichtungszugriff. Dieser Zugriff wird automatisch entzogen.


## Dokumentationssprache (Ergänzung v5.9)

Es gelten die [deutsche Dokumentationsrichtlinie](13_Dokumentationsrichtlinie_DE_ASD_STE100.md)
und das [Fachwortverzeichnis](14_Fachwortverzeichnis_DE.md).
Die Richtlinie adaptiert Sprachprinzipien von ASD-STE100.
Sie ist keine offizielle deutsche Übersetzung und belegt keine formelle ASD-STE100-Konformität.
