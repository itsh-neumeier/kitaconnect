# ADR-044: Gemeinsame mTLS-Grundlage

- Datum: 22.10.2026
- Status: Technische Konkretisierung von ADR-019 vor Code
- Umfang: KC-006F; technischer Transport ohne Fachrechte

## Entscheidung

Der [Transportvertrag](../../contracts/service-transport-v1.md) legt eine gemeinsame technische Bibliothek fest.
Sie verwendet Rustls mit verpflichtender gegenseitiger Zertifikatsprüfung und Tokio-Rustls für den Datenstrom.
Private Typfelder binden Zertifikatsmetadaten an den tatsächlichen erfolgreichen Handshake.
Beide Endzertifikate werden auch auf wiederverwendeten Verbindungen erneut zeitlich geprüft.
Sitzungswiederaufnahme und frühe Anwendungsdaten bleiben deaktiviert.

## Begründung und Alternative

Die gemeinsame Bibliothek vermeidet unterschiedliche TLS-Prüfung in jedem Dienst.
Eine Ableitung aus behaupteten Headern wurde verworfen, weil sie keinen geprüften Ursprung belegt.
Eine alleinige Handshake-Zeitprüfung wurde verworfen, weil eine Verbindung länger als ihr Zertifikat bestehen kann.
Die Bibliothek enthält keine fachlichen Modelle oder dienstübergreifenden Tabellenzugriffe.

## Grenzen

Die 24-Stunden-Obergrenze stammt aus ADR-019. Es entsteht keine neue fachliche Berechtigung.
rcgen wird nur als Testabhängigkeit verwendet und trifft keine produktive Ausstellerentscheidung.
Die freie TCP-Portwahl im isolierten Test entscheidet nicht das offene Browserprofil.
Eine gültige Zertifikatskette ersetzt weder aktuelle Registrierung noch Tokenintrospektion oder Benutzerrechte.
Diese Nachweise und der Axum-HTTP-Vertrag folgen in getrennten Paketen.

## Notwendige Abhängigkeitspflege nach erstem CI-Lauf

Die Sicherheitsprüfung meldet am 22.10.2026 Tokio 1.54.0 als vom Paketregister zurückgezogen (`yanked`).
Der Fehler betrifft die bereits vorhandene gemeinsame Laufzeitversion und nicht eine Ausnahme für die TLS-Bibliothek.
Alle direkten Tokio-Abhängigkeiten werden deshalb vor erneuter Prüfung auf die exakte Patchversion 1.54.1 umgestellt.
Die [offizielle Paketdokumentation](https://docs.rs/tokio/1.54.1/tokio/) weist diese Version aus.
Die Warnung wird nicht unterdrückt. Vollständige Build-, Datenbank- und Sicherheitsprüfungen bleiben Pflicht.
Die frühere Versionsentscheidung in ADR-025 bleibt als damaliger Stand nachvollziehbar.
