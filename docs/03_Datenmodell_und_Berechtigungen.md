# Datenmodell und Berechtigungsmatrix

**KitaConnect Spezifikation v5.9 · 17.10.2026 · Konsolidierter Planungsstand**

> Status: Anforderungsspezifikation, keine implementierte oder rechtsverbindlich geprüfte Software. Die neuesten ausdrücklichen Interviewentscheidungen gelten.

## 1. Kernentitäten
| Kontext | Tabellen / Aggregate (konzeptionell) | Schlüssel/Beziehungen |
|---|---|---|
| Plattform | identities, credentials, sessions, organizations, carriers, facilities, memberships | globale IDs, scopes |
| Konfiguration | module_flags, feature_flags, setting_versions, roles, permissions, policy_rules | Einrichtung/Träger + Version |
| Kinder | children, guardians, guardian_child_links, custody_restrictions, groups, enrollments, care_contracts | Einrichtungsschema; guardian-child Rechte |
| Betrieb | attendance_events, absences, pickup_authorizations, daily_care_logs, incidents, handovers | Kind/Gruppe/Zeitraum |
| Gesundheit | health_notes, medication_authorizations, medication_administrations, emergency_contacts | streng getrennte Zugriffsregeln |
| Personal | employees, employment_contracts, assignments, shifts, time_entries, leave_requests, qualifications | Träger- und Einrichtungsbezug |
| Pädagogik | portfolios, portfolio_pages, observations, development_records, approvals, media_assets | getrennte Freigabeebenen |
| Essen | meal_plans, meals, allergens, orders, cancellations, caterers | keine Rechnungs-/Payment-Entität |
| Elternbeirat | councils, meetings, minutes, events, volunteer_shifts, bazaar_tables, reservations, cash_entries, vouchers | Kasse und Kita-Finanzen getrennt |
| Wahl | election_eligibility, anonymous_ballots, election_tallies | keine rückverfolgbare Verbindung |
| Plattformbetrieb | consent_records, privacy_requests, audit_events, workflow_definitions, workflow_instances, outbox_events | Versionen, Löschfristen |

## 2. Berechtigungsmatrix (Basis, durch Policies verfeinert)
Legende: V=ansehen, B=bearbeiten, G=gezielt freigeben, -=standardmäßig kein Zugriff. Rechte gelten nur im expliziten Scope.

| Daten | Sorgeberechtigte | Fachkraft | Leitung | Elternbeirat | Träger-HR | Systemadmin |
|---|---|---|---|---|---|---|
| Eigenes Kind / Kontaktdaten | V/B* | V* | V/B | - | - | - |
| Anwesenheit Gruppe | V eigenes Kind | V/B zuständige Gruppe | V/B | - | - | - |
| Gesundheitsdaten | V eigenes Kind* | V* bei Bedarf | V* | - | - | - |
| Entwicklungsakte | nur G-Veröffentlichungen | V/B* | V/B* | - | - | - |
| Veröffentlichtes Portfolio | V eigenes Kind | V/B* | V/B | - | - | - |
| Personalakte | - | V eigene Daten | V/B* | - | V/B* | - |
| Dienstplan | - | V* | V/B* | - | V/B* | - |
| Elternbeiratskasse | - | - | - | V/B nach Rolle | - | - |
| Gemeinsamer Kalender | V freigegeben | V freigegeben | V/B | V/B freigegeben | V freigegeben | - |
| Audit | - | - | V* | - | V* | nur Betrieb/gezielt |

`*` = abhängig von Rolle, Einwilligung, Zweck, Objekt/Feld und Rechtsgrundlage; G-Veröffentlichungen nur explizit. Einrichtungsadmin darf nicht automatisch sämtliche sensiblen Akten lesen. Systemadmin verwaltet Infrastruktur, nicht pauschal Inhalte.

## 3. Sicherheitsinvarianten
Keine Zuordnung nur anhand von E-Mail oder Nachname. Sorgerechtsbeschränkungen gehen vor Standard-Elternrechten. Elternbeirat kann nicht über gemeinsame Feeds auf interne Kinder-/Personalakten zugreifen. Aufhebung einer Mitgliedschaft widerruft Sitzungsrechte sofort. Trägerübergreifende Personalrechte gelten nur für konkrete Einsatz-/HR-Aufgaben. Medienfreigaben werden bei jeder Veröffentlichung geprüft. Löschanträge beachten Rechtsgrundlage, Fristen und Backup-Löschjournal.

## 4. Migration und Schema-Lifecycle

Die [Dienstmatrix](architecture/KC-002_Dienstgrenzen.md) ordnet die oben genannten Entitäten ihrem zuständigen Dienst zu.
[ADR-014](architecture/ADR-014_Datenhoheit_und_Migrationen.md) trennt Laufzeit-, Eigentümer- und Migrationsrollen.
Der eigene Personalbereich je Träger folgt [ADR-016](architecture/ADR-016_Personalbereich.md).
Ein Einrichtungsbezug gewährt keinen Vollzugriff auf diesen Personalbereich.
Wahlberechtigung und Stimmabgabe liegen gemäß [ADR-017](architecture/ADR-017_Wahltrennung.md)
in getrennten Datenbanken mit getrennten Dienstidentitäten.
Diese technische Konkretisierung ändert keine fachlichen Berechtigungen.
Registrierte Schema-Namen, mandantenbezogene Migrationsversionen, transaktionssichere Schemaänderungen, getrennte Rollout-Wellen und Validierungsberichte. Globale Identitäten bleiben bei Einrichtungstransfers stabil, personenbezogene Fachakten werden nur mit expliziter Freigabe migriert.


## Dokumentationssprache (Ergänzung v5.9)
Für dieses Dokument gilt die deutsche Projektrichtlinie `13_Dokumentationsrichtlinie_DE_ASD_STE100.md` mit dem verbindlichen Fachwortverzeichnis `14_Fachwortverzeichnis_DE.md`. Die Richtlinie orientiert sich an ASD-STE100, stellt aber **keine offizielle deutsche ASD-STE100-Konformität** dar.
