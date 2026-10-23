# Verbrauchervertrag: OAuth-Antwortprüfung v1

- Stand: 23.10.2026
- Umfang: KC-006F / #67, Prüfung bereits begrenzt empfangener Antworten
- Grundlage: ADR-019/046/051, bestehender OAuth-Vertrag

## Trennung und Voraussetzungen

Das gemeinsame Rust-Paket `service-oauth` prüft ausschließlich Protokolldaten.
Es erhält weder Datenbank-, Transport- noch Dienstabhängigkeiten.
Dadurch kann der Organisationsdienst Antworten prüfen, ohne den Identitätsdienst einzubinden.
Der Transport prüft weiterhin tatsächliches mTLS, Header, Cache-Regeln und Zeitlimits.
Der Aufrufer muss die aktuelle zuständige Serverregistrierung gesondert prüfen.
Ein erfolgreicher Parser beweist weder Registrierung noch Mitgliedschaft, MFA oder Fachrechte.

Der Aufrufer setzt eine positive maximale Bodygröße ausdrücklich fest.
Nur HTTP 200 und genau ein JSON-Objekt ohne nachfolgende Daten können positiv enden.
JSON-Typfehler, doppelte bekannte Felder einschließlich escaped Feldnamen und bekannte Fehlerfelder werden abgewiesen.
Unbekannte Erweiterungsfelder werden entsprechend RFC 6749 ignoriert; die Byte- und JSON-Tiefengrenzen gelten weiter.
Der Parser gibt ausschließlich einen festen Fehler ohne Eingabedaten aus.

## Tokenantwort

Erforderlich sind `access_token`, `token_type`, `expires_in` und der bereits im Projektvertrag verlangte `scope`.
Das Token besteht kanonisch aus 32 Bytes in 43 Base64url-Zeichen ohne Padding.
`token_type` ist gemäß RFC 6749 unabhängig von Groß-/Kleinschreibung `Bearer`.
`expires_in` ist eine ganze Zahl von 1 bis 300 entsprechend dem bestehenden Diensttokenvertrag.
`scope` stimmt genau mit dem ausdrücklich erwarteten, nicht leeren Umfang überein.
Ein `error` oder `refresh_token` verhindert ein positives Ergebnis; ein Refresh-Verfahren ist nicht Bestandteil dieses Client-Credentials-Profils.
Der Ergebnistyp hält das Token in einem beim Freigeben überschriebenen String und verdeckt es in Debug-Ausgaben.
Andere interne Parser-/Bibliothekspuffer werden dadurch nicht pauschal als überschrieben behauptet.
Die Restlaufzeit ist eine empfangene Angabe und keine neue lokale Ablauf- oder Cachefreigabe.

## Introspektionsantwort

`active` ist erforderlich und boolesch. `false` ergibt ausschließlich den internen Zustand inaktiv.
Optionale bekannte Felder müssen auch bei inaktiven Antworten ihren richtigen Typ haben.
Bei `true` sind `client_id`, `iss`, `aud`, `scope`, `iat`, `exp` und `cnf.x5t#S256` erforderlich.
Der Aufrufer gibt erwarteten Aussteller, aufrufenden Dienst, Zieldienst, Umfang und tatsächlichen Zertifikatsfingerabdruck aus vertrauenswürdiger Konfiguration beziehungsweise geprüfter eingehender Verbindung vor.
Alle Werte müssen genau übereinstimmen. Der Fingerabdruck hat dieselbe kanonische 32-Byte-Darstellung wie oben.
Ein `error` verhindert ein positives Ergebnis. Doppelte bekannte Felder im `cnf`-Objekt werden ebenfalls abgewiesen.

Die frische vertrauenswürdige Zeit wird nach dem HTTP-Aufruf übergeben.
Es gilt `iat <= jetzt < exp` und `1 <= exp - iat <= 300` ohne arithmetischen Überlauf.
Der Zustand ist eine geprüfte Momentaufnahme; er darf keine positive Berechtigung für spätere Anfragen zwischenspeichern.
Aktueller Entzug wird weiterhin durch eine neue zuständige Prüfung erkannt.
Vertrauenswürdige Zeitbeschaffung, aktuelle Registrierung und der tatsächliche Organisationsadapter bleiben gesonderte Aufgaben.

## Nachweise und Abhängigkeiten

Deterministische Tests prüfen Typen, doppelte/fehlende Felder, Fremdwerte, Zahlen-/Zeitgrenzen,
kanonische Token/Fingerabdrücke, Erweiterungen, Tiefen-/Bytegrenzen und verdeckte Ausgaben.
Die vorhandene tatsächliche Client-/Server-/Datenbankgruppe verwendet die Parser vor und nach Neustart.
Sie übergibt die tatsächlichen Testzertifikate und frische Datenbankzeit; sie ist kein produktiver Organisationsadapter.
Direkte Pakete verwenden ausschließlich bestehende Lockfile-Versionen von Serde, Serde JSON, Base64 und Zeroize.
Es gibt keine neue externe Paketversion, Migration, Rolle oder erlaubende Policy.

Quellen: [RFC 6749, Abschnitt 5.1](https://www.rfc-editor.org/rfc/rfc6749#section-5.1),
[RFC 6749, Abschnitt 4.4.3](https://www.rfc-editor.org/rfc/rfc6749#section-4.4.3),
[RFC 7662, Abschnitt 2.2](https://www.rfc-editor.org/rfc/rfc7662#section-2.2),
[RFC 8705, Abschnitt 3.1](https://www.rfc-editor.org/rfc/rfc8705#section-3.1).
