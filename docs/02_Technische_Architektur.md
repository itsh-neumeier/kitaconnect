# Technische Architektur

**KitaConnect Spezifikation v5.9 · 17.10.2026 · Konsolidierter Planungsstand**

> Status: Anforderungsspezifikation, keine implementierte oder rechtsverbindlich geprüfte Software. Die neuesten ausdrücklichen Interviewentscheidungen gelten.

## Verbindliche Architekturentscheidungen
Rust (Axum, Tokio, SQLx), PostgreSQL, Redis, SvelteKit/Svelte 5, TypeScript, Tailwind, Docker Compose/Portainer, Kubernetes/Helm, GHCR. **Microservices frühzeitig**, gemeinsam im Monorepo. Separierte PostgreSQL-Schemas je Mandant, RLS zusätzlich, explizite Schema-/Rollen-/Rechtezuordnung. Zentrale Identität/Organisation ist eine besonders restriktive Plattformdomäne. Kein pauschaler Träger- oder Systemadmin-Bypass.

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
Ein Mandant ist eine Einrichtung; Träger sind Organisationsaggregate mit ausdrücklich freigegebenen Querzugriffen. Schema-Namen nie direkt aus Benutzerangaben interpolieren; nur serverseitig registrierte, gequotete Identifikatoren. Datenbank-`search_path` strikt transaktionsgebunden/qualifiziert, Connection-Pooling mit Isolationstests. Dienstrollen ohne BYPASSRLS; FORCE RLS soweit sinnvoll; Sicherheitsdefinierte Funktionen und Migrationen prüfen. RLS auf jeder sensiblen Mandantentabelle, auch wenn Schema-Trennung vorhanden. Plattformtabellen für Identität/Träger separat; Zuordnung von User/Role/Scope/Policy; keine personenbezogenen Kinderakten im globalen Index. Auditprotokolle mandantengebunden, append-only und getrennt aufbewahrt.

## Service- und Eventdesign

KC-037 konkretisiert die Dienstidentitäten in [ADR-019](architecture/ADR-019_Dienstidentitaeten.md)
und die Zustellung von Hintergrundaufträgen in [ADR-020](architecture/ADR-020_Hintergrundauftraege.md).
Diese technischen Festlegungen ändern keine Benutzerrechte und erteilen keinen Implementierungsauftrag.
OpenAPI pro Dienst, versionierte Ereignisse (z. B. ChildAdmitted.v1, AttendanceRecorded.v1, ConsentWithdrawn.v1), Transactional Outbox, idempotente Konsumenten, Dead-Letter, Retry-Backoff. Keine synchronen Cross-Service-Transaktionen; Saga/kompensierende Aktionen für Aufnahme, Wechsel und Austritt. Interservice-Authentisierung mit kurzlebigen Service-Identitäten, mTLS wo sinnvoll; Default-Deny und überprüfbare Scopes. Vermeide Microservice-Schnitt pro Bildschirm: fachlich kohärente Bounded Contexts, gemeinsam versionierte API-Verträge.

## Daten und Dateien
PostgreSQL je Mandant in eigenem Schema; Migrationen pro Schema mit Schema-Version, Sperr-/Rollback-Plan und automatischem Drift-Check. Redis für Cache/Queue, keine alleinige persistente Wahrheit. Medien: S3-kompatibel oder lokaler Storage; Mandantenpräfix/Bucket und separate Schlüssel/Policies; signierte URLs mit kurzer Laufzeit; EXIF-Strip und Face-Masking-Worker. Dateispeicher muss konsistent mit Datenbank gesichert werden. Suche und Reporting nur über autorisierte Projektionen; kein mandantenübergreifendes Suchindex-Leak.

## Identität und Rechte
Argon2id, WebAuthn/Passkeys, TOTP, Recovery, OIDC und SAML; Sitzung mit Secure/HttpOnly/SameSite Cookies und CSRF-Schutz; Step-up für sensible Aktionen. RBAC + objekt-/feldbezogene Policy-Auswertung, deny by default; Rollenwechsel erzeugt neuen geprüften Organisationskontext. Audits für sämtliche Lese-/Schreibzugriffe mit minimierten Metadaten und eigener Retention. Break-glass zeitbegrenzt, begründet, Zweitfreigabe soweit möglich und nachprüfbar.

## Skalierung und Ausfallsicherheit

Das bestätigte [Referenzprofil und Messverfahren](17_P0_Entscheidungsregister.md) bildet eine Einrichtung ab.
Die dokumentierten Zielwerte sind noch keine Messergebnisse oder allgemeinen Mindestressourcen.
Stateless API-Replikate, separate Worker-Pools, verteilte Jobs, Readiness/Liveness, idempotente Operationen. HA Postgres optional; PITR via WAL, verschlüsselte Backups. RPO/RTO jeweils Ziel 24 h (betriebsabhängig nachzuweisen). Mandantenspezifischer Restore via kontrolliertem temporärem Restore + selektiver Übernahme, niemals direktes Zurückrollen der gesamten Datenbank für einen Mandanten.

## Sicherheitsgrenzen
Geheime Wahlstimmen strikt von Wähleridentitäten isolieren, keinen Korrelationsschlüssel in Logs/Backups. Kasse append-only mit Gegenbuchungen; QR- und Basartransaktionen atomar. Uploads auf MIME, Malware und Größenlimits prüfen. Datenschutzrechtliche Anforderungen und kryptografisches Wahlprotokoll vor produktivem Einsatz separat prüfen.


## Dokumentationssprache (Ergänzung v5.9)
Für dieses Dokument gilt die deutsche Projektrichtlinie `13_Dokumentationsrichtlinie_DE_ASD_STE100.md` mit dem verbindlichen Fachwortverzeichnis `14_Fachwortverzeichnis_DE.md`. Die Richtlinie orientiert sich an ASD-STE100, stellt aber **keine offizielle deutsche ASD-STE100-Konformität** dar.
