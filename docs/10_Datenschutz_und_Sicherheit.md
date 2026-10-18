# Datenschutz- und Sicherheitskonzept

**KitaConnect Spezifikation v5.9 · 17.10.2026 · Konsolidierter Planungsstand**

> Status: Anforderungsspezifikation, keine implementierte oder rechtsverbindlich geprüfte Software. Die neuesten ausdrücklichen Interviewentscheidungen gelten.

## Verantwortlichkeiten
Datenschutzverantwortlichkeiten sind je Träger und Einrichtung konfigurierbar.
Die Software überträgt diese Verantwortung nicht automatisch auf den Softwarebetreiber.
Vor produktivem Einsatz prüfen Fachstellen die Rechtsgrundlagen der Datenschutz-Grundverordnung (DSGVO).
Sie prüfen außerdem Aufbewahrungsfristen, Verträge und landesrechtliche Vorschriften für Kindertageseinrichtungen.
Eine erforderliche Datenschutz-Folgenabschätzung (DSFA) ist Teil dieser Prüfung.

## Schutzklassen
Es gibt vier Schutzklassen: öffentlich, intern, vertraulich und besonders sensibel.
Gesundheitsdaten, Entwicklungsakten, Sorgerechtsbeschränkungen, Personalakten, Beschwerden und Wahlgeheimnis sind besonders sensibel.
Es gelten Datenminimierung, Zweckbindung und Zugriff nur bei erforderlichem Kenntnisbedarf.
Übertragung und Speicherung sind verschlüsselt. Zusätzliche Schutzmaßnahmen gelten je Feld und Objekt.

## Rechte und Zugriffsprotokollierung

Zugriff wird standardmäßig verweigert.
Rollenbasierte Berechtigungen (RBAC) werden durch attributbasierte (ABAC), objektbezogene und feldbezogene Regeln ergänzt.
Berechtigungsumfänge für Träger und Einrichtungen bleiben getrennt.
Rechte sind zeitlich befristbar, sofort widerrufbar und werden regelmäßig erneut geprüft.

Alle personenbezogenen Lese- und Schreibzugriffe werden protokolliert.
Das Zugriffsprotokoll erfasst Akteur, Zeit, Zweck, Berechtigungsumfang, Objektkategorie, Aktion und Entscheidung.
Es erfasst außerdem eine technische Korrelationskennung. Vollständige Gesundheitsinhalte gehören nicht in das Zugriffsprotokoll.
Die Speicherung erlaubt nur Ergänzungen und macht Manipulationen erkennbar.
Zugriffsprotokolle haben getrennte Aufbewahrungsregeln. Folgen für Last und Speicher werden getestet.

## Einwilligungen und Medien
Einwilligungen sind versioniert und gelten je Zweck sowie je Einzelbild beziehungsweise Veröffentlichung.
Sie werden bei jeder Nutzung erneut geprüft. Ein Widerruf startet einen Ablauf zur Sperrung oder Entfernung.
Gesichtserkennung erfolgt ausschließlich lokal zur Maskierung und ohne biometrische Identifikation.
Eine manuelle Freigabe ist verpflichtend. Die Erkennbarkeit durch den Kontext wird berücksichtigt.
EXIF-Metadaten und GPS-Standortdaten werden entfernt.

## Betroffenenrechte und Löschung
Auskunft, Berichtigung, Export, Einschränkung und Löschprüfung benötigen eine Identitätskontrolle.
Sie berücksichtigen Fristen und Rechtsgrundlagen. Der Bearbeitungsstatus ist nachvollziehbar.
Lösch- und Aufbewahrungsregeln gelten je Datenkategorie, Träger und Einrichtung.
Gesetzliche Grenzen haben Vorrang. Eine Anfrage führt nicht pauschal zur Löschung.
Bei Wiederherstellung einer Datensicherung wird das Löschjournal geprüft.
Verschlüsselungsschlüssel werden sicher gewechselt.

## Notfallzugriff
Notfallzugriff ist nur für ausdrücklich definierte Notfälle zulässig.
Er benötigt eine Begründung und ist zeitlich begrenzt. Eine Zweitfreigabe erfolgt, soweit möglich.
Der Zugriff löst sofort einen Alarm aus und wird protokolliert. Eine Nachkontrolle ist verpflichtend.
Notfallzugriff darf das Wahlgeheimnis nicht umgehen.

## Bedrohungsmodell und Prüfungen
Die Prüfung berücksichtigt folgende Bedrohungen:

- Offenlegung fremder Mandantendaten durch `search_path` oder gemeinsam genutzte Datenbankverbindungen.
- Unberechtigte direkte Objektzugriffe (IDOR), gestohlene Sitzungen und kompromittierte Dienstkonten.
- Uploads mit Schadsoftware, fehlerhafte Medienfreigaben und Wiederherstellung bereits gelöschter Daten.
- Auswertung kleiner Gruppen, Angriffe durch interne berechtigte Personen und Zuordnung von Wählern zu Stimmen.
- Manipulierte QR-Codes.

Gegenmaßnahmen sind automatisierte Isolationstests, minimale Rechte und Sicherheitsreviews.
Weitere Gegenmaßnahmen sind Anfragemengenbegrenzung, Schutz gegen gefälschte browserbasierte Anfragen (CSRF)
und eine Sicherheitsrichtlinie für Browserinhalte (CSP).
Signierte Ereignisse, automatisierte Sicherheitsprüfungen, Wiederherstellungsproben und Vorfallmanagement ergänzen diese Maßnahmen.


## Dokumentationssprache (Ergänzung v5.9)
Für dieses Dokument gilt die deutsche Projektrichtlinie `13_Dokumentationsrichtlinie_DE_ASD_STE100.md` mit dem verbindlichen Fachwortverzeichnis `14_Fachwortverzeichnis_DE.md`. Die Richtlinie orientiert sich an ASD-STE100, stellt aber **keine offizielle deutsche ASD-STE100-Konformität** dar.
