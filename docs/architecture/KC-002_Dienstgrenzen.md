# KC-002 – Dienstgrenzen und Datenhoheit

Stand: 18.10.2026. Grundlage: Spezifikation v5.9, ADR-001 und ADR-002.
Status: Architekturentscheidungen durch die Projektleitung bestätigt. Umsetzung ausstehend.
Issue: [KC-002](https://github.com/itsh-neumeier/kitaconnect/issues/2).

## Ziel und Abgrenzung

Dieses Dokument ordnet die konzeptionellen Entitäten aus `docs/03` den Diensten aus `docs/02` zu.
Es benennt erlaubte Kommunikationszwecke. Es definiert noch keine Endpunkte oder Datenbanktabellen.
Ein Eintrag erteilt weder einer Person noch einem Dienst eine Berechtigung.
Die fachlichen Regeln aus [Datenmodell](../03_Datenmodell_und_Berechtigungen.md)
und [Sicherheitskonzept](../10_Datenschutz_und_Sicherheit.md) bleiben verbindlich.

Die Entscheidungen [ADR-014](ADR-014_Datenhoheit_und_Migrationen.md) und
[ADR-015](ADR-015_Gateway_Grenze.md) bearbeiten O-001 und O-002.
Die Projektleitung hat am 18.10.2026 mit `1A, 2A, 3A, 4A` ausdrücklich entschieden.
[ADR-016](ADR-016_Personalbereich.md) regelt Personalbereiche.
[ADR-017](ADR-017_Wahltrennung.md) regelt getrennte Wahldatenbanken und die unabhängige Protokollprüfung.
Die Freigabe dieser Architektur erteilt keinen Auftrag zur Implementierung nach P0.

## Datenhoheit

Datenhoheit bezeichnet hier die alleinige fachliche Verantwortung für Änderungen
und die zugehörigen Dienstverträge. Sie ersetzt keine datenschutzrechtliche Verantwortlichkeit.
Die technischen Namen sind Entwürfe. Sie sind keine zugesagten Schnittstellen.

| Dienst | Zugeordnete Entitäten oder Verantwortung | Entschiedener Speicherbereich |
|---|---|---|
| `identity-access` | `identities`, `credentials`, `sessions`, `roles`, `permissions`, `policy_rules` | Eigener restriktiver Plattformbereich |
| `organization-config` | `organizations`, `carriers`, `facilities`, `memberships`, `module_flags`, `feature_flags`, `setting_versions`; technische Schemaregistrierung | Eigener restriktiver Plattformbereich |
| `children-admissions` | `children`, `guardians`, `guardian_child_links`, `custody_restrictions`, `groups`, `enrollments`, `care_contracts`, `health_notes`, `medication_authorizations`, `emergency_contacts` | PostgreSQL-Schema der Einrichtung; getrennte Feldrechte für Gesundheit und Sorgerecht |
| `attendance-care` | `attendance_events`, `absences`, `pickup_authorizations`, `daily_care_logs`, `incidents`, `handovers`, `medication_administrations` | PostgreSQL-Schema der Einrichtung |
| `staff-scheduling` | `employees`, `employment_contracts`, `assignments`, `shifts`, `time_entries`, `leave_requests`, `qualifications` | Eigener Personalbereich je Träger; eigener Bereich für trägerlose Einrichtungen; einrichtungsbezogene Einsätze nach ADR-016 |
| `portfolio-media` | `portfolios`, `portfolio_pages`, `observations`, `development_records`, `approvals`, `media_assets` | Einrichtungsschema und zugeordnete Dateiobjekte; Entwicklungsakte mit eigener Berechtigungsprüfung |
| `meals` | `meal_plans`, `meals`, `allergens`, `orders`, `cancellations`, `caterers` | Einrichtungsschema; kein Abrechnungsmodell |
| `communication-events` | Kalender, Nachrichten, Umfragen, allgemeine Veranstaltungen, Teilnehmer und Helferschichten | Einrichtungsschema; Tabellenbezeichnungen erst im Dienstvertrag |
| `council-bazaar-cash` | `councils`, `meetings`, `minutes`, Basarkonfiguration, `bazaar_tables`, `reservations`, `cash_entries`, `vouchers`; Wissen, Inventar und zusätzliche Elternservices | Einrichtungsschema mit getrennter fachlicher Freigabe für Elternbeirat und Einrichtung |
| `election` | `election_eligibility`, `anonymous_ballots`, `election_tallies` | Getrennte Datenbanken und Dienstidentitäten für Berechtigung und Stimme nach ADR-017 |
| `workflow-notification` | `workflow_definitions`, `workflow_instances`, Auftrags- und Zustellstatus | Einrichtungsschema; nur erforderliche Referenzen und Inhalte |
| `audit-privacy` | `consent_records`, `privacy_requests`, `audit_events`, Löschjournal | Mandantengebundener, restriktiver Bereich; getrennte Aufbewahrung |
| `media-worker` | Verarbeitung eines begrenzten Medienauftrags | Keine eigene Fachdatenbank; Zugriff nur auf die zugewiesenen Dateiobjekte |
| `gateway` | Öffentlicher API-Zugang gemäß ADR-015 | Keine Fachdatentabellen und keine Datenbankrolle |

`outbox_events` gehört jeweils zum erzeugenden Dienst. Es gibt keine gemeinsame
dienstübergreifende Outbox mit Zugriff auf sämtliche Fachinhalte.
Redis bleibt Transport oder Zwischenspeicher. PostgreSQL bleibt die dauerhafte Quelle.

Die Veranstaltungen aus `docs/03` gehören zum Dienst `communication-events`.
Ein Basar verweist auf eine solche Veranstaltung. Seine Plätze, Reservierungen
und Kassenbuchungen gehören zum Dienst `council-bazaar-cash`.
Helferschichten sind keine Personal-Dienstpläne. Diese bleiben bei `staff-scheduling`.

## Erlaubte Kommunikationszwecke

Jeder Aufruf MUSS eine geprüfte Dienstidentität sowie den erforderlichen
Benutzer-, Mandanten-, Zweck-, Objekt- und Feldumfang besitzen.
Der empfangende Dienst prüft diesen Umfang selbst. Ein Mandanten-Header allein genügt nicht.
Ein berechtigter Dienst besitzt dadurch keinen pauschalen Zugriff auf alle Benutzeraktionen.
Alle nicht beschriebenen Aufrufe bleiben gesperrt.
Neue Zwecke benötigen einen versionierten Dienstvertrag und eine Prüfung der Datenminimierung.

| Aufrufer | Ziel | Zulässiger Zweck und Datenumfang |
|---|---|---|
| Browser / SvelteKit-Server | `gateway` | Nur veröffentlichte, berechtigte API-Operationen; SSR leitet keine beliebigen Header weiter |
| `gateway` | `identity-access` | Anmeldung, Sitzungsprüfung, Widerruf und angeforderte Rechteentscheidung |
| `gateway` | `organization-config` | Berechtigte Organisationsauswahl, Kontextwechsel und Konfiguration |
| `gateway` | Fach-/Datenschutzdienst | Nur ausdrücklich veröffentlichte Operation; Ziel prüft Objekt- und Feldrechte erneut |
| Dienste mit Plattformbezug | `identity-access`, `organization-config` | Konkrete Rechte-/Mitgliedschaftsprüfung und registrierter Organisationskontext; keine vollständigen Verzeichnisabzüge |
| `attendance-care` | `children-admissions` | Für einen Vorgang nötige Kind-/Gruppenreferenz, Sorge- und Medikamentenberechtigung |
| `portfolio-media` | `children-admissions` | Berechtigte Kind-/Gruppenreferenz ohne pauschalen Zugriff auf Gesundheit oder Sorgebeschränkungen |
| `meals` | `children-admissions` | Nur erforderliche Bestellberechtigung und freigegebene Kostinformationen |
| `council-bazaar-cash` | `communication-events` | Veranstaltungsreferenz, ausdrücklich geteilte Informationen und verfügbare Teilnahmefunktionen |
| `portfolio-media` | `audit-privacy` | Aktuelle zweck- und einzelbildbezogene Einwilligung vor jeder Veröffentlichung |
| Fach-/Plattformdienste | `audit-privacy` | Minimierte Zugriffsmetadaten; keine vollständigen Kinder-, Gesundheits- oder Personalinhalte |
| `audit-privacy` | Zuständiger Fach-/Plattformdienst | Geprüfte Auskunft, Berichtigung, Löschprüfung oder Sperrung; keine direkten Tabellenzugriffe |
| Fach-/Plattformdienste | `workflow-notification` | Zweckgebundener Auftrag mit nötigen Empfängern und minimierten Inhalten |
| `workflow-notification` | Zuständiger Fach-/Plattformdienst | Versionierte, ausdrücklich zugelassene Aktion; der Zieldienst prüft erneut die aktuelle Berechtigung |
| `portfolio-media` | `media-worker` | Ein Medienauftrag mit begrenzten Eingabe-/Ausgabeobjekten; keine Identitätserkennung |
| `media-worker` | `portfolio-media` | Auftragsstatus und Verarbeitungsergebnis; keine eigenständige Veröffentlichung |
| `staff-scheduling` im Einrichtungskontext | Eigener Personalbereich | Nur erlaubte Einsatzdaten und Felder tatsächlich zugewiesener Mitarbeitender; kein Vollzugriff aufgrund des Einrichtungskontexts |
| `election` / Wahlberechtigung | Plattform-/Auditdienste | Nur geprüfte Verwaltungs- und Berechtigungsprozesse; keine Stimm- oder korrelierbaren Abgabedaten |
| `election` / Stimmabgabe | Plattform-/Auditdienste | Keine Identitätsauflösung und keine gemeinsame Identitätsspur; spätere Protokollprüfung verbindlich |

SSR bedeutet serverseitige Darstellung des Frontends.
Personalübergaben zwischen Einrichtungen erhalten keinen pauschalen Datenzugriff.
ADR-016 verlangt zweckgebundene Einsatzrechte. Die konkreten Felder und Endpunkte folgen in KC-015.
Externe Integrationen erhalten später eigene begrenzte Dienstverträge nach KC-028.

## Ereignisse und Ausfallverhalten

Außerhalb des gesondert zu prüfenden Wahlprotokolls schreibt der zuständige Dienst
Fachänderung und Outbox-Ereignis in derselben Transaktion.
Der empfangende Dienst verarbeitet eine Ereignis-ID idempotent, also ohne doppelte Wirkung.
Ein Ereignis ersetzt keine aktuelle Berechtigungs- oder Einwilligungsprüfung.
Unbekannte Versionen dürfen keine stillen fachlichen Änderungen auslösen.
Wiederholungen, Fehlerablage und Redis-Verfahren bleiben O-004.

Eine fehlgeschlagene Rechte- oder Einwilligungsprüfung darf keine Freigabe erzeugen.
Die sofortige Wirkung eines Widerrufs darf nicht durch einen ungeprüften Cache abgeschwächt werden.
Ein technisches Ereignisformat ist kein geprüfter Entwurf für das geheime Wahlprotokoll.
Korrelation zwischen Wähler und Stimme bleibt auch in Transport, Telemetrie und Datensicherung verboten.

## Entscheidungen und nachfolgende Nachweise

| ID | Gegenstand | Entscheidung oder nachfolgender Nachweis |
|---|---|---|
| O-001 | Einrichtungsschema, Dienstrollen und Migrationen | `1A`: ADR-014 entschieden; Nachweise in KC-005 |
| O-001-HR | Trägerweite Personalakte bei Mandant gleich Einrichtung | `3A`: Eigener Personalbereich je Träger nach ADR-016; Nachweise in KC-015 |
| O-001-WAHL | Trennung von Wahlberechtigung und Stimme | `4A`: Getrennte Datenbanken und Dienstidentitäten; unabhängige Protokollprüfung vor Implementierung in KC-025 |
| O-002 | Gateway und Frontend-Schnittstelle | `2A`: Eigenständiges Rust-Gateway nach ADR-015 |
| O-003 | Nachweis der Dienstidentität und Weitergabe des Benutzerkontexts | Aussteller, Laufzeit und Widerruf festlegen; bis dahin keine Dienstkommunikation implementieren |

Die Personal- und Wahlgrenzen wurden ausdrücklich gesondert bestätigt.
Die Entscheidungen ändern keine fachlichen Anforderungen.
O-003 bleibt eine nachfolgende technische Entscheidung vor Dienstkommunikation.
Sie wird über KC-037 verfolgt und verhindert keine Dokumentation der jetzt entschiedenen Grenzen.

## Geplanter Prüfnachweis vor Implementierungsabnahme

Diese Fälle sind Testanforderungen. Sie wurden noch nicht gegen Dienste ausgeführt.

| Fall | Erwartetes Ergebnis | Zuständiges Issue |
|---|---|---|
| Dienst liest fremde Diensttabelle im gleichen Einrichtungsschema | Datenbank verweigert Zugriff | KC-005 |
| Dienst liest eigene Tabellen einer fremden Einrichtung | RLS und Anwendungsprüfung verweigern Zugriff | KC-005, KC-008 |
| Pool-Verbindung wechselt Einrichtung nach Erfolg, Fehler oder Abbruch | Kein alter Organisationskontext bleibt wirksam | KC-005 |
| Laufzeitrolle erzeugt Tabellen, wechselt Eigentümer oder umgeht RLS | Datenbank verweigert die Aktion | KC-005 |
| Migration trifft falsches Schema oder falschen Dienstbestand | Vorprüfung bricht ab; kein Lauf als Anwendungsdienst | KC-005 |
| Eine Migrationswelle scheitert bei einer Einrichtung | Zustand bleibt eindeutig versioniert; weitere Wellen starten nicht unkontrolliert | KC-005 |
| Mitgliedschaft wird während aktiver Sitzung widerrufen | Nächste geschützte Aktion wird verweigert | KC-006, KC-008 |
| Hintergrundauftrag wird doppelt zugestellt oder nach Rollenentzug ausgeführt | Keine doppelte Wirkung; entzogene Rechte bleiben unwirksam | KC-028, KC-029 |
| Medienveröffentlichung folgt einem Widerruf oder Ausfall der Einwilligungsprüfung | Keine Veröffentlichung | KC-020, KC-030 |
| Elternbeirat ruft vertrauliche Kinder-/Personalinhalte ab | Keine Datenfreigabe, auch nicht über Kalender oder Berichte | KC-008, KC-022 |
| Einrichtung liest die vollständige Personalakte eines zugewiesenen Mitarbeitenden | Nur ausdrücklich erlaubte Felder werden geliefert | KC-008, KC-015 |
| Einrichtung oder Träger greift auf fremden Personalbereich zu | Zugriff wird verweigert; auch nach Einsatz- oder Kontextwechsel | KC-005, KC-015 |
| Eine Wahl-Laufzeitidentität versucht beide Wahldatenbanken zu lesen | Zugriff auf den fremden Bereich wird verweigert | KC-025 |
| Wähler- und Stimmdaten werden einschließlich Zeiten und Datensicherungen verglichen | Keine rückverfolgbare Verbindung; unabhängiger Nachweis erforderlich | KC-025 |
| Wiederherstellung enthält zuvor gelöschte Daten | Löschjournal und Einwilligungswiderrufe werden vor Freigabe durchgesetzt | KC-030, KC-033 |

## Abschlusskriterien für KC-002

Die Dienstzuordnung und Kommunikationszwecke sind dokumentiert.
Die Datenbankrollen und Migrationszuständigkeit folgen dem bestätigten ADR-014.
ADR-015 bis ADR-017 dokumentieren die weiteren ausdrücklichen Entscheidungen.
Damit sind die Architekturfragen für den P0-Umfang von KC-002 beantwortet.
Review und Integration des Dokumentations-PR bleiben vor Abschluss auf dem Hauptbranch erforderlich.
Ein grüner Dokumentationsprüfschritt ist kein Nachweis implementierter Sicherheitsmaßnahmen.
