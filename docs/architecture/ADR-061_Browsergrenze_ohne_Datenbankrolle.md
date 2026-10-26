# ADR-061: Browsergrenze ohne Datenbankrolle

- Datum: 26.10.2026
- Status: Technische Konkretisierung bestätigter Regeln vor Code
- Umfang: KC-006F; keine Fachfreigabe

## Entscheidung

Der [Browservertrag](../../contracts/gateway/browser-boundary-v1.md) führt das eigenständige Gateway als reine Rust-Bibliothek ein.
Die erste Komponente prüft genaue HTTPS-Origin, feste Anfrageprofile und eindeutige Cookie-/CSRF-Darstellung.
Ihr Ergebnis heißt ausdrücklich vorgelegter Nachweis und erteilt keine Anmeldung oder CSRF-Freigabe.
Der Identitätsdienst behält die spätere serverseitige Bindung und aktuelle Sitzungsentscheidung.
Das Gateway erhält keine Datenbankabhängigkeit oder Datenbankrolle.

## Begründung und Grenzen

Frühe Trennung verhindert, dass Cookie-Vorhandensein oder Clientheader als Benutzerberechtigung gelten.
Feste Profile schließen frei wählbare Zielpfade und Prüfmodi aus.
Der bestätigte Schutz vor Login-CSRF benötigt eine anonyme Vorbereitung und danach einen serverseitig gebundenen Nachweis.
Die Vorbereitung erteilt keine Benutzerrechte. Die konkrete Bindung folgt als eigener Schritt.
Die Projektleitung bestätigt am 26.10.2026 ausdrücklich die eigene eindeutige Anmeldekennung je Installation, unabhängig von E-Mail.
Ihr konkreter Kontenauflösungsvertrag folgt gesondert vor Code. Unabhängige bestätigte Browserregeln werden bereits umgesetzt.
Alle verwendeten externen Bibliotheksversionen stehen bereits im gesperrten Workspace. Es gibt keine externe Versionsänderung.
