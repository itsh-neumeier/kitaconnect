# Fachkonzept

**KitaConnect Spezifikation v5.9 · 17.10.2026 · Konsolidierter Planungsstand**

> Status: Anforderungsspezifikation, keine implementierte oder rechtsverbindlich geprüfte Software. Die neuesten ausdrücklichen Interviewentscheidungen gelten.

## 1. Produktvision und Betriebsmodelle
KitaConnect ist eine deutsch-/englischsprachige, responsive, barrierearme Self-Hosting-PWA für Elternbeiräte, Kindertageseinrichtungen, Eltern, Mitarbeitende und Träger. Betriebsmodi: (A) nur Elternbeirat ohne Kinderakte, (B) nur Einrichtung, (C) kombiniert. Wechsel nur autorisiert, ohne unbemerkte Rechteausweitung oder Datenlöschung. Mehrere unabhängige Einrichtungen und mehrere Träger können in einer Installation koexistieren. Funktionsumfang und Unterfunktionen pro Einrichtung konfigurierbar, verbindliche Sicherheitsgrenzen nicht abschaltbar.

## 2. Nutzer, Organisation und Zuständigkeiten
Systemadministrator, Trägeradministrator, Einrichtungsleitung/-administrator, Personalverwaltung, pädagogische Mitarbeitende, Eltern/Sorgeberechtigte, Elternbeiratsvorsitz/-mitglieder, Wahlleitung, Kassenführung, externe Helfer, Küchen-/Catererrollen, datenschutzverantwortliche Rollen. Ein Benutzer kann mehrere Mitgliedschaften/Rollen besitzen; konfigurierbare Kontomodelle pro Einrichtung. Getrennte fachliche Datenverantwortlichkeiten; Elternbeirat erhält keinen impliziten Zugriff auf Kinderakten, Gesundheits-, Personal- oder Entwicklungsdaten. Trägerweite Personalverwaltung, Springerpool, gemeinsame Vorlagen und Berichte mit begrenzten expliziten Zugriffsrechten.

## 3. Elternbeirat und gemeinsame Prozesse
Elternkommunikation, Newsfeed, Termine, Veranstaltungen, Helfer- und Mitbringlisten, Basare/Standbuchungen, manuelle PayPal-Abgleiche, Kassenbuch und Belege, QR-Wertmarken, geheime Online-/Papier-/Hybridwahlen, Sitzungen/Beschlüsse, Wissensdatenbank, Inventar, Umfragen, Beschwerden, Tauschbörse, Formulare, Elternsprechstunden, Einwilligungen, Krisenkommunikation und Berichte. Gemeinsam freigebbare Veranstaltungen, Dokumente, Projekte, Umfragen, Kalender, Kontaktverzeichnis und Benachrichtigungen; Freigaben zweckgebunden und nachvollziehbar. Finanzen der Einrichtung und des Elternbeirats getrennt.

## 4. Kinder, Familien und Aufnahme (Interview 42,45)
Alle 18 Kinderverwaltungsfunktionen und 20 Lebenszyklusprozesse: Kinderstammdaten, Sorgeberechtigte mit individuellen Rechten, Gruppen/Altersbereiche, Betreuungsverträge/Buchungszeiten, digitale Anwesenheit, Krank-/Abwesenheitsmeldungen, Abholberechtigungen, Notfallkontakte, notwendige Gesundheitsinformationen, Medikamentengabe/Einwilligung, geschützte Dokumente, Geschwister/mehrere Einrichtungen, Im-/Export, Aufnahme-/Änderungsformulare, Änderungshistorie. Digitale Vormerkung/Warteliste/Platzvergabe, Aufnahme, Eingewöhnungsmodelle/-dokumentation, automatische Eltern-Einladung, Bring-/Abholprozess, Tagesdokumentation, Pflege/Wickeln/Schlaf, Übergaben, Unfälle, Elterngespräche, Gruppen-/Schuljahreswechsel, Vorschule, geführter Austrittsassistent und konfigurierbare Workflows. Umfang je Betriebsmodell/Einrichtung und Tagesdokumentation je Altersgruppe konfigurierbar; datensparsame Defaults.

## 5. Personal und Dienstorganisation (Interview 40)
Alle 18 Funktionen: Personalstamm/Beschäftigungsmodelle, Dienstpläne/Schichten/Vorlagen, Wünsche/Verfügbarkeiten, Urlaub/Krankheit, Tausch/Vertretung, Arbeitszeit (manuell/Stempeluhr/beides), Zeitkonten/Mehrarbeit, genehmigte Korrekturen, Besetzungsplanung und Unterbesetzungswarnungen, Qualifikationen/Fortbildungen, Sitzungen/Gespräche, Übergabebuch, Mitarbeiterbereich, Auswertungen, Historie/Freigaben. Automatische Dienstplanvorschläge sind Empfehlungen, Freigabe durch zuständige Person. Regionale und trägerspezifische Betreuungsschlüssel als versionierte Regelwerke; Rechtsgültigkeit separat prüfen. Trägerübergreifende Verwaltung nach minimalen Rechten.

## 6. Portfolios, Pädagogik und Medien (Interview 37a,39)
Alle 17 Portfolio-/Entwicklungsfunktionen: Kindportfolio, frei gestaltbare Drag-and-drop-Seiten, Vorlagen, Gruppenrückblicke, Beobachtungen, strukturierte Entwicklungsbögen, Ziele, Meilensteine/Lerngeschichten, Foto/Video/Audio, Freigabe, Elternansicht/-feedback, Kinderbeiträge, PDF/Druck, Versionshistorie, Gruppenwechsel. Portfolio und vertrauliche Entwicklungsakte strikt logisch getrennt. Freigaben nach Einrichtung/Gruppe/Inhalt/Rolle. Lokale Gesichtserkennung ohne Identitätsmatching; erkannte Gesichter zunächst maskieren, manuell korrigierbar, zwingende Prüfung vor Veröffentlichung; EXIF/GPS entfernen, Einwilligung bei jeder Wiederverwendung prüfen. Keine automatisierte KI-Bewertung kindlicher Entwicklung.

## 7. Essen und Verpflegung (Interview 38)
Wochen-/Monatspläne, Vorlagen, Allergene/Zusatzstoffe, Nährwerte/Zutaten, Bilder, Bestellungen, Abmeldungen/Fristen, automatische Essenszahlen, besondere Kostformen, Küchen-/Catereransicht, Preisanzeige/Kostenberichte, PDF/Excel/CSV, Erinnerungen, Feedback, mehrere Anbieter. Eigene Küche, Caterer oder Mischform. Bestellung deaktivierbar, einzeln oder Abo mit Abmeldungen. **Keine Rechnungsstellung, Forderungsverwaltung oder Zahlungsabwicklung im Essensmodul**; Preisangaben sind informativ/auswertbar. Gesundheitsbezogene Kostangaben besonders geschützt.

## 8. Träger und gemeinsame Prozesse (Interview 41,43)
16 Trägerfunktionen: Trägerhierarchie, zentrale Administration, Personal/Vertretung, Besetzung, Vorlagen, Vorgaben, Branding, Berichte, Termine, Kommunikation, Dokumente, getrennte Finanzen, Audit, Personalwechsel, zentrale Im-/Exporte. Mehrere Träger und trägerlose Einrichtungen. Konfigurierbare Trägerrechte mit verbindlichen Mindeststandards. Kinder-/Familiendatentransfers nur mit Freigaben und rechtlicher Prüfung. 17 gemeinsame Kita-/Elternbeiratsfunktionen mit kontrollierter Freigabe und konfigurierbarer gemeinsamer/getrennter Elternansicht.

## 9. Datenschutz und Sicherheit (Interview 44,48)
Alle 20 Datenschutzfunktionen: Datenkategoriebezogene Fristen, automatisierte Prüf-/Löschjobs, Auskunft/Export/Berichtigung/Löschung, Einwilligungen, Einzelbildfreigabe, Schutz für Gesundheit/Sorgerecht/Entwicklung/Personal, vollständige Lese-/Schreibzugriffsprotokollierung, begründeter zeitbegrenzter Notfallzugriff mit Zweitfreigabe soweit möglich, Zusatzverschlüsselung, Backup-Löschabgleich, Vorfallmanagement, Datenschutzberichte, Hinweise/Vereinbarungen, vollständiger Mandantenexport, Rezertifizierung, datensparsame Defaults. Fristen sind rechtlich zu validieren; keine universellen Fristen erfinden. Alle 20 Identitätsfunktionen: Passwort, Passkeys, TOTP, Recovery, OIDC/OAuth2, SAML, Einladungen/Selfservice, Mehrfachrollen, Wechsel, RBAC+Objekt-/Feldregeln (Default-Deny), temporäre Rechte, Vier-Augen-Freigaben, Sitzungs-/Gerätekontrolle, sofortiger Widerruf, Audit, Sicherheitsmeldungen, scoped Dienstkonten. MFA risikobasiert; Sitzungsgrenzen je Träger mit Mindestschutz.

## 10. Administration (Interview 46)
Alle 20 Funktionen: Onboarding, Betriebsprofile, Module und Unterfunktionen, Abhängigkeiten, Branding, Domains, Rollen, Dashboards, Navigation, Formulare/Workflows, Vorlagen, Benachrichtigungen, Integrationen, Speicher, Datenschutz, Backups, Monitoring, Updates, Konfigurations-Im-/Export, Audit/Freigaben. Moduldeaktivierung versteckt Funktionen und erhält Daten geschützt; Löschen gesondert. Träger-Mindeststandards nicht übersteuerbar. Delegierte Administration je Bereich.

## 11. Qualitäts- und Abnahmekriterien
WCAG 2.2 AA, mobile PWA, de/en, responsives UI, verständliche Fehler, vollständige Auditierbarkeit, keine Mandantendaten-Lecks, transaktionssichere Basar- und QR-Buchungen, kryptografisch und organisatorisch überprüftes Wahlgeheimnis. Erste öffentliche Stable 1.0 enthält alle vereinbarten Funktionen; Implementierung in internen Phasen. Spätere Stable-Versionen erweitern den Umfang. Fachliche Definition of Done: Tests, Dokumentation, Berechtigungsmatrix, Datenmigration, Monitoring, Accessibility, Datenschutzprüfung, Review und Release-Gates.

## 12. Historische Basisfunktionen aus Interview 1–36
Die folgenden ursprünglichen fachlichen Festlegungen bleiben erhalten, sofern nicht durch Fragen 37–51 ausdrücklich ersetzt. Einzelne frühere Ausschlüsse (z. B. Abwesenheit, Abholberechtigung) sind durch spätere Anforderungen aufgehoben.

## 4. Elternportal und Kommunikation
- Personalisierter Newsfeed, Pinnwand, zielgruppenbezogene Nachrichten, Elternbeiratschat, Kontaktpostfach, Kommentare/Diskussionen, Umfragen, Abstimmungen, anonyme Rückmeldungen und Lesebestätigungen.
- Push und E-Mail; konfigurierbare Pflichtmeldungen, individuelle Präferenzen und Ruhezeiten.
- Offenheit, Moderation und Berechtigungen pro Kommunikationsbereich und Einrichtung.
- Kontakt-/Ansprechpartnerverzeichnis, vertrauliche Beschwerden/Anliegen mit Bearbeitungsstatus, Notfall-/Krisenmeldungen mit Empfangsbestätigung und alternativen Kommunikationswegen.

## 5. Veranstaltungen, Planung und Helfende
- Veranstaltungserstellung, wiederverwendbare Vorlagen, Aufgaben, Checklisten, Dateien, Kalender, Terminabstimmungen, Teilnahme und Helferschichten.
- Mitbringlisten, Verpflegungsplanung, Bestands-/Materialbezug, Budget, Kommunikation und Erinnerungen.
- Einrichtungs- und veranstaltungsbezogene Rollen; Verantwortlichkeiten, Fristen und Status.

## 6. Basarverwaltung
- Basare, Verkäufervoranmeldung und -registrierung; interne Elternkonten, externe Gäste ohne Konto und externe registrierte Nutzer; E-Mail-Bestätigungen.
- Stand-/Tischbuchungen, digitaler Raumplan, manuelle Zuteilung oder First-Come-Automatik – je Basar wählbar.
- Konfigurierbare Tischgrößen, Ausstattungen, Preise, Reservierungsfristen, Verbindlichkeit, Stornoregeln, Wartelisten und Erstattungsregeln.
- Zahlung per Bargeld oder manuell verifiziertem PayPal; Ablauf einer Reservierung gibt Kapazität wieder frei. Keine automatische Rückzahlung.
- Transaktionssichere Buchung, um Doppelreservierungen bei gleichzeitigen Anfragen zu verhindern.

## 7. Finanzen, Kassen und Zahlungen
- Kassenbuch, mehrere Kassen/Verkaufsstände, Veranstaltungskalkulation, Belege, Einnahmen/Ausgaben, Spenden ohne automatisch ausgestellte steuerliche Spendenbescheinigung, Budgets, Genehmigungen, Auswertungen und Kassenübergabe.
- PayPal-Zahlungen ausschließlich manuell anhand eindeutiger Referenzen abgleichen; Bargeld, Teil- und Mischzahlungen, manuelle Rückerstattungen, Stornos und Gutschriften.
- Akzeptierte Zahlungsarten bis zur Artikel-/Buchungsebene konfigurierbar; Rückerstattungsfreigaben nach Rolle und Betrag je Einrichtung.
- Vier-Augen-Prinzip, Kassensturz, Jahresabschluss, append-only Buchungshistorie mit nachvollziehbaren Korrekturen.
- PayPal-Nutzungsbedingungen für Verkäufe separat prüfen; kein automatisches Anpreisen von „Freunde und Familie“ für kommerzielle Verkäufe.

## 8. Essen, Getränke und QR-Wertmarken
- Vorverkauf von Artikeln und Bündeln; einmalige, kryptografisch schwer erratbare QR-Codes; ausdruckbare Wertmarken und Tausch gegen physische Marken.
- PWA-Scanner, standbezogene Einlösung, Teilverbräuche bei Bündeln, Storno/Sperre, Audit, Bestandsabgleich und manuelle Erstattungen.
- Online-Einlösung atomar; Offline-Modus nur nach ereignisspezifischer Aktivierung mit ausdrücklichem Hinweis auf Doppelverwendungsrisiko und späterem Abgleich.
- Rechte je Veranstaltung und Verkaufsstand konfigurierbar.

## 9. Elternbeiratswahlen
- Kandidatenaufnahme/-freigabe ausschließlich durch berechtigte Wahladministration; Kandidatenfoto und Kurzprofil, editierbar durch Wahlgremium oder berechtigte Eltern.
- Wahlberechtigung und Stimmgewicht je Wahl konfigurierbar und nach lokal geltender Wahlordnung zu prüfen; ein Stimmrecht pro Kind ist nicht pauschal vorgeschrieben.
- Geheime Wahl; online, Papier oder hybrid; doppelte Stimmabgabe verhindern, ohne Identität mit Stimmzettel zu verknüpfen.
- Wahlkommission, Vier-Augen-Prüfung, Wahlprotokoll, Ergebnisprüfung, Korrektur/Nachzählung und Veröffentlichung erst nach Freigabe.
- Wahlgeheimnis auch gegenüber Systemadministratoren; anonymisierte Wahlbeteiligung und Berichte.

## 10. Sitzungen und Beschlüsse
- Sitzungstermine, Einladungen, Tagesordnung, Anwesenheit, Protokoll-Rich-Text, Beschlüsse, interne Abstimmungen, Aufgaben, Freigabe, Archiv, Vorlagen und PDF/Druck.
- Zugriff je einzelnem Protokoll steuerbar; Freigabeabläufe je Einrichtung konfigurierbar; Verknüpfung mit Wissensdatenbank und Kalender.

## 11. Wissensdatenbank
- OneNote-ähnliche Notizbücher je Einrichtung, Abschnitte, Seiten, Unterseiten, Rich Text, Tabellen, frei platzierbare und skalierbare Bilder, Anhänge, Drag-and-drop, Inhaltsverzeichnisse und interne Verlinkungen.
- Kollaborative oder exklusive Bearbeitung je Notizbuch, Autosave, Versionierung/Wiederherstellung, Volltextsuche, Vorlagen, Exporte, Rechte je Notizbuch.
- Verlinkung zu Sitzungen, Veranstaltungen und Aufgaben; Jahresübergabe des Elternbeirats.

## 12. Inventar und Materialien
- Lager- und Materialstammdaten, Mengen, Standorte, Ausgabe/Rückgabe, Verknüpfung zu Veranstaltungen und Verkaufsständen, Bestandskorrekturen und Berichte.

## 13. Zusätzliche Elternservices
- Digitale Formulare, Rückmeldungen und Bestätigungen; Dokumentenfreigaben und Kenntnisnahmen; Sprechstunden-/Terminbuchung.
- Foto-/Medieneinwilligungen und geschützte Medien; zweckbezogen, widerrufbar, mit nachgelagerter Berechtigungsprüfung.
- Eltern-Tauschbörse mit Angeboten zum Tauschen/Verschenken und Moderationsrechten.
- Kontaktverzeichnis, vertrauliche Anliegen, Jahreswechsel-Assistent und Notfallkommunikation.
- Diese neun Zusatzfunktionen sind Teil der ersten Stable-Version, jedoch je Einrichtung deaktivierbar.
- Nicht vereinbart: Krank-/Abwesenheitsmeldung, Abholberechtigungen, Fundbüro.

## 14. Kalender, Benachrichtigungen und Automationen
- Zentrale, persönliche und gruppenbezogene Kalender, Kategorien, Serientermine, Terminfindung, ICS-Export und widerrufbare private Abonnement-URLs; kein bidirektionaler Kalendersync.
- Rollen-, Gruppen- und Ereignistyp-Sichtbarkeit; Terminüberschneidungsprüfung.
- Wiederanlaufsichere, idempotente Jobs für Reservierungen/Wartelisten, Zahlungsrückstände, Schichten, Fristen, Sitzungen, Wertmarken, Wahlen, Jahreswechsel und Digest-Nachrichten.
- Pflichtnachrichten je Einrichtung; übrige Präferenzen je Nutzer; Ruhezeiten mit Einrichtungsstandard und persönlichem Override.

## 15. Berichte und Auswertungen
- Dashboards für Einrichtungen, Events, Helfereinsätze, Basare, Finanzen, QR-Verkäufe, Aufgaben, Umfragen und Wahlbeteiligung.
- PDF, Excel und CSV; flexible Filter; Sichtbarkeit je Einrichtung und Bericht; Versandpläne je Bericht.
- Wahlgeheimnis und Datenminimierung auch in Exporten und aggregierten Auswertungen wahren.

## 16. Administration, Datenschutz und Lebenszyklus
- Zentrale und lokale Administration, Einrichtungsbranding, Domain-/Subdomainoptionen, individuelle SMTP-Absender, Modulschalter, Wartungsmodus global/je Einrichtung, Diagnose, Jobs, Logs, Speicher und Konfigurationsexport.
- Datenschutzinformationen und Einwilligungsversionen; Auskunft, Berichtigung, Export, Aufbewahrung und Löschung nach Datenkategorie/Einrichtung; gesetzliche Mindestfristen beachten.
- Beim Familienaustritt besondere interne Rollen sofort entziehen; endgültige Löschung automatisch oder mit Freigabe je Kategorie; Backup-Restores dürfen bereits umgesetzte Löschpflichten nicht rückgängig machen.
- Vollständiger Mandantenexport/Transfer, kontrollierter Import und Testlauf mit Dublettenprüfung und Feldzuordnung.


## Dokumentationssprache (Ergänzung v5.9)
Für dieses Dokument gilt die deutsche Projektrichtlinie `13_Dokumentationsrichtlinie_DE_ASD_STE100.md` mit dem verbindlichen Fachwortverzeichnis `14_Fachwortverzeichnis_DE.md`. Die Richtlinie orientiert sich an ASD-STE100, stellt aber **keine offizielle deutsche ASD-STE100-Konformität** dar.
