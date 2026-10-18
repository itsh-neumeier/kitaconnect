# Technische Architektur

**KitaConnect Spezifikation v5.9 · 17.10.2026 · Konsolidierter Planungsstand**

> Status: Anforderungsspezifikation, keine implementierte oder rechtsverbindlich geprüfte Software. Die neuesten ausdrücklichen Interviewentscheidungen gelten.

## Verbindliche Architekturentscheidungen

Die technische Grundlage umfasst Rust mit Axum, Tokio und SQLx sowie PostgreSQL und Redis.
Das Frontend verwendet SvelteKit, Svelte 5, TypeScript und Tailwind.
Für den Betrieb sind Docker Compose mit Portainer sowie Kubernetes mit Helm vorgesehen.
Images liegen in der GitHub Container Registry (GHCR).
**Microservices entstehen frühzeitig und liegen gemeinsam im Monorepo.**

Jeder Mandant erhält ein getrenntes PostgreSQL-Schema.
Row-Level Security (RLS) ergänzt diese Trennung. Schemas, Rollen und Rechte werden ausdrücklich zugeordnet.
Zentrale Identitäts- und Organisationsdaten bilden einen besonders restriktiven Plattformbereich.
Träger- und Systemadministratoren erhalten keinen pauschalen Zugriff unter Umgehung der Regeln.

## Systemskizze

Die Projektleitung hat die technische Konkretisierung am 18.10.2026 mit `1A, 2A, 3A, 4A` bestätigt:

- [ADR-014](architecture/ADR-014_Datenhoheit_und_Migrationen.md): Ein Einrichtungsschema mit getrennten Dienstrechten und Migrationsrollen.
- [ADR-015](architecture/ADR-015_Gateway_Grenze.md): Eigenständiger Rust-Dienst als Gateway.
- [ADR-016](architecture/ADR-016_Personalbereich.md): Gesonderter Personalbereich je Träger und eigener Bereich für trägerlose Einrichtungen.
- [ADR-017](architecture/ADR-017_Wahltrennung.md): Getrennte Datenbanken und Dienstidentitäten für Wahlberechtigung und Stimmabgabe.

Die unabhängige Prüfung des Wahlprotokolls MUSS vor dessen Implementierung erfolgen.
Die [Dienstmatrix](architecture/KC-002_Dienstgrenzen.md) ordnet Entitäten und Kommunikationszwecke zu.
Diese Entscheidungen dokumentieren Architektur. Sie behaupten keine implementierten Dienste.
```text
PWA SvelteKit -> TLS/Ingress -> API Gateway / BFF
                              |-> Identity & Access Service
                              |-> Organization & Configuration Service
                              |-> Children & Admissions Service
                              |-> Attendance & Daily Care Service
                              |-> Staff & Scheduling Service
                              |-> Portfolio & Media Service -> isolated media worker
                              |-> Meals Service
                              |-> Communication & Events Service
                              |-> Parent Council / Bazaar / Cash Service
                              |-> Election Service (strong isolation)
                              |-> Workflow & Notification Service
                              |-> Audit & Privacy Service
Each service -> authorized PostgreSQL schemas + outbox; Redis queues; local/S3 object store
Observability -> OpenTelemetry/structured logs -> Prometheus/Grafana
```

## Trennung der Daten

Ein Mandant ist eine Einrichtung. Träger bündeln Organisationen und erhalten nur ausdrücklich freigegebene Querzugriffe.
Schemanamen dürfen nicht direkt aus Benutzerangaben in Abfragen übernommen werden.
Zulässig sind nur serverseitig registrierte Bezeichner mit korrekter SQL-Quotierung.
Der Datenbank-`search_path` ist strikt an die Transaktion gebunden; Objektbezüge werden qualifiziert verwendet.
Die gemeinsame Nutzung von Datenbankverbindungen benötigt Isolationstests.

Dienstrollen besitzen kein `BYPASSRLS`-Recht. `FORCE RLS` wird verwendet, soweit sinnvoll.
Funktionen mit den Rechten ihres Eigentümers und Migrationen benötigen eine Prüfung.
Jede sensible Mandantentabelle verwendet RLS, auch bei vorhandener Schematrennung.
Plattformtabellen für Identität und Träger bleiben getrennt.
Benutzer, Rolle, Berechtigungsumfang und Berechtigungsregel werden zugeordnet.
Der globale Index enthält keine personenbezogenen Kinderakten.
Zugriffsprotokolle sind mandantengebunden. Sie werden nur ergänzt und getrennt aufbewahrt.

## Dienste und Ereignisse

KC-037 konkretisiert die Dienstidentitäten in [ADR-019](architecture/ADR-019_Dienstidentitaeten.md)
und die Zustellung von Hintergrundaufträgen in [ADR-020](architecture/ADR-020_Hintergrundauftraege.md).
Diese technischen Festlegungen ändern keine Benutzerrechte und erteilen keinen Implementierungsauftrag.
Jeder Dienst beschreibt seine Schnittstelle mit OpenAPI.
Ereignisse sind versioniert, zum Beispiel `ChildAdmitted.v1`, `AttendanceRecorded.v1` und `ConsentWithdrawn.v1`.
Eine transaktionale Outbox speichert ausstehende Ereignisse dauerhaft zusammen mit der Fachänderung.
Die Verarbeitung ist idempotent: Wiederholungen erzeugen keine doppelte fachliche Wirkung.
Fehlerfälle verbleiben in einer Fehlerablage. Wiederholungen verwenden steigende Wartezeiten.

Es gibt keine synchronen Transaktionen über mehrere Dienste.
Aufnahme, Wechsel und Austritt verwenden koordinierte Teilschritte mit ausgleichenden Aktionen.
Die Authentisierung zwischen Diensten verwendet kurzlebige Dienstidentitäten.
Die ursprüngliche Vorgabe für mTLS gilt dort, wo sinnvoll; ADR-019 konkretisiert die verbindliche Dienstkommunikation.
mTLS bezeichnet TLS mit beidseitiger Zertifikatsprüfung. TLS schützt die Transportverbindung.
Zugriffe werden standardmäßig verweigert. Der zulässige Berechtigungsumfang ist überprüfbar.
Dienstgrenzen folgen fachlich zusammengehörigen Bereichen statt einzelnen Bildschirmen.
Die Dienstverträge werden gemeinsam versioniert.

## Daten und Dateien

PostgreSQL verwendet ein eigenes Schema je Mandant.
Migrationen benötigen je Schema eine Version, einen Sperrplan und einen Plan zum Rücksetzen.
Eine automatische Prüfung erkennt Abweichungen vom erwarteten Schema.
Redis dient als Zwischenspeicher und Auftragswarteschlange. Redis ist nicht die einzige dauerhafte Datenquelle.

Medien liegen in einem S3-kompatiblen oder lokalen Speicher.
Mandanten erhalten getrennte Präfixe oder Speicherbereiche sowie getrennte Schlüssel und Zugriffsregeln.
Signierte URLs gelten nur kurz. Ein Hintergrunddienst entfernt EXIF-Metadaten und maskiert Gesichter.
Dateispeicher und Datenbank werden konsistent gesichert.
Suche und Berichte verwenden nur berechtigte Datensichten.
Der Suchindex darf keine Daten zwischen Mandanten offenlegen.

## Identität und Rechte

Die Identitätsverwaltung verwendet Argon2id, WebAuthn beziehungsweise Passkeys und zeitbasierte Einmalpasswörter (TOTP).
Sie unterstützt Kontowiederherstellung, OpenID Connect (OIDC) und Security Assertion Markup Language (SAML).
Sitzungscookies verwenden `Secure`, `HttpOnly` und `SameSite`.
Schutz gegen gefälschte browserbasierte Anfragen (CSRF) ist erforderlich.
Sensible Aktionen benötigen eine zusätzliche Authentisierungsprüfung.

Rollenbasierte Berechtigungen (RBAC) werden durch Objekt- und Feldregeln ergänzt.
Zugriffe werden standardmäßig verweigert. Ein Rollenwechsel erzeugt einen neuen, geprüften Organisationskontext.
Alle Lese- und Schreibzugriffe werden mit minimierten Metadaten protokolliert.
Zugriffsprotokolle haben eigene Aufbewahrungsregeln.
Notfallzugriff ist zeitbegrenzt, begründet und nachprüfbar. Er benötigt eine Zweitfreigabe, soweit möglich.

## Skalierung und Ausfallsicherheit

Das bestätigte [Referenzprofil und Messverfahren](17_P0_Entscheidungsregister.md) bildet eine Einrichtung ab.
Die dokumentierten Zielwerte sind noch keine Messergebnisse oder allgemeinen Mindestressourcen.
API-Replikate sind zustandslos. Hintergrunddienste verwenden getrennte Ausführungsgruppen.
Aufträge sind verteilt. Prüfungen erfassen Betriebsbereitschaft und Lebenszeichen.
Operationen sind idempotent. Hochverfügbares PostgreSQL ist optional.
Die Wiederherstellung zu einem Zeitpunkt (PITR) verwendet das PostgreSQL-Schreibprotokoll (WAL).
Datensicherungen sind verschlüsselt.

Der angestrebte maximale Datenverlustzeitraum (RPO) beträgt 24 Stunden.
Die angestrebte Wiederherstellungsdauer (RTO) beträgt ebenfalls 24 Stunden.
Beide Ziele benötigen betriebsabhängige Nachweise.
Eine mandantenspezifische Wiederherstellung verwendet eine kontrollierte temporäre Umgebung und selektive Datenübernahme.
Die gesamte Datenbank darf dafür niemals direkt auf einen früheren Stand zurückgesetzt werden.

## Sicherheitsgrenzen

Geheime Wahlstimmen bleiben strikt von Wähleridentitäten getrennt.
Protokolle und Datensicherungen dürfen keinen Zuordnungsschlüssel zwischen Person und Stimme enthalten.
Kassenbuchungen werden nur ergänzt. Korrekturen erfolgen durch Gegenbuchungen.
QR- und Basartransaktionen sind atomar.
Uploads werden auf Dateityp (MIME), Schadsoftware und Größenlimits geprüft.
Datenschutzrechtliche Anforderungen und das kryptografische Wahlprotokoll benötigen vor produktivem Einsatz gesonderte Prüfungen.


## Dokumentationssprache (Ergänzung v5.9)

Es gelten die [deutsche Dokumentationsrichtlinie](13_Dokumentationsrichtlinie_DE_ASD_STE100.md)
und das [Fachwortverzeichnis](14_Fachwortverzeichnis_DE.md).
Die Richtlinie adaptiert Sprachprinzipien von ASD-STE100.
Sie ist keine offizielle deutsche Übersetzung und belegt keine formelle ASD-STE100-Konformität.
