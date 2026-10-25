# Dienstvertrag: Begrenzter Mitgliedschaftsverbraucher v1

- Stand: 25.10.2026
- Umfang: KC-006F / #67; identity-access liest organization-config
- Grundlage: membership-http-v1, ADR-019/044/051–054, Spezifikation v5.9

## Transport und Eingaben

Der feste HTTP-Client im Paket service-transport übernimmt genau einen verbundenen TCP-Socket.
Er verwendet den bestehenden tatsächlichen mTLS-Transport mit ausdrücklichen Vertrauensankern und genauem DNS-Zielnamen.
Ein zusätzlich ausdrücklich konfigurierter Serverfingerabdruck wird vor Übertragung des Diensttokens geprüft.
Produktive Verteilung und Rotation dieser Konfiguration bleiben gesondert.
Der Client besitzt keine Datenbankabhängigkeit und startet keinen Listener.

Nur `POST /internal/v1/membership/read` ist möglich.
Die Methode erhält einen binären Diensttoken mit 32 Bytes und zwei positive u128-Kennungen.
Sie erzeugt selbst den einzelnen Bearer-Header und den kanonischen JSON-Body aus membership-http-v1.
Der geheime Header erhält die Markierung als vertraulicher Header.
Es gibt keine beliebigen Pfade, Header, Cookies, URL-Parameter oder Weiterleitungsziele.
Der Token muss aus dem zuständigen Dienstanmeldeablauf stammen und zum tatsächlichen Clientzertifikat gehören.
Der Server prüft weiterhin die aktuelle Registrierung und den Umfang bei jeder Anfrage.

Der Aufrufer konfiguriert positive Grenzen für Parallelität, Anfrage-/Antwortbytes,
Headeranzahl, Headerpuffer sowie Handshake-, Header-, Body- und Gesamtdauer.
Der Headerpuffer beträgt technisch mindestens 8.192 Bytes.
Ein Aufruf besitzt genau eine neue Verbindung. Es gibt keinen Pool und keine automatische Wiederholung.
Die Verbindung wird im selben Future wie die HTTP-Verarbeitung angetrieben.
Erfolg, Fehler, Abbruch oder Fristende verwirft den eigenen Datenstrom und gibt das Budget frei.
Nach Headern und Body werden Zertifikatszeit und gemeinsamer Sperrzustand erneut geprüft.

Antworten benötigen jeweils genau einmal `Cache-Control: no-store` und `Pragma: no-cache`.
Status 200 verlangt `application/json`. Fehler verlangen `application/problem+json`.
Optional ist genau ein UTF-8-Charset zulässig. Doppelte Medientypen werden abgewiesen.
Weiterleitungen, Content-Encoding und Set-Cookie werden abgewiesen. Es gibt keine Dekompression.
Leere, abgeschnittene oder zu große Antworten ergeben keinen positiven Datenstand.
Der rohe Ergebnisbody liegt in einem beim Freigeben überschriebenen Puffer.
Interne HTTP-/TLS-Puffer werden dadurch nicht pauschal als überschrieben behauptet.
Debug- und Fehlerausgaben enthalten keine Tokens, Header, Antwortbytes oder Diagnosen.

## Verbraucher im Identitätsdienst

Der Identitätsdienst erhält ausdrücklich Zieladresse, positive Verbindungsfrist und positive Gesamtdauer.
Ein eigenes ausdrücklich positives Parallelitätsbudget begrenzt bereits den TCP-Verbindungsaufbau.
Die Verbindungsfrist darf die Gesamtdauer nicht überschreiten.
Der äußere Aufruf begrenzt auch den TCP-Verbindungsaufbau. Er gibt nur nach vollständiger Antwortprüfung einen Datenstand zurück.
Ein erfolgreicher HTTP-Transport ist selbst kein Mitgliedschafts- oder Anmeldenachweis.

Der Parser akzeptiert genau ein Objekt mit genau einem erforderlichen Feld `membership`.
`null` bedeutet ausschließlich keinen sichtbaren aktuellen Treffer. Ein fehlendes Feld ist ein Fehler.
Ein vorhandener Treffer muss ein Objekt mit genau den sieben Feldern aus membership-http-v1 sein.
Doppelte Felder einschließlich gleichwertiger JSON-Escapes, unbekannte Felder,
Arraydarstellungen, falsche Typen und Daten hinter dem einzelnen Objekt werden abgewiesen.
Kennungen und Versionen sind positive kanonische Dezimalzeichenketten mit u128-/u64-Grenzen.
Identitäts- und Einrichtungskennung müssen genau zur aktuellen Anfrage passen.
Fristen sind positive ganze Sekunden innerhalb 900/28.800/300 Sekunden.
Inaktivitäts- und zusätzliche Frist dürfen die Gesamtlaufzeit nicht überschreiten.
Es gibt keine Voreinstellungen bei fehlendem oder fehlerhaftem Bestand.

Nur Status 200 kann einen Treffer oder eine Nullantwort liefern.
400/401/404/405/413/415 ergeben eine feste Ablehnung. 429/503 und sonstige Fehler ergeben einen festen Prüfausfall.
Fehlerbody und Diagnosen werden nicht in Fehlermeldungen übernommen.
Jede folgende geschützte Prüfung muss erneut anfragen. Es gibt keinen positiven Berechtigungscache.
Der Datenstand enthält keine MFA-Entscheidung und implementiert allein noch nicht OrganizationAuthority.
Fehlende aktuelle Rechte-/MFA-Daten verweigern weiterhin Zugriff. Es gibt keine dienstübergreifende Atomarität.

## Nachweise und Grenzen

Reine Parsertests prüfen Feldmehrdeutigkeit, Zuordnung, Ganzzahlen, Grenzen und Nullantwort.
Tatsächliche Loopback-/mTLS-/HTTP-Tests prüfen festen Pfad, geheimen Header, frühe Fingerabdruckprüfung,
Header-/Bodygrenzen, falsche Medientypen, Weiterleitungen, Zeitlimits und Budgetfreigabe nach Abbruch.
Die bestehende tatsächliche HTTP-Datenbankgruppe nutzt den neuen Verbraucher vor/nach Neustart.
Sie prüft positive und verborgene/fremde Mitgliedschaft sowie bestätigten Mitgliedschafts- und Dienstentzug.
Die Datenbanken bleiben getrennt. Reine Parserfälle sind keine Dienstidentitätsnachweise.

Neue direkte Abhängigkeiten verwenden ausschließlich bereits gesperrte Versionen: Base64 0.22.1 im Transportpaket
und Serde 1.0.229 im Identitätsdienst. Keine externe Version, Migration, Rolle oder erlaubende Policy wird geändert.
Produktiver Zertifikatsbetrieb, Rollen-/Poolbindung, vollständiger Anmeldeadapter, Gateway und Browser bleiben gesondert.
