# Dienstvertrag: HTTP-Verbindungsbindung v1

- Stand: 22.10.2026
- Umfang: KC-006F / #67, Vorbereitung der HTTP- und Introspektionsanbindung
- Status: Vertrag vor Code; noch kein Router oder öffentlicher Endpunkt

## Herkunft und Lebensdauer

Der spätere Axum-Server verwaltet den verschlüsselten Datenstrom.
Jeder Handler erhält zusätzlich einen Prüfzugang zu genau dieser tatsächlich geprüften mTLS-Verbindung.
Nur `VerifiedConnection` aus dem erfolgreichen Handshake darf `ConnectionPeer` erzeugen.
Es gibt keinen öffentlichen Konstruktor aus Fingerabdruck, Header oder Dienstname.
Der Server bindet den Prüfzugang an die Verbindung, bevor er deren Anfragen verarbeitet.
Er darf ihn nicht aus einem Anfragefeld übernehmen oder zwischen Verbindungen austauschen.

Der Prüfzugang darf für parallele Handler derselben Verbindung geklont werden.
Alle Kopien teilen Zeitbeobachtung, endgültige Fehlersperre und Verbindungsende.
`certificate()` prüft bei jedem Aufruf beide Endzertifikate mit der aktuellen Systemzeit.
Die Zeit wird innerhalb derselben Sperre gelesen, die Beobachtungen serialisiert.
Rückläufige Zeit oder beobachteter Ablauf sperrt alle Kopien endgültig.
Ein nicht lesbarer oder beschädigter Prüfzustand verweigert Zugriff.
Das Freigeben von `VerifiedConnection` sperrt sämtliche noch vorhandenen Prüfzugänge.
Das erfolgreiche lokale Herunterfahren des Datenstroms sperrt sie ebenfalls.
Ein TLS-Eingabefehler, TLS-Ausgabefehler oder beobachtetes Eingabeende sperrt sie ebenfalls.
Eine unbemerkt unterbrochene Gegenstelle wird erst beim nächsten Ein-/Ausgabeversuch erkannt.
Der Prüfzugang garantiert keine Erreichbarkeit der Gegenstelle.

## Reihenfolge geschützter HTTP-Aktionen

1. Der Handler prüft seinen verbindungsgebundenen Prüfzugang.
2. Der Dienst prüft die aktuelle Registrierung des tatsächlichen Zertifikats und den erlaubten Dienstnamen.
3. Der Dienst prüft den aktuellen Tokenzustand und dessen Bindung an Aussteller, Empfänger, Zweck und tatsächliches Zertifikat.
4. Nach wartenden Vorprüfungen prüft der Handler den Prüfzugang erneut unmittelbar vor der geschützten Aktion.
5. Fachaktionen prüfen zusätzlich aktuelle Sitzung, Mitgliedschaft und ihre Objekt-/Feldrechte.

Zertifikatsmetadaten und eine frühere positive Prüfung ersetzen keinen dieser Schritte.
Eine zurückgegebene Metadatenreferenz ist eine Momentaufnahme und kein Berechtigungsschein.
Die Bibliothek führt keine Registrierung, Tokenprüfung oder fachliche Aktion aus.
Die bestehende Methode `peer_certificate()` bleibt erhalten und nutzt denselben Prüfzustand.

## Anschluss an OAuth

Tokenausstellung und Introspektion authentisieren Dienste über die tatsächliche mTLS-Verbindung.
Die Introspektion löst keine weitere Tokenprüfung aus.
Die spätere HTTP-Implementierung muss `client_id` an die geprüfte Registrierung binden.
Sie darf die behauptete Kennung nicht selbst als Authentisierung behandeln.
Nur ein berechtigter Introspektionsaufrufer erhält Tokeninformationen.
Unbekannte, abgelaufene oder für ihn nicht bestimmte Tokens liefern ausschließlich `{"active":false}`.
Die aktive Antwort bindet das ursprüngliche Clientzertifikat über `cnf` und `x5t#S256`.
Der empfangende Dienst vergleicht diese Bindung mit dem tatsächlichen Zertifikat seines Aufrufers.
Er darf positive Antworten nicht als dauerhafte Zugriffserlaubnis speichern.

Grundlagen: [RFC 8705, Abschnitt 2](https://www.rfc-editor.org/rfc/rfc8705#section-2),
[Abschnitt 3.2](https://www.rfc-editor.org/rfc/rfc8705#section-3.2),
[RFC 7662, Abschnitt 2](https://www.rfc-editor.org/rfc/rfc7662#section-2).
Die bestätigte OAuth-Fehlerformatausnahme gilt weiterhin ausschließlich für OAuth-Protokollendpunkte.
Pfad, OpenAPI-Schema, Formularprüfung, Fehlerzuordnung und Anfragegrenzen folgen vor dem jeweiligen Routercode.
Dieser Vertrag ist noch kein vollständiger OAuth-Endpunktvertrag.

## Nachweise und Grenzen

Tests verwenden tatsächliche mTLS-Verbindungen, freie Loopback-Ports und ausschließlich flüchtige Testzertifikate.
Sie prüfen identische Herkunft zwischen Datenstrom und Handler-Prüfzugang, unterschiedliche Verbindungen,
gemeinsame Ablauf-/Fehlersperre, Verbindungsende und parallele Prüfzugänge.
Eine getrennte Zustandsprüfung weist rückläufige Zeit und die endgültige Sperre nach.
Sie wird nicht als tatsächlicher Systemuhrsprung ausgegeben.

Keine neue Migration, Datenbankberechtigung, Abhängigkeit oder fachliche Zugriffsregel entsteht.
HTTP-Router-, Registrierungs-, Introspektions-, Organisations- und Browsernachweise bleiben offen.
Das noch unbestätigte zusätzliche HTTPS-Browserprofil bleibt gesperrt.
