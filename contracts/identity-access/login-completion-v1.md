# Interner Vertrag: Einmaliger Anmeldeabschluss v1

Stand: 21.10.2026. KC-006F / #67. Grundlage: v5.9, ADR-019/034/035/036.
Dieser Schritt implementiert den lokalen Übergang von geprüften Faktoren zu einer neuen Sitzung.
Gateway, Cookies, Sitzungsrotation und die laufende Prüfung geschützter Browseraktionen folgen getrennt.

## Vorgang und Vertrauensgrenze

Ein Anmeldevorgang gehört zu genau einer Identität, Einrichtung und Route: Passwort oder Passkey.
Seine Referenz enthält 32 zufällige Bytes. Nur deren Ableitung wird gespeichert. Die Referenz allein beweist keine Anmeldung.
Lebensdauer und Mengenlimits werden ausdrücklich übergeben. Die Lebensdauer ist auf höchstens 300 Sekunden begrenzt.
Es gibt ein globales und ein identitätsbezogenes Limit offener Vorgänge. Abgelaufene unverbrauchte Vorgänge können entfernt werden.
Alle Aufrufer müssen dieselbe explizite Grenzkonfiguration verwenden. Ein einmal angelegter Grenzsatz wird nicht durch einen Anmeldeaufruf überschrieben; abweichende Werte werden abgewiesen.
Verbrauchte, einer Sitzung zugeordnete Vorgänge bleiben für deren Bindung erhalten; die spätere Aufbewahrung benötigt einen eigenen Betriebsvertrag.

Ein vertrauenswürdiger Organisationsadapter liefert aktuelle Mitgliedschaft, Einrichtungseinstellungen und MFA-Anforderung.
Er muss Antwort und Anfrage eindeutig an Identität und Einrichtung binden. Fehlende oder fehlerhafte Antworten verweigern den Vorgang.
Die Rust-Schnittstelle dafür ist eine interne Integrationsgrenze und kein vom Browser übermittelbarer Freigabewert.
In diesem Schritt verwenden ausschließlich Tests ein ausdrücklich synthetisches Testdoppel.
Ein späterer Netzwerkadapter muss ADR-019 einschließlich mTLS und aktueller Rechteprüfung erfüllen.
Während einer gehaltenen Identitätssperre darf der Adapter nicht in einen schreibenden Identitätsaufruf zurückrufen.
Transportfristen und Ausfallbehandlung müssen bei seiner späteren Anbindung geprüft werden.

## Gemeinsame lokale Sperre

Passwortpflege, abschließende Passwortprüfung, Passkey-Abschlüsse und -Widerruf sowie TOTP-Pflege und -Prüfung verwenden dieselbe transaktionsgebundene Identitätssperre.
Die Sperre verwendet PostgreSQL-Advisory-Locks mit einem stabilen, aus Identität und festem Namensraum abgeleiteten 64-Bit-Schlüssel.
Eine Hashkollision kann nur zusätzliche Serialisierung verursachen; sie erteilt keine Rechte.
Die Schlüsselableitung verwendet SHA-256 und ist ausschließlich eine Sperrzuordnung, kein Anmelde- oder Kryptoprotokoll.
Teure Passwortarbeit bleibt vor der abschließenden Sperre. Bestandsprüfung und positiver Abschluss liegen hinter der Sperre.
Die Reihenfolge ist Identitätssperre, dienstinterner Faktorbestand, Anmeldevorgang, Sitzungsbestand und Bindung.
Direkte Credential-Schreibvorgänge ohne diesen gemeinsamen Adaptervertrag sind unzulässig.

Die Hilfsoperation verwendet vorhandene Identitätsobjekte. Alte Tests mit früheren Migrationen benötigen keine neuen Anmeldetabellen.
Die neuen vorgangsgebundenen Operationen verlangen dagegen die neue Migration 0005 und verweigern bei fehlendem Bestand.
Die bisherigen positiven Ergebnistypen bleiben lokale Momentaufnahmen; sie erhalten keine Umwandlung in einen Vorgang.

## Atomare Faktorübergabe

Neue vorgangsgebundene Varianten der bestehenden Bibliotheksprüfungen merken ihren Erfolg innerhalb derselben Faktortransaktion vor.
Passwort prüft weiterhin den tatsächlich gehashten PHC-Bestand und dessen Version. Eine erfolgreiche Hashaktualisierung wird vor der Vormerkung berücksichtigt.
TOTP verbraucht den Zeitschritt und merkt den zugeordneten zweiten Faktor atomar vor. Passwort muss zuvor in demselben Vorgang geprüft sein.
Passkey verlangt die bestätigte Benutzerverifikation und merkt den Erfolg nach der notwendigen Credential-Aktualisierung vor.
Die Passkey-Challenge bleibt bereits vor der Bibliotheksprüfung dauerhaft verbraucht.
Ein fehlgeschlagener späterer Vorgang gibt weder reservierte Prüfversuche noch verbrauchte Passkey-Challenges zurück.

Die Vormerkung bindet eine Ableitung des aktuellen Faktorbestands: Passwortversion und PHC, Passkey-Version und begrenzter Credential-Bestand,
aktiver verschlüsselter TOTP-Faktor sowie Wiederherstellungssperre. Geheimnisse werden nicht zusätzlich im Vorgang gespeichert.
Der TOTP-Schrittzähler ist ausgenommen; dessen separate dauerhafte Einmalprüfung bleibt bestehen.
Weitere Faktoren und Abschluss verlangen denselben Bestand. Änderungen führen zur Verweigerung und einem neuen Anmeldevorgang.
Auch eine unabhängige Passkey-Prüfung kann durch deren Versionsfortschritt einen offenen Vorgang konservativ entwerten.
Die erste Vormerkzeit wird nicht durch einen weiteren Faktor verjüngt.
Ein als abgelaufen erkannter Vorgang wird dauerhaft als ungültig markiert. Ein späterer Zeitrücksprung darf ihn nicht reaktivieren.
Erkennt die tatsächliche Faktorprüfung den Ablauf erst nach erfolgreicher Verifikation, bestätigt sie ausschließlich die Faktorzustände und diese Sperre; sie merkt keinen Anmeldeerfolg vor und gibt einen Fehler zurück.
Dies kann einen bereits richtig geprüften TOTP-Schritt verbrauchen oder eine gültige Passwort-Hashaktualisierung bewahren. Es erteilt keine Sitzung und gibt keine neue Nachweisreferenz aus.
Eine Wiederherstellung erzeugt keinen dieser Erfolgsmarker und keine normale Sitzung.

## Lokaler Sitzungsabschluss

1. Sperren und prüfen Sie Identität und Vorgang. Prüfen Sie Zweck, Route, Einrichtung, Frist, Verbrauch und monotone Datenbankzeit.
2. Prüfen Sie den aktuellen Faktorbestand gegen die Vormerkung. Eine aktuelle Wiederherstellungssperre verweigert den Abschluss.
3. Fragen Sie den Organisationsadapter erneut ab. Prüfen Sie Identität, Einrichtung, Policy-Version und bestätigte Wertebereiche.
4. Kombinieren Sie ursprüngliche und aktuelle Fristen durch das jeweilige Minimum. Eine rückläufige Version oder veränderte Werte derselben Version werden abgewiesen.
5. Verlangen Sie MFA, wenn sie ursprünglich oder aktuell erforderlich ist oder ein aktiver TOTP-Faktor eingerichtet ist. Passkey mit Benutzerverifikation oder Passwort plus TOTP erfüllen die bestätigte Regel.
6. Prüfen Sie nach der Organisationsantwort die Datenbankzeit erneut. Ein während Wartezeit abgelaufener Vorgang darf nicht erfolgreich werden.
7. Verbrauchen Sie den Vorgang, erzeugen Sie einen neuen zufälligen Sitzungsnachweis und speichern Sie Sitzung sowie Einrichtungs-/Policy-Bindung in einer lokalen Transaktion.
8. Geben Sie den rohen Sitzungsnachweis erst nach erfolgreichem Commit aus. Zwei konkurrierende Abschlüsse dürfen höchstens eine Sitzung erzeugen.

Die Sitzung speichert die gekürzten ursprünglichen Fristen unveränderlich. Ihre gesonderte Bindung speichert Einrichtung, Policy-Version und Authentisierungsstärke.
Die nächste Integrationsstufe muss diese Daten für aktuelle Frist-, Mitgliedschafts- und Zweckprüfungen verwenden.
Passkey-Entzug und Passwortpflege widerrufen vorhandene Sitzungen atomar. Unterdrückte Widerrufe führen zum Fehler.

Eine lokale Transaktion schließt die fremde Mitgliedschaft nicht atomar ein. Jede spätere geschützte Aktion muss erneut aktuell prüfen.
Ein altes positives Organisations-Testdoppel ist kein produktiver Berechtigungsnachweis.
Weder eine normale Sitzung noch eine Mitgliedschaft erteilt allein Objekt- oder Feldrechte aus KC-008.

## Datenbank und Nachweise

Migration 0005 ergänzt Anmeldevorgänge und deren Sitzungsbindung. Alte Migrationen bleiben unverändert.
Neue Tabellen erhalten FORCE RLS ohne erlaubende Laufzeit-Policy. Ein Trigger schützt unveränderliche Bindungen, monotone Zeit und endgültigen Verbrauch.
Der Adapter verwendet vorhandene geprüfte Verbindungen außerhalb aufrufender Transaktionen und eigene READ-COMMITTED-Transaktionen.
Rohe äußere SQL-Transaktionen sind unzulässig. Fehler und Debugausgaben enthalten keine Nachweise oder Datenbankdetails.

Tests verwenden nur synthetische Identitäten, RAM-Testschlüssel und bestehende Bibliotheks-Authentikatoren.
Nachzuweisen sind richtige und falsche Route/Identität, unvollständige Faktoren, geänderter Bestand, Einmaligkeit mit tatsächlichen Sperrwartenden,
Wiederherstellungssperre, aktuelle Mitgliedschaftsverweigerung, Policy-Verschärfung/-Lockerung, Ablauf während Organisationswartezeit,
Mengenlimits, äußere Transaktionen, unterdrückter Eintrag, Commitfehler, Migration, RLS und Neustart.
Die bisherigen Passwort-, WebAuthn- und TOTP-Datenbanktests laufen weiter gegen ihre getrennten frühen Versionsstände.
