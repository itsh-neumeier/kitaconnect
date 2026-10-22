# ADR-048: Tatsächliche Ausgaberestlaufzeit und grobe Fehlerklassen

- Datum: 22.10.2026
- Status: Technische Konkretisierung von ADR-042/046/047 vor Code
- Umfang: KC-006F, interne Voraussetzung für OAuth-Antworten

## Entscheidung

Der [Ausgabevertrag](../../contracts/identity-access/service-response-v1.md) bindet tatsächliche Ablaufmetadaten an die bestätigte Tokenausgabe.
Die Restlaufzeit wird aus aktueller Datenbankzeit berechnet. Die angeforderte Laufzeit genügt dafür nicht.
Ein nachweislich abgelaufenes oder zeitlich widersprüchliches Ergebnis bleibt für die Metadatenprüfung gesperrt.
Die vorhandene interne Ausgabe bleibt kompatibel; der erweiterte Ergebnistyp hat keinen öffentlichen Metadatenkonstruktor.

Der Dienstanmeldeadapter trennt grobe Ablehnung und Ausfall notwendiger Prüfungen.
Die Fehler enthalten ausschließlich feste Texte. SQL-Fehler und Eingabewerte gelangen nicht in Antworten.

## Begründung und Grenzen

Eine Zertifikatsobergrenze oder Wartezeit kann die angeforderte Laufzeit verkürzen.
Ein HTTP-Adapter darf deshalb keine ungeprüfte Laufzeitangabe übernehmen.
Eine einheitliche Fehlerklasse würde die erforderliche Unterscheidung zwischen `invalid_client` und HTTP 503 verhindern.
Die neuen Ergebnisse erteilen weder eine neue Dienstfreigabe noch Benutzer- oder Fachrechte.
Neue Migrationen, produktive Grenzen und erlaubende Datenbankpolicies sind nicht erforderlich.
