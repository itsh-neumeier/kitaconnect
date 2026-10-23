# ADR-054: Interner Organisationsrouter

- Datum: 23.10.2026
- Status: Technische Konkretisierung vor Code
- Umfang: KC-006F / #67; lesende interne HTTP-Schnittstelle

## Entscheidung

Der [Dienstvertrag](../../contracts/organization-config/membership-http-v1.md) beschreibt eine einzelne
interne POST-Abfrage vor Code. Ihr Body enthält die beiden Kennungen; URLs enthalten keine Identitätsreferenz.
Der Token wird nur im Authorization-Header übertragen und bleibt an das tatsächliche mTLS-Zertifikat gebunden.
Der Router nutzt unverändert den authentisierten Leser aus ADR-053.
Sein privater Verbindungszustand stammt ausschließlich aus dem übernommenen TLS-Datenstrom.
Budgets und Fristen müssen ausdrücklich konfiguriert sein. Es gibt keinen Listener und keinen Berechtigungscache.

## Begründung und Grenzen

Die Bibliotheksprüfung besitzt bereits tatsächliche Introspektions- und Datenbanknachweise.
Der Router ergänzt eine begrenzte Transportform. Er erfindet keine Fachberechtigung oder MFA-Entscheidung.
Kennungen und Versionen werden verlustfrei als Dezimalzeichenketten übertragen.
Fehlender oder verborgener Bestand ergibt dieselbe Nullantwort.
Feste Problem Details setzen die allgemeine API-Regel um; dies ist kein OAuth-Protokollendpunkt.
Fehler schließen die Verbindung und geben keine Diagnosen aus.
Anfrage- und Verbindungsbudgets ergänzen die bestehenden Grenzen des Lesers.
Produktiver Zertifikatsbetrieb und Rollen-/Poolbindung bleiben offen.
Der vollständige Anmeldeadapter, Gateway und Browser folgen getrennt.
