# ADR-014 – Datenhoheit und Migrationen je Dienst

Datum: 18.10.2026. Status: von der Projektleitung entschieden; Umsetzung ausstehend.
Bezug: ADR-001, ADR-002, O-001 und [KC-002](https://github.com/itsh-neumeier/kitaconnect/issues/2).

## Verbindliche Ausgangslage

Ein Mandant ist eine Einrichtung. Die Spezifikation fordert PostgreSQL-Schemas je Mandant
und zusätzliche Row-Level Security (RLS). Frühe Microservices bleiben verbindlich.
Ein Dienst darf keine Fachtransaktion über fremde Diensttabellen ausführen.

## Entscheidung – Ein Einrichtungsschema mit getrennten Dienstrechten

Die Datenbank enthält ein registriertes PostgreSQL-Schema je Einrichtung.
Jeder Dienst erhält darin nur Rechte an seinen eigenen Tabellen und Sequenzen.
Technische Namenspräfixe unterscheiden die Dienstbestände. Ein Präfix ist keine Sicherheitsgrenze.
Die Sicherheitsgrenze bilden ausdrücklich vergebene Objektrechte, RLS und die Anwendungsprüfung.
Gemeinsame Fachtabellen, gemeinsame Laufzeitrollen und dienstübergreifende Fremdschlüssel sind ausgeschlossen.
Referenzen auf andere Dienste werden über geprüfte Dienstverträge aufgelöst.

Die globale Identität und Organisationsverwaltung erhalten getrennte restriktive Plattformbereiche.
Die [Dienstmatrix](KC-002_Dienstgrenzen.md) ordnet die konzeptionellen Entitäten zu.
Die ergänzenden Entscheidungen [ADR-016](ADR-016_Personalbereich.md) und
[ADR-017](ADR-017_Wahltrennung.md) regeln Personalbereiche und die getrennten Wahldatenbanken.
Diese besonders geschützten Bereiche erweitern keine allgemeinen Dienstrechte.

| Rolle | Zulässige Aufgabe | Ausgeschlossene Rechte |
|---|---|---|
| Laufzeitrolle je Dienst | Explizite Lese-/Schreibrechte auf eigene Objekte im autorisierten Kontext | Kein Eigentum, kein CREATE im Schema, kein BYPASSRLS, kein Superuser, keine Migrationsrolle |
| Eigentümerrolle je Dienst | Eigentum am Dienstbestand; kein direkter Login | Keine Verwendung durch Anwendungsdienste; keine pauschale Eigentümerschaft an fremden Tabellen |
| Migrationsrolle je Dienst | Geprüfte Änderungen des eigenen Bestands im Wartungsprozess | Keine Ausgabe ihrer Zugangsdaten an Laufzeitdienste |
| Provisionierungsrolle | Registrierte Einrichtungsschemas und geprüfte Rollen-/Objektzuordnungen anlegen | Keine Nutzung für Fachanfragen |
| Migrationssteuerung | Freigegebene Migrationsartefakte je Dienst und Einrichtung koordinieren | Keine öffentliche Fachschnittstelle und keine dauerhafte Laufzeitidentität mit DDL-Rechten |
| Gateway und Frontend | Dienstverträge aufrufen | Keine Fachdatenbankrolle |

DDL bezeichnet Anweisungen zur Änderung der Datenbankstruktur.
Ein CREATE-Recht in einem gemeinsam genutzten Schema kann nicht durch einen Namenspräfix begrenzt werden.
Migrationen sind deshalb privilegierte, geprüfte Betriebsaktionen.
KC-005 MUSS auch Namensverdeckung, Funktionsauflösung und Eigentümerwechsel prüfen.
Ein freigegebener Tabellenname wird vollständig qualifiziert verwendet.
Ein vom Client gelieferter Schemaname darf niemals zur Autorität werden.

## Zuständigkeit und Migrationsablauf

1. Der fachlich zuständige Dienst liefert seine Migration, Vertragsversion und Kompatibilitätsangaben.
2. Die Provisionierung ordnet die Einrichtung einem registrierten Schema zu.
3. Die Migrationssteuerung prüft freigegebenes Artefakt, Objektliste, Dienstrolle und Ausgangsversion.
4. Sie sperrt konkurrierende Schemaänderungen derselben Einrichtung und startet die Dienstmigration.
5. Sie prüft Objektrechte, RLS und Versionsstand. Sie protokolliert Erfolg oder Fehler je Dienst und Einrichtung.
6. Bei Fehler stoppt sie die nächste Welle. Sie verwendet den vorab geprüften Wiederherstellungs- oder Vorwärtskorrekturplan.

Die Laufzeitdienste führen keine Migrationen beim Start aus.
Sensible Mandantentabellen benötigen RLS. KC-005 prüft zusätzlich FORCE ROW LEVEL SECURITY,
Eigentümerrechte und jede ausdrücklich begründete Ausnahme.
Es gibt keinen pauschalen Rollback durch Zurücksetzen einer gesamten Mehrmandanten-Datenbank.
Das genaue Sperrverfahren und die Einbindung von SQLx folgen in KC-005.
Ein Kontext für RLS MUSS aus einer vertrauenswürdig geprüften Identität stammen.
Ein beliebig durch die Laufzeitrolle gesetzter Sitzungswert ist allein kein Isolationsnachweis.

## Nicht gewählte Alternative – Getrennte Dienstdatenbanken mit Einrichtungsschemas

Jeder Dienst verwendet eine eigene Datenbank und darin registrierte Einrichtungsschemas mit RLS.
Das trennt Datenbankrechte stärker. Es erhöht den Aufwand für Provisionierung,
Verbindungen, Datensicherung, Wiederherstellung und dienstübergreifende Abläufe.
Die Anzahl der Datenbanken ist in v5.9 nicht ausdrücklich entschieden.
Diese allgemeine Dienstaufteilung wurde nicht gewählt. Die Wahltrennung nach ADR-017 bleibt verpflichtend.

Ein gemeinsamer Dienstbenutzer mit Vollzugriff ist keine zulässige Alternative.
Er widerspricht den verbindlichen Sicherheitsgrenzen.

## Entscheidung und Konsequenzen

Die Projektleitung hat Variante A mit der Antwort `1A` am 18.10.2026 bestätigt.
Die Quelle ist die ausdrückliche Antwort `1A, 2A, 3A, 4A` im Projektchat.
Vor Umsetzung sind die privilegierten Migrationsrechte ausdrücklich zu prüfen.
O-001 ist damit auf Architekturebene entschieden.
Die Personal- und Wahlgrenzen wurden gesondert mit `3A` und `4A` bestätigt.
Der [Prüfplan](KC-002_Dienstgrenzen.md) nennt die nachzuweisenden Negativfälle.

**Entscheidung der Projektleitung:** Variante A bestätigt. P0 dokumentiert die Entscheidung.
**Umsetzung:** keine. Keine Rollen, Tabellen, Migrationen oder Zugangsdaten angelegt.

## Technische Quellen

Die PostgreSQL-Dokumentation beschreibt getrennte Objektrechte und das Risiko
beschreibbarer Schemas im `search_path`. Sie beschreibt außerdem die besonderen
RLS-Rechte von Superusern, Rollen mit BYPASSRLS und Tabelleneigentümern.
Diese Quellen stützen die genannten Prüfanforderungen. Sie genehmigen den Architekturvorschlag nicht.

- [PostgreSQL 18: Schemas und Rechte](https://www.postgresql.org/docs/18/ddl-schemas.html).
- [PostgreSQL 18: Row Security Policies](https://www.postgresql.org/docs/18/ddl-rowsecurity.html).

PostgreSQL 18 ist hier die Bezugsdokumentation. Die Laufzeitversion bleibt O-006.
