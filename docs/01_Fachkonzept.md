# Fachkonzept

**KitaConnect Spezifikation v5.9 · 17.10.2026 · Konsolidierter Planungsstand**

> Status: Anforderungsspezifikation, keine implementierte oder rechtsverbindlich geprüfte Software. Die neuesten ausdrücklichen Interviewentscheidungen gelten.

## 1. Produktvision und Betriebsmodelle

KitaConnect ist eine selbst betriebene Progressive Web App (PWA).
Sie richtet sich an Elternbeiräte, Kindertageseinrichtungen, Eltern, Mitarbeitende und Träger.
Die Anwendung unterstützt Deutsch und Englisch. Die Oberfläche passt sich an die Bildschirmgröße an und ist barrierearm.

Es gibt drei Betriebsmodelle:

- A: nur Elternbeirat, ohne Kinderakte.
- B: nur Einrichtung.
- C: Einrichtung und Elternbeirat kombiniert.

Ein Wechsel benötigt eine Berechtigung. Er darf keine unbemerkte Rechteausweitung oder Datenlöschung verursachen.
Mehrere unabhängige Einrichtungen und mehrere Träger können dieselbe Installation verwenden.
Funktionsumfang und Unterfunktionen sind je Einrichtung konfigurierbar.
Verbindliche Sicherheitsgrenzen sind nicht abschaltbar.

## 2. Nutzer, Organisation und Zuständigkeiten

Das Rollenmodell umfasst folgende Zuständigkeiten:

- Systemadministrator und Trägeradministrator.
- Einrichtungsleitung und Einrichtungsadministrator.
- Personalverwaltung und pädagogische Mitarbeitende.
- Eltern und sorgeberechtigte Personen.
- Vorsitz und Mitglieder des Elternbeirats.
- Wahlleitung, Kassenführung und externe Helfende.
- Küche, Caterer und datenschutzverantwortliche Rollen.

Ein Benutzer kann mehrere Mitgliedschaften und Rollen besitzen.
Kontomodelle sind je Einrichtung konfigurierbar. Fachliche Datenverantwortlichkeiten bleiben getrennt.
Der Elternbeirat erhält keinen impliziten Zugriff auf Kinderakten, Gesundheitsdaten, Personalakten oder Entwicklungsdaten.
Trägerweite Personalverwaltung, Springerpool, gemeinsame Vorlagen und Berichte benötigen begrenzte, ausdrücklich vergebene Zugriffsrechte.

## 3. Elternbeirat und gemeinsame Prozesse

Der Elternbeiratsbereich umfasst folgende Funktionen:

- Elternkommunikation, Nachrichtenübersicht, Termine und Veranstaltungen.
- Helferlisten, Mitbringlisten, Basare und Standbuchungen.
- Manuelle PayPal-Abgleiche, Kassenbuch, Belege und QR-Wertmarken.
- Geheime Online-, Papier- und Hybridwahlen sowie Sitzungen und Beschlüsse.
- Wissensdatenbank, Inventar, Umfragen, Beschwerden und Tauschbörse.
- Formulare, Elternsprechstunden, Einwilligungen, Krisenkommunikation und Berichte.

Veranstaltungen, Dokumente, Projekte, Umfragen, Kalender, Kontaktverzeichnis und Benachrichtigungen können gemeinsam freigegeben werden.
Freigaben bleiben zweckgebunden und nachvollziehbar.
Die Finanzen der Einrichtung und des Elternbeirats bleiben getrennt.

## 4. Kinder, Familien und Aufnahme (Interview 42,45)

Der Umfang enthält alle 18 Kinderverwaltungsfunktionen und 20 Lebenszyklusprozesse aus den Interviews.
Die Kinderverwaltung umfasst:

- Kinderstammdaten und sorgeberechtigte Personen mit individuellen Rechten.
- Gruppen, Altersbereiche, Betreuungsverträge und Buchungszeiten.
- Digitale Anwesenheit, Krankmeldungen, Abwesenheitsmeldungen und Abholberechtigungen.
- Notfallkontakte, notwendige Gesundheitsinformationen, Medikamentengabe und Einwilligung.
- Geschützte Dokumente, Geschwister und Zuordnungen zu mehreren Einrichtungen.
- Import, Export, Aufnahmeformulare, Änderungsformulare und Änderungshistorie.

Die Lebenszyklusprozesse umfassen:

- Digitale Vormerkung, Warteliste, Platzvergabe und Aufnahme.
- Eingewöhnungsmodelle, Eingewöhnungsdokumentation und automatische Einladung der Eltern.
- Bringen, Abholen, Tagesdokumentation, Pflege, Wickeln und Schlaf.
- Übergaben, Unfälle und Elterngespräche.
- Gruppenwechsel, Schuljahreswechsel, Vorschule und geführten Austritt.
- Konfigurierbare Arbeitsabläufe.

Der Umfang ist je Betriebsmodell und Einrichtung konfigurierbar.
Die Tagesdokumentation ist je Altersgruppe konfigurierbar. Voreinstellungen sind datensparsam.

## 5. Personal und Dienstorganisation (Interview 40)

Der Umfang enthält alle 18 Funktionen aus Interview 40:

- Personalstamm, Beschäftigungsmodelle, Dienstpläne, Schichten und Vorlagen.
- Wünsche, Verfügbarkeiten, Urlaub, Krankheit, Tausch und Vertretung.
- Arbeitszeiterfassung manuell, per Stempeluhr oder in beiden Formen.
- Zeitkonten, Mehrarbeit und genehmigte Korrekturen.
- Besetzungsplanung und Warnungen bei Unterbesetzung.
- Qualifikationen, Fortbildungen, Sitzungen und Gespräche.
- Übergabebuch, Mitarbeiterbereich, Auswertungen, Historie und Freigaben.

Automatische Dienstplanvorschläge sind Empfehlungen. Die zuständige Person erteilt die Freigabe.
Regionale und trägerspezifische Betreuungsschlüssel werden als versionierte Regelwerke geführt.
Ihre Rechtsgültigkeit benötigt eine gesonderte Prüfung.
Trägerübergreifende Verwaltung verwendet nur die erforderlichen Rechte.

## 6. Portfolios, Pädagogik und Medien (Interview 37a,39)

Der Umfang enthält alle 17 Portfolio- und Entwicklungsfunktionen aus den Interviews:

- Kindportfolio, frei gestaltbare Seiten per Drag-and-drop und Vorlagen.
- Gruppenrückblicke, Beobachtungen und strukturierte Entwicklungsbögen.
- Ziele, Meilensteine, Lerngeschichten sowie Foto, Video und Audio.
- Freigabe, Elternansicht, Elternrückmeldung und Kinderbeiträge.
- PDF, Druck, Versionshistorie und Gruppenwechsel.

Portfolio und vertrauliche Entwicklungsakte bleiben logisch strikt getrennt.
Freigaben berücksichtigen Einrichtung, Gruppe, Inhalt und Rolle.
Die Gesichtserkennung erfolgt lokal und ohne Identitätsabgleich.
Erkannte Gesichter werden zunächst maskiert. Die Maskierung ist manuell korrigierbar.
Vor Veröffentlichung ist eine Prüfung verpflichtend.
EXIF-Metadaten und GPS-Standortdaten werden entfernt.
Die Einwilligung wird bei jeder Wiederverwendung geprüft.
Eine automatisierte Bewertung kindlicher Entwicklung durch künstliche Intelligenz (KI) ist ausgeschlossen.

## 7. Essen und Verpflegung (Interview 38)

Das Essensmodul umfasst:

- Wochenpläne, Monatspläne, Vorlagen und Bilder.
- Allergene, Zusatzstoffe, Nährwerte und Zutaten.
- Bestellungen, Abmeldungen, Fristen und automatische Essenszahlen.
- Besondere Kostformen und Ansichten für Küche und Caterer.
- Preisanzeige, Kostenberichte und Export als PDF, Excel oder CSV.
- Erinnerungen, Rückmeldungen und mehrere Anbieter.

Eigene Küche, Caterer und Mischformen werden unterstützt.
Die Bestellung ist deaktivierbar. Sie ist einzeln oder als Abonnement mit Abmeldungen möglich.
**Das Essensmodul enthält keine Rechnungsstellung, Forderungsverwaltung oder Zahlungsabwicklung.**
Preisangaben dienen der Information und Auswertung. Gesundheitsbezogene Kostangaben sind besonders geschützt.

## 8. Träger und gemeinsame Prozesse (Interview 41,43)

Der Umfang enthält 16 Trägerfunktionen:

- Trägerhierarchie und zentrale Administration.
- Personal, Vertretung, Besetzung und Personalwechsel.
- Vorlagen, Vorgaben, Gestaltung, Berichte und Termine.
- Kommunikation, Dokumente, getrennte Finanzen und Zugriffsprotokollierung.
- Zentrale Importe und Exporte.

Mehrere Träger und trägerlose Einrichtungen werden unterstützt.
Trägerrechte sind konfigurierbar. Verbindliche Mindeststandards bleiben bestehen.
Die Übertragung von Kinder- und Familiendaten benötigt Freigaben und rechtliche Prüfung.
Der Umfang enthält außerdem 17 gemeinsame Funktionen für Einrichtung und Elternbeirat.
Diese Funktionen verwenden kontrollierte Freigaben.
Die Elternansicht ist gemeinsam oder getrennt konfigurierbar.

## 9. Datenschutz und Sicherheit (Interview 44,48)

Der Umfang enthält alle 20 Datenschutzfunktionen:

- Fristen je Datenkategorie sowie automatisierte Prüf- und Löschaufträge.
- Auskunft, Export, Berichtigung, Löschung, Einwilligungen und Einzelbildfreigabe.
- Besonderer Schutz für Gesundheit, Sorgerecht, Entwicklung und Personal.
- Vollständige Protokollierung von Lese- und Schreibzugriffen.
- Begründeter, zeitbegrenzter Notfallzugriff mit Zweitfreigabe, soweit möglich.
- Zusätzliche Verschlüsselung und Löschabgleich mit Datensicherungen.
- Vorfallmanagement, Datenschutzberichte, Hinweise und Vereinbarungen.
- Vollständiger Mandantenexport, regelmäßige erneute Rechteprüfung und datensparsame Voreinstellungen.

Fristen benötigen eine rechtliche Prüfung. Es werden keine universellen Fristen erfunden.
Der Umfang enthält außerdem alle 20 Identitätsfunktionen:

- Passwort, Passkeys, zeitbasierte Einmalpasswörter (TOTP) und Kontowiederherstellung.
- OpenID Connect (OIDC), OAuth 2.0 und Security Assertion Markup Language (SAML).
- Einladungen, Selbstbedienung, Mehrfachrollen und Kontextwechsel.
- Rollenbasierte Berechtigungen (RBAC) sowie Objekt- und Feldregeln.
- Standardmäßige Zugriffsverweigerung, temporäre Rechte und Vier-Augen-Freigaben.
- Sitzungs- und Gerätekontrolle, sofortiger Widerruf und Zugriffsprotokollierung.
- Sicherheitsmeldungen und Dienstkonten mit begrenztem Berechtigungsumfang.

Mehrfaktor-Authentisierung (MFA) richtet sich nach dem Risiko.
Sitzungsgrenzen sind je Träger konfigurierbar. Der Mindestschutz bleibt verbindlich.

## 10. Administration (Interview 46)

Der Umfang enthält alle 20 Administrationsfunktionen:

- Ersteinrichtung, Betriebsprofile, Module, Unterfunktionen und Abhängigkeiten.
- Gestaltung, Domains, Rollen, Übersichten und Navigation.
- Formulare, Arbeitsabläufe, Vorlagen und Benachrichtigungen.
- Integrationen, Speicher, Datenschutz und Datensicherungen.
- Betriebsüberwachung, Updates, Konfigurationsimport und Konfigurationsexport.
- Zugriffsprotokollierung und Freigaben.

Eine Moduldeaktivierung blendet Funktionen aus. Die Daten bleiben geschützt erhalten.
Löschen ist eine gesonderte Aktion. Träger-Mindeststandards sind nicht übersteuerbar.
Administration kann je Bereich delegiert werden.

## 11. Qualitäts- und Abnahmekriterien

Die Abnahme verlangt die Web Content Accessibility Guidelines (WCAG) 2.2 auf Stufe AA.
Sie verlangt außerdem eine mobile PWA, Deutsch und Englisch sowie eine anpassungsfähige Benutzeroberfläche.
Fehlermeldungen sind verständlich. Datenzugriffe sind vollständig prüfbar.
Es gibt keine unberechtigte Offenlegung von Mandantendaten.
Basar- und QR-Buchungen sind transaktionssicher.
Das Wahlgeheimnis ist kryptografisch und organisatorisch geprüft.

Die erste öffentliche stabile Version 1.0 enthält alle vereinbarten Funktionen.
Die Umsetzung erfolgt in internen Phasen. Spätere stabile Versionen erweitern den Umfang.
Die fachliche Fertigstellung verlangt Tests, Dokumentation, Berechtigungsmatrix und Datenmigration.
Sie verlangt außerdem Betriebsüberwachung, Barrierefreiheit, Datenschutzprüfung, Review und verpflichtende Veröffentlichungsprüfungen.

## 12. Historische Basisfunktionen aus Interview 1–36

Die folgenden ursprünglichen Festlegungen bleiben erhalten, soweit Fragen 37–51 sie nicht ausdrücklich ersetzen.
Frühere Ausschlüsse von Abwesenheitsmeldungen und Abholberechtigungen sind durch Abschnitt 4 aufgehoben.
Die historische Ausschlussliste in Abschnitt 12.10 dokumentiert den damaligen Stand und hebt diese späteren Anforderungen nicht auf.

### 12.1 Elternportal und Kommunikation

- Personalisierte Nachrichtenübersicht, Pinnwand, zielgruppenbezogene Nachrichten, Elternbeiratschat, Kontaktpostfach, Kommentare/Diskussionen, Umfragen, Abstimmungen, anonyme Rückmeldungen und Lesebestätigungen.
- Push und E-Mail; konfigurierbare Pflichtmeldungen, individuelle Präferenzen und Ruhezeiten.
- Offenheit, Moderation und Berechtigungen pro Kommunikationsbereich und Einrichtung.
- Kontakt-/Ansprechpartnerverzeichnis, vertrauliche Beschwerden/Anliegen mit Bearbeitungsstatus, Notfall-/Krisenmeldungen mit Empfangsbestätigung und alternativen Kommunikationswegen.

### 12.2 Veranstaltungen, Planung und Helfende

- Veranstaltungserstellung, wiederverwendbare Vorlagen, Aufgaben, Checklisten, Dateien, Kalender, Terminabstimmungen, Teilnahme und Helferschichten.
- Mitbringlisten, Verpflegungsplanung, Bestands-/Materialbezug, Budget, Kommunikation und Erinnerungen.
- Einrichtungs- und veranstaltungsbezogene Rollen; Verantwortlichkeiten, Fristen und Status.

### 12.3 Basarverwaltung

- Basare, Verkäufervoranmeldung und -registrierung; interne Elternkonten, externe Gäste ohne Konto und externe registrierte Nutzer; E-Mail-Bestätigungen.
- Stand-/Tischbuchungen, digitaler Raumplan, manuelle Zuteilung oder First-Come-Automatik – je Basar wählbar.
- Konfigurierbare Tischgrößen, Ausstattungen, Preise, Reservierungsfristen, Verbindlichkeit, Stornoregeln, Wartelisten und Erstattungsregeln.
- Zahlung per Bargeld oder manuell verifiziertem PayPal; Ablauf einer Reservierung gibt Kapazität wieder frei. Keine automatische Rückzahlung.
- Transaktionssichere Buchung, um Doppelreservierungen bei gleichzeitigen Anfragen zu verhindern.

### 12.4 Finanzen, Kassen und Zahlungen

- Kassenbuch, mehrere Kassen/Verkaufsstände, Veranstaltungskalkulation, Belege, Einnahmen/Ausgaben, Spenden ohne automatisch ausgestellte steuerliche Spendenbescheinigung, Budgets, Genehmigungen, Auswertungen und Kassenübergabe.
- PayPal-Zahlungen ausschließlich manuell anhand eindeutiger Referenzen abgleichen; Bargeld, Teil- und Mischzahlungen, manuelle Rückerstattungen, Stornos und Gutschriften.
- Akzeptierte Zahlungsarten bis zur Artikel-/Buchungsebene konfigurierbar; Rückerstattungsfreigaben nach Rolle und Betrag je Einrichtung.
- Vier-Augen-Prinzip, Kassensturz, Jahresabschluss, nur ergänzbare Buchungshistorie mit nachvollziehbaren Korrekturen.
- PayPal-Nutzungsbedingungen für Verkäufe separat prüfen. Kein automatisches Anpreisen von „Freunde und Familie“ für kommerzielle Verkäufe.

### 12.5 Essen, Getränke und QR-Wertmarken

- Vorverkauf von Artikeln und Bündeln; einmalige, kryptografisch schwer erratbare QR-Codes; ausdruckbare Wertmarken und Tausch gegen physische Marken.
- PWA-Scanner, standbezogene Einlösung, Teilverbräuche bei Bündeln, Storno/Sperre, Zugriffsprotokollierung, Bestandsabgleich und manuelle Erstattungen.
- Online-Einlösung atomar; Offline-Modus nur nach ereignisspezifischer Aktivierung mit ausdrücklichem Hinweis auf Doppelverwendungsrisiko und späterem Abgleich.
- Rechte je Veranstaltung und Verkaufsstand konfigurierbar.

### 12.6 Elternbeiratswahlen

- Kandidatenaufnahme/-freigabe ausschließlich durch berechtigte Wahladministration; Kandidatenfoto und Kurzprofil, editierbar durch Wahlgremium oder berechtigte Eltern.
- Wahlberechtigung und Stimmgewicht je Wahl konfigurierbar und nach lokal geltender Wahlordnung zu prüfen; ein Stimmrecht pro Kind ist nicht pauschal vorgeschrieben.
- Geheime Wahl; online, Papier oder hybrid; doppelte Stimmabgabe verhindern, ohne Identität mit Stimmzettel zu verknüpfen.
- Wahlkommission, Vier-Augen-Prüfung, Wahlprotokoll, Ergebnisprüfung, Korrektur/Nachzählung und Veröffentlichung erst nach Freigabe.
- Wahlgeheimnis auch gegenüber Systemadministratoren; anonymisierte Wahlbeteiligung und Berichte.

### 12.7 Sitzungen und Beschlüsse

- Sitzungstermine, Einladungen, Tagesordnung, Anwesenheit, Protokoll-Rich-Text, Beschlüsse, interne Abstimmungen, Aufgaben, Freigabe, Archiv, Vorlagen und PDF/Druck.
- Zugriff je einzelnem Protokoll steuerbar; Freigabeabläufe je Einrichtung konfigurierbar. Verknüpfung mit Wissensdatenbank und Kalender.

### 12.8 Wissensdatenbank

- OneNote-ähnliche Notizbücher je Einrichtung, Abschnitte, Seiten, Unterseiten, Rich Text, Tabellen, frei platzierbare und skalierbare Bilder, Anhänge, Drag-and-drop, Inhaltsverzeichnisse und interne Verlinkungen.
- Kollaborative oder exklusive Bearbeitung je Notizbuch, Autosave, Versionierung/Wiederherstellung, Volltextsuche, Vorlagen, Exporte, Rechte je Notizbuch.
- Verlinkung zu Sitzungen, Veranstaltungen und Aufgaben; Jahresübergabe des Elternbeirats.

### 12.9 Inventar und Materialien

- Lager- und Materialstammdaten, Mengen, Standorte, Ausgabe/Rückgabe, Verknüpfung zu Veranstaltungen und Verkaufsständen, Bestandskorrekturen und Berichte.

### 12.10 Zusätzliche Elternservices

- Digitale Formulare, Rückmeldungen und Bestätigungen; Dokumentenfreigaben und Kenntnisnahmen; Sprechstunden-/Terminbuchung.
- Foto-/Medieneinwilligungen und geschützte Medien; zweckbezogen, widerrufbar, mit nachgelagerter Berechtigungsprüfung.
- Eltern-Tauschbörse mit Angeboten zum Tauschen/Verschenken und Moderationsrechten.
- Kontaktverzeichnis, vertrauliche Anliegen, Jahreswechsel-Assistent und Notfallkommunikation.
- Diese neun Zusatzfunktionen sind Teil der ersten stabilen Version, jedoch je Einrichtung deaktivierbar.
- Historischer Ausschluss: Krankmeldungen, Abwesenheitsmeldungen, Abholberechtigungen und Fundbüro.
  Abschnitt 4 ersetzt den Ausschluss von Krankmeldungen, Abwesenheitsmeldungen und Abholberechtigungen.

### 12.11 Kalender, Benachrichtigungen und Automationen

- Zentrale, persönliche und gruppenbezogene Kalender, Kategorien, Serientermine, Terminfindung, ICS-Export und widerrufbare private Abonnement-URLs; keine Kalendersynchronisation in beide Richtungen.
- Rollen-, Gruppen- und Ereignistyp-Sichtbarkeit; Terminüberschneidungsprüfung.
- Wiederanlaufsichere, idempotente Aufträge für Reservierungen/Wartelisten, Zahlungsrückstände, Schichten, Fristen, Sitzungen, Wertmarken, Wahlen, Jahreswechsel und Digest-Nachrichten.
- Pflichtnachrichten je Einrichtung; übrige Präferenzen je Nutzer; Ruhezeiten mit Einrichtungsstandard und persönlicher Übersteuerung.

### 12.12 Berichte und Auswertungen

- Übersichten für Einrichtungen, Veranstaltungen, Helfereinsätze, Basare, Finanzen, QR-Verkäufe, Aufgaben, Umfragen und Wahlbeteiligung.
- PDF, Excel und CSV; flexible Filter; Sichtbarkeit je Einrichtung und Bericht; Versandpläne je Bericht.
- Wahlgeheimnis und Datenminimierung auch in Exporten und aggregierten Auswertungen wahren.

### 12.13 Administration, Datenschutz und Lebenszyklus

- Zentrale und lokale Administration, Einrichtungsbranding, Domain-/Subdomainoptionen, individuelle SMTP-Absender, Modulschalter, Wartungsmodus global/je Einrichtung, Diagnose, Aufträge, Protokolle, Speicher und Konfigurationsexport.
- Datenschutzinformationen und Einwilligungsversionen; Auskunft, Berichtigung, Export, Aufbewahrung und Löschung nach Datenkategorie/Einrichtung. Gesetzliche Mindestfristen beachten.
- Beim Familienaustritt besondere interne Rollen sofort entziehen. Endgültige Löschung automatisch oder mit Freigabe je Kategorie; Wiederherstellungen dürfen bereits umgesetzte Löschpflichten nicht rückgängig machen.
- Vollständiger Mandantenexport/Transfer, kontrollierter Import und Testlauf mit Dublettenprüfung und Feldzuordnung.


## Dokumentationssprache (Ergänzung v5.9)

Es gelten die [deutsche Dokumentationsrichtlinie](13_Dokumentationsrichtlinie_DE_ASD_STE100.md)
und das [Fachwortverzeichnis](14_Fachwortverzeichnis_DE.md).
Die Richtlinie adaptiert Sprachprinzipien von ASD-STE100.
Sie ist keine offizielle deutsche Übersetzung und belegt keine formelle ASD-STE100-Konformität.
