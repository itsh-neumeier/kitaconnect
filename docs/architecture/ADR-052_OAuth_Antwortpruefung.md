# ADR-052: Gemeinsame OAuth-Antwortprüfung

- Datum: 23.10.2026
- Status: Technische Konkretisierung vor Code
- Umfang: KC-006F, Verbraucherseite der vorhandenen OAuth-Schnittstelle

## Entscheidung

Der [Verbrauchervertrag](../../contracts/identity-access/service-oauth-consumer-v1.md) beschreibt ein gemeinsames reines Protokollpaket.
Es prüft Token- und Introspektionsantworten mit ausdrücklicher Bytegrenze und festen verdeckten Fehlern.
Bekannte mehrdeutige Felder werden abgewiesen. Unbekannte Erweiterungen bleiben innerhalb der Grenzen zulässig.
Introspektion bindet Aussteller, Dienste, Umfang, tatsächlichen Fingerabdruck und frische Zeit an ausdrücklich übergebene Erwartungen.

## Begründung und Grenzen

Der Organisationsdienst soll keine Datenbankimplementierung des Identitätsdienstes importieren.
Ein reines Paket vermeidet diese Abhängigkeit und doppelte Antwortparser.
Transportprüfung bleibt im Transportpaket; aktuelle Registrierung und zuständige Berechtigungen bleiben in den Diensten.
Die vorhandenen Scope- und Laufzeitregeln werden unverändert geprüft.
Der Parser erteilt keine Fachberechtigung und erhält keinen positiven Cache.
Produktiver Schlüsselbetrieb, aktuelle Organisationsanbindung und ursprüngliche Gesamtabnahme bleiben gesondert.
