# Aktuelle zusammengesetzte Anmeldegrundlage v1

Stand: 25.10.2026. Begrenzter Integrationsschritt von KC-006F / #67 auf KC-008A.

## Ablauf und Ergebnis

Der Identitätsdienst verbindet die vorhandene tatsächliche Dienstanmeldung, aktuelle Mitgliedschaft und eigene Rollen-/MFA-Quelle.
Jede Prüfung fordert über den bestehenden mTLS-/OAuth-Client einen neuen Diensttoken an.
Die feste Anfrage lautet `client_id=identity-access&grant_type=client_credentials&scope=membership%3Aread`.
Zieladresse, DNS-Name, Zertifikate und erwarteter Serverfingerabdruck stammen aus ausdrücklicher vertrauenswürdiger Konfiguration.
Die Antwort muss den erwarteten tatsächlichen Serverfingerabdruck besitzen und den vorhandenen strengen Tokenparser bestehen.
Der Token darf nur für die feste interne Mitgliedschaftsabfrage verwendet werden. Es gibt keinen vom Benutzer übergebenen Diensttoken.
Die vorhandene Organisationsprüfung prüft dessen aktuelle Registrierung, Empfänger, Zweck, Zertifikatsbindung und Zeit.
Danach liest die eigene Rollenquelle den aktuellen eingestuften Bestand für dieselbe Identität und Einrichtung.
Fehlende Mitgliedschaft, fehlender Rollenbestand, unbekannte Einstufung, Entzug oder Prüfausfall liefern ausschließlich einen Fehler.
Kein fehlender Bestand wird durch einen falschen MFA-Wert ersetzt.

Das erfolgreiche interne Ergebnis enthält genaue Kennungen, Fristen, abgeleitete MFA-Anforderung und drei getrennte Versionen:
Mitgliedschaft, Einrichtungseinstellungen und Rollenbestand. Die Versionen werden weder addiert noch zu einem Hash zusammengefasst.
Der Ergebnisdatentyp hat keinen öffentlichen Konstruktor und verdeckt seine Inhalte in Debug-Ausgaben.
Er erteilt keine Sitzung oder Fachberechtigung. Er enthält keine Umwandlung in den bisherigen einversionigen `OrganizationContext`.
Die nächste Integration muss die drei Versionen getrennt dauerhaft binden und prüfen, bevor sie den Anmeldeport ersetzt.
Dieser Schritt behauptet deshalb weder vollständigen Anmeldeabschluss noch Gateway- oder Browsernachweise.

## Fristen und Mengen

Die Parallelitätsgrenze, Verbindungsfrist zur Tokenausgabe, maximale Tokenantwort und Gesamtfrist werden ausdrücklich konfiguriert.
Alle Grenzen sind positiv und darstellbar. Die Verbindungsfrist darf die Gesamtfrist nicht überschreiten.
Das gemeinsame Budget gilt vor Verbindungsaufbau. Die Gesamtfrist umfasst Tokenausgabe, Mitgliedschaft und eigenen Rollenleser.
Die vorhandenen Quellen behalten ihre eigenen Transport-, Verbindungs-, Datenbank- und Mengenbudgets.
Die Startzeit wird vor der Tokenverbindung mit der monotonen Uhr erfasst.
Die gesamte bis zum Ergebnis verstrichene Zeit muss kleiner als `expires_in` der geprüften Antwort sein.
Dies ist eine konservative zusätzliche Frist, keine positive Tokenablage oder neue Berechtigung.
Es gibt keinen positiven Cache, keinen Wiederholungsversuch, keine Ausweichquelle und keinen abgelösten Hintergrundauftrag.
Abbruch gibt das äußere Budget frei. Eigene Verbindungen gehören den vorhandenen begrenzten Quellen.
Rohe und kodierte Token bleiben in löschenden Speicherbehältern und erscheinen nicht in Fehlern oder Debug-Ausgaben.

## Aktualität und Grenzen

Jede Quelle liefert ihren eigenen aktuellen Snapshot. Es gibt keine dienstübergreifende atomare Zusage.
Ein späterer Entzug muss die nächste aktuelle Prüfung verweigern.
Der eigene Rollenleser nimmt keine fremde Transaktion oder zusätzliche Identitätssperre an.
Die Zusammensetzung schreibt weder Identitäts- noch Organisationsbestand und ruft keine schreibende Identitätsoperation zurück.
Die spätere atomare Anmeldeeinbindung muss lokale Sperren, getrennte Versionsbeobachtung und Fehlertransaktionen gesondert nachweisen.
Es entstehen keine neue Anwendungstabelle, Standardrolle, öffentliche Rollenvergabe oder erlaubende Laufzeitpolicy.
Die vorhandene Identitätsmigration 0010 wird zusätzlich im isolierten tatsächlichen HTTP-Testbestand angewendet.
Nur dieser Testbestand verwendet privilegierte positive synthetische Daten. Die Laufzeitprüfung bleibt geschlossen.

## Nachweise

Reine Tests prüfen Konfigurationsgrenzen, die konservative Tokenfrist und verdeckte Diagnosen.
Die tatsächliche HTTP-/Datenbankgruppe verbindet neue Tokenausgabe, Organisationsrouter und getrennte Datenbestände.
Sie prüft genaue Bindung, drei getrennte Versionen, beide MFA-Ergebnisse, Rollenänderung, Entzug, fehlende Mitgliedschaft,
falschen Ausstellerfingerabdruck, geschlossene Rollen-RLS und Ausfall vor/nach PostgreSQL-Neustart.
Die bestehenden tatsächlichen Transport- und Abbruchnachweise bleiben erhalten.
Die ursprünglichen Akzeptanzkriterien von KC-006F und KC-008 bleiben offen.
