# Datenschutz- und Sicherheitskonzept

**KitaConnect Spezifikation v5.9 · 17.10.2026 · Konsolidierter Planungsstand**

> Status: Anforderungsspezifikation, keine implementierte oder rechtsverbindlich geprüfte Software. Die neuesten ausdrücklichen Interviewentscheidungen gelten.

## Verantwortlichkeiten
Datenschutzverantwortlichkeiten je Träger/Einrichtung konfigurierbar, ohne Verantwortlichkeit automatisch auf Softwarebetreiber zu übertragen. DSGVO-Rechtsgrundlagen, Aufbewahrungsfristen, Verträge, ggf. DSFA und landesrechtliche Kita-Vorschriften müssen vor Produktiveinsatz mit Fachstellen geprüft werden.

## Schutzklassen
Öffentlich, intern, vertraulich, besonders sensibel. Besonders sensibel: Gesundheitsdaten, Entwicklungsakte, Sorgerechtsbeschränkungen, Personalakten, Beschwerden, Wahlgeheimnis. Datenminimierung, Zweckbindung, Need-to-Know, verschlüsselte Übertragung/Speicherung, zusätzliche feld-/objektbezogene Schutzmaßnahmen.

## Rechte und Audit
Default-Deny, RBAC plus ABAC/Objekt-/Feldregeln, separate Träger-/Einrichtungsscope, zeitlich befristete Rechte, sofortiger Widerruf, regelmäßige Rezertifizierung. Alle personenbezogenen Lese-/Schreibzugriffe protokollieren: Akteur, Zeit, Zweck/Scope, Objektkategorie, Aktion, Entscheidung, technische Korrelations-ID; keine vollständigen Gesundheitsinhalte in Auditdaten. Append-only/tamper-evident Speicherung und getrennte Audit-Retention; Last-/Speicherfolgen testen.

## Einwilligungen und Medien
Versionierte Einwilligungen pro Zweck und je Einzelbild/Veröffentlichung; erneute Prüfung bei jeder Nutzung; Widerruf führt zu Sperr-/Entfernungsworkflow. Gesichtserkennung ausschließlich lokal zur Maskierung ohne biometrische Identifikation; verpflichtende manuelle Freigabe, Kontextidentifikation beachten, EXIF/GPS entfernen.

## Betroffenenrechte und Löschung
Auskunft, Berichtigung, Export, Einschränkung und Löschprüfung mit Identitätskontrolle, Fristen, Rechtsgrundlagen und nachvollziehbarem Status. Lösch- und Aufbewahrungsregeln je Kategorie/Träger/Einrichtung, gesetzliche Grenzen haben Vorrang. Backup-Restore prüft Löschjournal; Verschlüsselungsschlüssel sicher rotieren. Keine pauschale Behauptung, jede Anfrage müsse stets zur Löschung führen.

## Notfallzugriff
Break-glass nur für ausdrücklich definierte Notfälle, Begründung, begrenzte Dauer, Zweitfreigabe soweit möglich, sofortiger Alarm, Audit und verpflichtende Nachkontrolle. Kein Notfall-Bypass des Wahlgeheimnisses.

## Bedrohungsmodell und Prüfungen
Cross-Tenant-Leakage durch search_path/Pooling, IDOR, gestohlene Sessions, kompromittierte Dienstkonten, Malware-Uploads, fehlerhafte Medienfreigaben, wiederhergestellte gelöschte Daten, Auswertung kleiner Gruppen, Insider-Angriffe, Wahlkorrelation, manipulierte QR-Codes. Gegenmaßnahmen: automatisierte Isolationstests, least privilege, Security Reviews, Rate-Limits, CSRF/CSP, signierte Events, Scans, Restore-Proben und Vorfallmanagement.


## Dokumentationssprache (Ergänzung v5.9)
Für dieses Dokument gilt die deutsche Projektrichtlinie `13_Dokumentationsrichtlinie_DE_ASD_STE100.md` mit dem verbindlichen Fachwortverzeichnis `14_Fachwortverzeichnis_DE.md`. Die Richtlinie orientiert sich an ASD-STE100, stellt aber **keine offizielle deutsche ASD-STE100-Konformität** dar.
