# ADR-045: Prüfzugang zur tatsächlichen HTTP-Verbindung

- Datum: 22.10.2026
- Status: Technische Konkretisierung von ADR-019/044 vor Code
- Umfang: KC-006F; Verbindung zwischen TLS-Datenstrom und späteren HTTP-Handlern

## Entscheidung

Der [Vertrag zur HTTP-Verbindungsbindung](../../contracts/http-transport-binding-v1.md) ergänzt einen klonbaren Prüfzugang.
Nur die tatsächlich geprüfte Verbindung kann ihn erzeugen.
Der Zugang teilt Zeitbeobachtung, endgültige Fehlersperre und Verbindungsende mit dem Datenstrom.
Der spätere HTTP-Server ordnet ihn ausschließlich den Anfragen dieser Verbindung zu.
Jede geschützte Aktion verlangt eine aktuelle Prüfung, auch nach wartenden Vorprüfungen.

## Begründung und Alternative

Der HTTP-Server benötigt den Datenstrom für seine Ein-/Ausgabe.
Ein Handler kann deshalb nicht zugleich exklusiven veränderlichen Zugriff auf diesen Datenstrom erhalten.
Eine einmal kopierte Zertifikatsableitung wurde verworfen: Sie verliert die laufende Zeit- und Lebensdauerprüfung.
Ein vom Handler behaupteter Header wurde bereits in ADR-044 verworfen.
Eine gemeinsame kurze Zustandssperre serialisiert Zeitbeobachtungen paralleler Handler.
Sie wird nie über eine asynchrone Warteoperation gehalten.

## Grenzen

Eine erfolgreiche Zertifikatsprüfung ersetzt keine aktive Registrierung oder Tokenintrospektion.
Die spätere Introspektion berücksichtigt RFC 7662 und die Zertifikatsbindung aus RFC 8705.
Die bereits bestätigte OAuth-Fehlerformatausnahme bleibt erhalten.
Der vollständige Endpunktvertrag und seine OpenAPI-Beschreibung folgen vor Routercode.
Produktiver Schlüsselbetrieb, Browserprofil, Rollenbindung und Fachrechte bleiben gesondert.
Die bestehende TLS-Konfiguration und die bestätigten Laufzeitgrenzen bleiben erhalten.
