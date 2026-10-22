# Dienstvertrag: Tatsächlicher OAuth-HTTP-Adapter v1

- Stand: 22.10.2026
- Umfang: KC-006F / #67, interner HTTP-Adapter ohne produktiven Startbefehl
- Grundlage: ADR-019/042/045 bis ADR-049 und OAuth-OpenAPI-Vertrag

## Tatsächlicher Transport und Grenzen

Der Adapter verwendet Axum 0.8.9, Hyper 1.12.0 und Hyper-Util 0.1.21.
Der Server verarbeitet ausschließlich HTTP/1.1 auf dem bestehenden tatsächlich geprüften mTLS-Datenstrom.
Der Router erhält den Prüfzugang ausschließlich aus genau dieser Verbindung, niemals aus Headern oder Formularfeldern.
Die Verbindungszuordnung bleibt für alle Anfragen dieser Verbindung erhalten. Jede geschützte Anfrage prüft sie erneut.
Die vorhandene gegenseitige Zertifikatsprüfung darf nicht abgeschaltet werden.

Der Aufrufer übergibt ausdrücklich positive Grenzen für Verbindungen, gleichzeitige Anfragen, Bodylänge, Parameterzahl,
Handshake, gesamte Verbindung, Anfragezeit, Headerzeit, Headerzahl und Headerpuffer.
Es gibt keine produktiven Vorgabewerte, Domain oder Umgebungsvariablen-Ausweichkonfiguration.
Der Headerpuffer berücksichtigt die technische Hyper-Untergrenze von 8.192 Bytes.
Ein erschöpftes Verbindungsbudget schließt den neuen Socket vor dem Handshake.
Ein erschöpftes Anfragebudget ergibt HTTP 429 ohne Body. Zeitüberschreitung einer Anfrage ergibt HTTP 503 ohne Body.
Der Adapter öffnet keinen Listener und startet keine unbeschränkte Annahmeschleife.
Er verarbeitet einen ausdrücklich übergebenen Socket unter diesen Grenzen.

## Datenbank und Abbruch

Jede Anfrage erhält eine eigene besitzende PostgreSQL-Verbindung aus ausdrücklich übergebenen Verbindungsoptionen.
Der Adapter erhält keinen Pool, setzt keine Datenbankrolle und erzeugt keine erlaubende Policy.
Produktive Rollen-/Poolbindung bleibt ein eigener Prüfschritt. Diese erste HTTP-Anbindung gibt keine produktive Laufzeit frei.
Eigene normale Fehlertransaktionen warten weiterhin Rollback gemäß ADR-040 ab.
Bei Abbruch oder Zeitüberschreitung wird die ausschließlich dieser Anfrage gehörende physische Verbindung verworfen.
Sie darf nicht mit einer nur vorgemerkten Rücknahme in einen Pool zurückkehren.
Ein tatsächlicher Negativtest zeigt: Ein geschlossener Clientsocket beendet einen an einer Datenbanksperre wartenden Backendprozess nicht zuverlässig sofort.
Der Adapter setzt deshalb zusätzlich `statement_timeout`, `transaction_timeout` und `idle_in_transaction_session_timeout` beim Verbindungsaufbau.
Alle drei Grenzen stammen ausschließlich aus der ausdrücklich konfigurierten Anfragefrist.
Die Millisekunden werden auf mindestens 1 Millisekunde aufgerundet und müssen in den positiven PostgreSQL-Bereich bis 2.147.483.647 passen.
Nicht darstellbare Fristen werden als ungültige Konfiguration abgewiesen.
Die Anwendung behält ihr gesamtes Anfragezeitlimit. Die serverseitigen Grenzen sichern die anschließende Bereinigung zusätzlich ab.
Der getrennte Datenbankzeuge muss das Verschwinden des wartenden Prozesses weiterhin ohne Freigabe seiner Sperre nachweisen.
Der zusätzliche Verbindungsaufwand ist vor produktiver Nutzung im Lastprofil zu bewerten.

## Anfrage und Antwort

Die Endpunkte und JSON-Formate folgen [service-oauth-v1](service-oauth-v1.md) und dessen OpenAPI-Beschreibung.
Nur POST ist erlaubt; andere Methoden ergeben HTTP 405 mit `Allow: POST`.
URL-Parameter und Browsercookies ergeben `invalid_request`. Es gibt keine Dienstanmeldung über Browsercookies.
Das Inhaltsformat ist genau einmal `application/x-www-form-urlencoded`, optional mit UTF-8-Charset.
Andere Inhalte ergeben eine leere HTTP-415-Antwort. Überschrittene Bodygrenzen ergeben eine leere HTTP-413-Antwort.
Unbekannte Pfade ergeben eine leere HTTP-404-Antwort.
Alle vom Router erzeugten Antworten tragen `Cache-Control: no-store` und `Pragma: no-cache`.
Framingfehler, überschrittene Headergrenzen und Headerzeitlimits verarbeitet Hyper vor dem Router.
Hyper darf dabei die Verbindung schließen oder eine feste Protokollantwort ohne Anwendungsdaten senden.
Für solche Transportantworten wird kein zusätzlicher Routerheader behauptet.

Ein Authorization-Header ist kein erlaubter Ersatz für mTLS.
Ein syntaktisch gültiger HTTP-Authentisierungsschemaname ergibt HTTP 401 / `invalid_client` mit passender `WWW-Authenticate`-Angabe gemäß RFC 6749, Abschnitt 5.2.
Das bestätigt keine Unterstützung dieses alternativen Verfahrens. Der TLS-Handshake verlangt weiterhin Clientzertifikate.
Ein unlesbarer oder mehrfach vorhandener Authorization-Header ergibt `invalid_request`.
Der Schemaname wird als begrenztes HTTP-Token geprüft. Credentialwerte werden nicht zurückgegeben.
Der OpenAPI-Vertrag ergänzt dafür die HTTP-401-Antwort am Tokenendpunkt vor Routercode.

## Tokenausstellung und Introspektion

Der Adapter authentisiert die behauptete Kennung über tatsächlichen SAN, aktiven genauen Fingerabdruck und aktuellen Registrierungsbestand.
Abgewiesene Tokenanmeldung ergibt HTTP 400 / `invalid_client`; fehlende Introspektionsauthentisierung ergibt HTTP 401 / `invalid_client`.
Notwendiger Prüfausfall ergibt eine leere HTTP-503-Antwort.
Introspektionsaufrufer sind ausschließlich die bereits vorgesehenen Zieldienste `identity-access` und `organization-config`.
`gateway` ist gemäß vorhandener Zweckmatrix kein Zieldienst und erhält keine Introspektionsfreigabe.
Das führt keine neue Dienst- oder Fachberechtigung ein.

Vor Ausgabe wird die aktuelle Freigabe für genau Aufrufer, Ziel und Scope gelesen.
Fehlende Freigabe ergibt `invalid_scope`; Lesefehler ergibt HTTP 503.
Diese Vorprüfung ersetzt nicht die vorhandene autoritative Prüfung und guarded Speicherung bei der Ausgabe.
Nach Ausgabe werden notwendige Registrierung und Freigabe erneut geprüft.
Die Antwort verwendet ausschließlich die bestätigten Metadaten und aktuelle Restlaufzeit aus ADR-048.
Ein Fehler nach Ausgabe gibt keinen rohen Token zurück. Ein eventuell bestätigter ungenutzter Hash ist keine Zugriffserlaubnis.

Introspektion authentisiert den Aufrufer vor jeder Antwort, auch bei unlesbarem oder unbekanntem Token.
Ein berechtigt abgefragter ungültiger oder fremder Token ergibt ausschließlich `{"active":false}`.
Aktive Metadaten stammen aus der vorhandenen aktuellen Prüfung. Fehler dürfen nicht als negatives Prüfergebnis maskiert werden.
Nach Warteoperationen prüft der Adapter erneut die tatsächliche Verbindung.
Nach der abschließenden Registrierungsprüfung wird auch die Datenbankzeit erneut gelesen.
Eine inzwischen abgelaufene aktive Momentaufnahme darf keine positive Antwort erzeugen.
Es gibt weder positiven Cache noch rekursive Introspektion.
Anfragen, Header, Roh-Tokens, Schlüssel und Datenbankdiagnosen werden nicht protokolliert.

## Nachweise und Quellen

Eine weitere isolierte Datenbankgruppe verbindet tatsächliche HTTP-Anfragen über mTLS mit den vorhandenen Migrationen 1 bis 9.
Tests prüfen Ausgabe, aktive/inaktive Introspektion, genaue Zertifikatsbindung, Fremdempfänger, aktuellen Widerruf auf derselben Verbindung,
Methoden, Header, Body-/Anfragegrenzen, alternative Anmeldemittel, Ausfall und verworfene Datenbankverbindungen nach Timeout.
Die neue Gruppe läuft vor und nach Datenbankneustart. Synthetische Testadministration ist keine produktive Rollenfreigabe.
Frische Schlüssel und Zertifikate bleiben im Testarbeitsspeicher. Loopback-Ports werden vom Betriebssystem gewählt.
Dies ist kein Browserprofil. Das ausdrücklich bestätigte HTTPS-Browserprofil folgt weiterhin getrennt.

Quellen: [Axum 0.8.9](https://docs.rs/axum/0.8.9/axum/),
[Hyper 1.12.0](https://docs.rs/hyper/1.12.0/hyper/),
[Hyper-Util 0.1.21](https://docs.rs/hyper-util/0.1.21/hyper_util/),
[Tower 0.5.3](https://docs.rs/tower/0.5.3/tower/),
[RFC 6749, Abschnitt 5.2](https://www.rfc-editor.org/rfc/rfc6749#section-5.2).
Serverseitige Grenzen: [PostgreSQL 18, Client Connection Defaults](https://www.postgresql.org/docs/18/runtime-config-client.html).
Abhängigkeiten werden exakt gebunden und durch die bestehenden Sicherheitsprüfungen geprüft.
