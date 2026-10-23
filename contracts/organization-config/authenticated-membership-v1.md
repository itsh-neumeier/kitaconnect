# Dienstvertrag: Authentisierte Mitgliedschaftsabfrage v1

- Stand: 23.10.2026
- Umfang: KC-006F / #67, interner Organisationsadapter vor öffentlichem Router
- Grundlage: ADR-019/041/044–052 und membership-read-v1

## Eingaben und Datenhoheit

Der Adapter erhält den Prüfzugang einer tatsächlichen offenen mTLS-Verbindung,
einen binären Diensttoken mit 32 Bytes, eine Identitätsreferenz und eine genaue Einrichtung.
Die Identitätsreferenz muss aus dem zuständigen geprüften Anmeldeablauf stammen.
Ein erfolgreicher Datenabruf beweist selbst keine Benutzeranmeldung.
Der Adapter liest ausschließlich eigene Organisationstabellen. Er importiert keinen Identitätsdienst.
Er öffnet keinen Listener und implementiert zunächst keinen öffentlichen HTTP-Endpunkt.

## Dienstprüfung und Ablauf

Nur der Dienst identity-access darf mit dem bestehenden Umfang `membership:read` anfragen.
Der tatsächliche DNS-SAN des Aufrufers muss mit dem ausdrücklich konfigurierten Namen übereinstimmen.
Die aktuelle Registrierung des genauen Aufruferfingerabdrucks und die Freigabe prüft der Identitätsdienst bei jeder Introspektion.
Der Adapter sendet über den vorhandenen begrenzten tatsächlichen OAuth-Client eine neue Introspektion als organization-config.
Das Formular enthält ausschließlich feste Clientkennung und kanonisch dargestellten Token.
Es gibt keinen positiven Zwischenspeicher und keine Wiederholung bei Ausfall.

Konfiguration enthält ausdrücklich Zieladresse, Aussteller, erwarteten Serverfingerabdruck und positive Grenzen.
Der Client prüft zusätzlich die bestehende Zertifikatskette und den konfigurierten DNS-Zielnamen.
Der beobachtete Serverfingerabdruck muss zur ausdrücklichen Konfiguration passen.
Dieser Konfigurationsvergleich ist kein Nachweis eines produktiven Verteilungs- oder Widerrufsverfahrens.
Produktive Zertifikatsverteilung, Rotation und Rollen-/Poolbindung bleiben gesondert.
Es wird keine gemeinsame Nutzung eines Server- und Clientschlüssels verlangt.

Nach dem HTTP-Aufruf öffnet der Adapter eine eigene begrenzte Organisationsverbindung.
Deren frische Datenbankzeit bestimmt die Antwortprüfung nach service-oauth-consumer-v1.
Aussteller, identity-access als Aufrufer, organization-config als Ziel, Umfang und tatsächlicher Aufruferfingerabdruck müssen genau passen.
Eine inaktive Antwort verweigert die Abfrage. Infrastruktur- oder Protokollausfall ist ein fester Prüffehler.
Erst danach liest der Adapter die aktuelle Mitgliedschaft und Fristen mit der bestehenden lokalen Einzelabfrage.
Er prüft nach Wartezeiten erneut den Verbindungszugang und die Tokenzeit mit neuer Datenbankzeit.
Ein beobachteter Zeitrücksprung innerhalb des Aufrufs verweigert das Ergebnis.

## Grenzen, Abbruch und Ergebnis

Gesamtdauer, TCP-Verbindungsaufbau, Datenbankoperationen, Antwortbytes und Parallelität sind ausdrücklich positiv begrenzt.
Eigene Datenbankverbindungen erhalten die bereits geprüften Anweisungs-, Transaktions- und Leerlauffristen bei Verbindungsbeginn.
Abbruch verwirft eigene Verbindungen und gibt das Budget frei. Es entsteht kein losgelöster Auftrag.
Eine bereits vom HTTP-Handler geteilte eingehende Verbindung bleibt dessen Eigentum.
Der Handler muss deren Lebenszyklus weiter beobachten; der Prüfzugang meldet beobachteten Abschluss oder Fehler.

Das Ergebnis ist ausschließlich ein aktueller Mitgliedschafts-/Einstellungsdatenstand oder kein sichtbarer Treffer.
Eine spätere geschützte Abfrage muss die gesamte Prüfung neu ausführen.
Bestätigter Entzug vor der nächsten Abfrage muss wirken. Zwischen getrennten Diensten besteht keine gemeinsame Transaktion.
Ein nach der jeweiligen Zustandsabfrage konkurrierender Entzug kann deren bereits gelesenen Datenstand nicht rückwirkend ändern.
Der Adapter legt keine Mitgliedschaft an und ändert keine Datenbankrolle oder Policy.
Die Laufzeit-RLS bleibt geschlossen. Positive Daten entstehen ausschließlich im gekennzeichneten isolierten Testbetrieb.

Dieser Adapter ersetzt noch nicht OrganizationAuthority des Identitätsdienstes.
Fehlende Rechte-/MFA-Entscheidungen dürfen nicht durch einen pauschalen falschen MFA-Wert ersetzt werden.
Gateway, Browser, Objekt-/Feldrechte und ursprüngliche Gesamtabnahme bleiben offen.

## Nachweise und Abhängigkeiten

Die vorhandene tatsächliche HTTP-Datenbankgruppe verbindet eingehendes mTLS,
OAuth-Introspektion und eine getrennte Organisationsdatenbank vor/nach Neustart.
Sie prüft exakte Zuordnung, Mitgliedschafts- und Dienstentzug, falsche Zertifikatsbindung,
falschen Serverfingerabdruck, geschlossenen Prüfzugang, Datenbankausfall und geschlossene Laufzeit-RLS.
Anweisungsfristen werden zusätzlich durch einen unabhängigen Zeugen bei gehaltener Datenbanksperre geprüft.
Grenzen und Budgetfreigabe erhalten gezielte Tests.
Neue direkte Abhängigkeiten verwenden ausschließlich vorhandene gesperrte Paketversionen.
Es entsteht keine Migration, erlaubende Policy oder produktive Konfiguration.
