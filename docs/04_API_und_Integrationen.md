# API- und Integrationskonzept

**KitaConnect Spezifikation v5.9 · 17.10.2026 · Konsolidierter Planungsstand**

> Status: Anforderungsspezifikation, keine implementierte oder rechtsverbindlich geprüfte Software. Die neuesten ausdrücklichen Interviewentscheidungen gelten.

## Leitlinien
Die Programmierschnittstelle (API) verwendet REST, OpenAPI 3.1, JSON und UTF-8.
API-Versionen folgen einer semantischen Versionierung.
Fehler werden einheitlich als Problem-Details nach RFC 9457 ausgegeben.
Ergebnislisten werden mit einem Fortsetzungszeiger seitenweise abgerufen.
Filter und Sortierung verwenden eine ausdrückliche Liste zulässiger Werte.
Anfragen sind mengenbegrenzt. Schreibaktionen sind idempotent und erzeugen bei Wiederholung keine doppelte fachliche Wirkung.
`organization_context` wird serverseitig aus der berechtigten Sitzung ermittelt.
Ein vom Client gesetzter Mandantenheader ist niemals alleinige Berechtigungsgrundlage.

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
Der Mailversand verwendet SMTP. Betriebshinweise erläutern die Absenderprüfverfahren SPF, DKIM und DMARC.
Web Push verwendet VAPID zur Identifikation des Anwendungsservers.
Webhooks werden mit einem schlüsselgebundenen Nachrichtencode (HMAC) signiert.
Sie verwenden Zeitstempel und Schutz gegen das erneute Einspielen derselben Nachricht.
Kalenderabonnements im ICS-Format verwenden widerrufbare Tokens.
Dateien liegen in S3, MinIO oder lokalem Speicher.
Die Anmeldung kann OpenID Connect (OIDC) und Security Assertion Markup Language (SAML) verwenden.
Externe Caterer erhalten gesonderte Berechtigungsumfänge. n8n wird über signierte Webhooks angebunden.
Dienstkonten erhalten nur die erforderlichen Rechte. Ihre Schlüssel werden regelmäßig gewechselt.
Exporte als CSV, Excel oder PDF sind versioniert und benötigen Berechtigungsprüfung sowie Zugriffsprotokollierung.

## Ereignisse
Vorgesehene Ereignisse sind `ChildAdmitted.v1`, `GuardianLinked.v1`, `AttendanceRecorded.v1`,
`ConsentWithdrawn.v1`, `ShiftPublished.v1`, `PortfolioPublished.v1` und `FacilityTransferred.v1`.
Jedes Ereignis trägt eine Ereigniskennung, Version und Zeitstempel.
Es trägt außerdem einen pseudonymisierten Berechtigungsumfang und eine technische Korrelationskennung.
Diese Kennung darf keine Gesundheitsdaten oder Daten zum Wahlgeheimnis enthalten.
Die weitergehenden Grenzen für Wahlkorrelation folgen ADR-017.
Eine Outbox hält ausstehende Ereignisse dauerhaft vor.
Die Verarbeitung ist idempotent. Fehlerablage und Wiederholungen sind vorgesehen.
Dienstvertragstests und definierte Zeiträume für Rückwärtskompatibilität sichern die Zusammenarbeit der Dienste.


## Dokumentationssprache (Ergänzung v5.9)
Für dieses Dokument gilt die deutsche Projektrichtlinie `13_Dokumentationsrichtlinie_DE_ASD_STE100.md` mit dem verbindlichen Fachwortverzeichnis `14_Fachwortverzeichnis_DE.md`. Die Richtlinie orientiert sich an ASD-STE100, stellt aber **keine offizielle deutsche ASD-STE100-Konformität** dar.
