# API- und Integrationskonzept

**KitaConnect Spezifikation v5.9 · 17.10.2026 · Konsolidierter Planungsstand**

> Status: Anforderungsspezifikation, keine implementierte oder rechtsverbindlich geprüfte Software. Die neuesten ausdrücklichen Interviewentscheidungen gelten.

## Leitlinien
REST/OpenAPI 3.1, JSON, UTF-8, semantische API-Versionierung, einheitliche Problem-Details (RFC 9457), Cursor-Pagination, Filter/Sortierung mit Allowlist, Rate-Limits und idempotente Schreibaktionen. `organization_context` wird serverseitig aus autorisierter Sitzung ermittelt; ein vom Client gesetzter Tenant-Header ist niemals alleinige Autorität.

## Beispielressourcen (konzeptionell, noch nicht implementiert)
| Methode | Pfad | Zweck | Sicherheitsregel |
|---|---|---|---|
| GET | /v1/me/memberships | zulässige Rollen/Einrichtungen | eigene Mitgliedschaften |
| POST | /v1/session/context | Kontextwechsel | Scope-Reautorisierung |
| GET/POST | /v1/children | Kinderlisten/Aufnahme | Einrichtung, Feldrechte |
| POST | /v1/attendance/events | Bringen/Abholen | Abhol-/Gruppenrechte, Idempotency-Key |
| GET/POST | /v1/staff/shifts | Dienstplanung | HR-/Leitungsrechte |
| GET/POST | /v1/portfolios/{id}/pages | Portfolio | Draft/Publish-Status |
| POST | /v1/media/{id}/publish | Bildveröffentlichung | Einwilligung + Freigabe |
| GET | /v1/meals/plans | Speiseplan | freigegebene Informationen |
| POST | /v1/privacy/requests | Auskunft/Löschung | Identitätsprüfung |
| GET | /v1/audit/events | Audit | strenger Audit-Scope |

## Integrationen
SMTP mit SPF/DKIM/DMARC-Hinweisen; Web Push (VAPID), signierte Webhooks (HMAC + Timestamp + Replay-Schutz), ICS-Abonnement mit widerrufbarem Token, S3/MinIO oder lokaler Speicher, OIDC/SAML, externe Caterer via dedizierte Scopes, n8n über signierte Webhooks. Dienstkonten mit minimalen Scopes und Schlüsselrotation. CSV/Excel/PDF als versionierte Exporte mit Berechtigungsprüfung und Audit.

## Events
`ChildAdmitted.v1`, `GuardianLinked.v1`, `AttendanceRecorded.v1`, `ConsentWithdrawn.v1`, `ShiftPublished.v1`, `PortfolioPublished.v1`, `FacilityTransferred.v1`; jedes Event mit Event-ID, Version, Zeitstempel, pseudonymisiertem Scope, Correlation-ID ohne Gesundheits-/Wahlgeheimnisdaten. Outbox und idempotente Consumer, Dead-Letter und Retry. Contract-Tests und Rückwärtskompatibilitätsfenster für Dienste.


## Dokumentationssprache (Ergänzung v5.9)
Für dieses Dokument gilt die deutsche Projektrichtlinie `13_Dokumentationsrichtlinie_DE_ASD_STE100.md` mit dem verbindlichen Fachwortverzeichnis `14_Fachwortverzeichnis_DE.md`. Die Richtlinie orientiert sich an ASD-STE100, stellt aber **keine offizielle deutsche ASD-STE100-Konformität** dar.
