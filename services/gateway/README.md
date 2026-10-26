# Gateway

Die Rust-Bibliothek beginnt die bestätigte Gateway-Grenze nach ADR-015 und ADR-061.
Sie prüft feste Browser-Anfrageprofile und Cookie-Ausgabe nach dem [Browservertrag](../../contracts/gateway/browser-boundary-v1.md).
Ein vorgelegtes Geheimnis ist keine Anmeldung, CSRF-Freigabe oder aktuelle Sitzung.
Die serverseitige Bindung bleibt beim Identitätsdienst. Die Bibliothek erhält keine Datenbankrolle und öffnet keinen Netzwerkport.
