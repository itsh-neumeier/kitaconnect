# Dienstvertrag: OAuth-Dienstschnittstelle v1

- Stand: 22.10.2026
- Umfang: KC-006F / #67, Protokollmodell vor Routercode
- Status: Protokollmodell und Zertifikatszuordnung bestätigt; Laufzeit noch nicht implementiert

## Umfang und Quellen

Der [OpenAPI-3.1-Vertrag](../openapi/identity-access/service-oauth-v1.json) beschreibt zwei interne Endpunkte.
`POST /oauth/token` stellt Diensttokens über Client Credentials aus.
`POST /oauth/introspect` prüft den aktuellen Diensttokenzustand.
Dieser erste Implementierungsschritt enthält nur Formularprüfung, Darstellungen und Dienstvertragstests.
Er öffnet keinen Listener und führt keine Anmeldung oder Datenbankaktion aus.

Grundlagen: [RFC 6749, Abschnitte 3.2, 4.4 und 5](https://www.rfc-editor.org/rfc/rfc6749),
[RFC 7662, Abschnitt 2](https://www.rfc-editor.org/rfc/rfc7662#section-2),
[RFC 8705, Abschnitte 2 und 3.2](https://www.rfc-editor.org/rfc/rfc8705),
[OpenAPI 3.1.1](https://spec.openapis.org/oas/v3.1.1.html).
Die bestätigte OAuth-Fehlerformatausnahme gilt. Fach-APIs behalten Problem Details.

## Authentisierung und bestätigte Zuordnung

Beide Endpunkte verlangen die tatsächliche gegenseitige Zertifikatsprüfung aus ADR-044/045.
Der Dienst muss `client_id` mit der aktuellen Registrierung des tatsächlichen Endzertifikats verbinden.
Die Introspektion authentisiert ihren Aufrufer ohne weiteres Diensttoken und ohne rekursive Abfrage.
Ein Formularmodell oder ein `PeerReference` aus behaupteten Metadaten beweist diese Herkunft nicht.

RFC 8705 unterscheidet zwei Verfahren zur Clientauthentisierung.
Das Verfahren mit Zertifizierungsstelle verlangt zusätzlich einen registrierten Namen aus dem Zertifikat.
Der vorhandene Fingerabdruckbestand ist allein noch kein vollständiger Nachweis dieses Verfahrens.
Die Projektleitung bestätigt am 22.10.2026 ausdrücklich folgende Zuordnung:
Genau ein ausdrücklich registrierter DNS-Name je Dienst, ohne Wildcard oder automatische Registrierung.
Zusätzlich bleibt der aktive genaue Zertifikatsfingerabdruck verpflichtend.
Produktive Namen, Aussteller und Schlüsselbetrieb bleiben offen.
Diese Entscheidung wird vor dem abhängigen Code versioniert. Sie ersetzt keine Implementierungs- oder Testnachweise.

## Formularprüfung

Anfragen verwenden ausschließlich `application/x-www-form-urlencoded` und UTF-8.
Ein zukünftiger Router prüft Methode, Inhaltsformat, TLS-Herkunft und Anfragelimits vor der Verarbeitung.
Body- und Parametergrenzen werden ausdrücklich positiv konfiguriert. Es gibt keine produktiven Vorgabewerte.
Der Protokollparser prüft die Bodylänge vor Allokation sowie die Parameterzahl vor dem Dekodieren.
Ungültige Prozentkodierung oder ungültiges UTF-8 verweigert die Anfrage.
Parameter dürfen auch nach Dekodierung ihres Namens nicht doppelt vorkommen.
Leere Werte werden wie fehlende Werte behandelt. Unbekannte Parameter werden innerhalb der Grenzen ignoriert.
`client_secret` und `client_assertion` sind bekannte, hier nicht erlaubte Anmeldemittel und werden abgewiesen.
Ein späterer Router darf keine alternative Anmeldung über Authorization-Header oder Browsercookie zulassen.
Tokens dürfen nicht über URL-Parameter übertragen oder protokolliert werden.

## Tokenanfrage und Antwort

Erforderlich sind `grant_type=client_credentials`, `client_id` und genau ein `scope`.
Zulässige Dienstkennungen sind `identity-access`, `organization-config` und `gateway`.
Die Kennung ist eine Behauptung, bis die aktuelle Zertifikatsregistrierung sie bestätigt.
Ein unbekannter Grant ergibt `unsupported_grant_type`; fehlender oder unbekannter Scope ergibt `invalid_scope`.
Fehlender Grant oder fehlende Kennung ergibt `invalid_request`; eine unbekannte Kennung ergibt `invalid_client`.
Der Scope bestimmt das Ziel eindeutig: `membership:read` führt zu `organization-config`;
`login:local` und `session:check` führen zu `identity-access`.
Der aktuelle Freigabebestand prüft weiterhin die bestehende Aufrufer-/Ziel-/Zweckmatrix aus ADR-042.
Das Parsen eines bekannten Scopes erteilt keine Freigabe.

Ein ausgegebener Token hat 32 zufällige Bytes. Die HTTP-Darstellung verwendet kanonisches Base64url ohne Füllzeichen.
Das ergibt 43 Zeichen. Abweichende Darstellung, Länge oder ungenutzte Restbits werden nicht als derselbe Token akzeptiert.
Base64 ist eine Darstellung und kein eigenes Kryptoverfahren.
Die spätere erfolgreiche Antwort enthält `access_token`, `token_type=Bearer`, `scope` und `expires_in`.
`expires_in` muss die tatsächlich verbleibende Laufzeit bei Antworterzeugung abbilden, höchstens 300 Sekunden.
Eine angeforderte Laufzeit darf nicht ungeprüft übernommen werden: Zertifikatsende und Wartezeiten können sie verkürzen.
Vor einer solchen Antwort muss der Ausgabeadapter bestätigte Ablaufmetadaten bereitstellen.
Dieser Teil implementiert deshalb noch keine positive Tokenantwort aus frei behaupteten Fristen.
Es gibt keinen Refresh-Token und keine Benutzerinformationen.

## Introspektion und Fehler

Erforderlich sind `client_id` und ein nicht leerer `token`. `token_type_hint` ist optional und nicht maßgeblich.
Ein nicht kanonischer, unbekannter, abgelaufener, widerrufener oder fremder Token ergibt nach berechtigter Abfrage nur `{"active":false}`.
Ein solcher Token ergibt keine Aufruffreigabe und keine Information über den Ablehnungsgrund.
Bei `active=true` enthält die Antwort ausschließlich `active`, `client_id`, `scope`, `iss`, `aud`, `iat`, `exp` und `cnf`.
`cnf` enthält `x5t#S256` als kanonische Base64url-Ableitung des ursprünglichen Clientzertifikats.
Der empfangende Dienst muss diese Bindung mit seiner tatsächlichen Aufruferverbindung vergleichen.
Er prüft außerdem aktuellen Aussteller, Empfänger, Zweck und Zeit. Positive Antworten werden nicht zwischengespeichert.
Der Antwortserializer bildet nur bereits geprüfte interne Metadaten ab. Er führt diese Prüfungen nicht selbst aus.

OAuth-Fehler enthalten ausschließlich einen standardisierten `error`-Wert und keine Diagnosedaten.
Formularfehler verwenden HTTP 400. Fehlende Clientauthentisierung wird am Tokenendpunkt als HTTP 400 / `invalid_client` ausgegeben.
Der Introspektionsendpunkt verwendet dafür gemäß RFC 7662 HTTP 401 / `invalid_client`.
Ein fehlgeschlagener TLS-Handshake erreicht keinen HTTP-Handler.
Ausfall notwendiger Infrastruktur ergibt eine leere HTTP-503-Antwort und niemals `active=false` als behauptetes Prüfergebnis.
Es wird dafür kein nicht definierter OAuth-Fehlercode erfunden.
HTTP 413, 415 und 429 sowie Methodenablehnung erhalten leere Antworten.
Alle Endpunktantworten verlangen `Cache-Control: no-store` und `Pragma: no-cache`.
Keine Fehlerantwort oder Debug-Ausgabe darf Formularwerte, Roh-Tokens oder SQL-Details enthalten.

## Nachweise und Folgearbeit

Parserprüfungen decken kodierte doppelte Namen, UTF-8, Prozentkodierung, leere Werte, unbekannte Erweiterungen,
alternative Anmeldemittel, mehrere Scopes, exakte Mengenbegrenzung und fremde Tokenformate ab.
Darstellungsprüfungen decken kanonische Tokenkodierung, Restbits und minimale inaktive Antworten ab.
Die aktive Darstellung wird getrennt gegen echte intern geprüfte Tokenmetadaten geprüft.
Diese Tests sind keine Router-, TLS-, Registrierungs- oder Browsernachweise.

Die Implementierung verwendet ausschließlich bereits vorhandene Paketversionen:
[percent-encoding 2.3.2](https://docs.rs/percent-encoding/2.3.2/percent_encoding/) und
[base64 0.22.1](https://docs.rs/base64/0.22.1/base64/) erhalten direkte Abhängigkeitseinträge.
Keine Migration, Datenbankrechte oder Laufzeitpolicy wird geändert.
Die Zertifikatszuordnung ist bestätigt. Der nächste Schritt benötigt den getrennten Registrierungs-/Routervertrag und dessen Nachweise.
Das zusätzliche HTTPS-Browserprofil ist inzwischen ausdrücklich bestätigt und folgt dem getrennten [Testprofilvertrag](browser-https-test-v1.md).
Dieser Protokollbaustein implementiert keinen Browsernachweis.
