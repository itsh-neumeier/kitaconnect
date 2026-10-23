# Sitzungsrotation v1 – interner Vertrag

Stand: 21.10.2026. KC-006F / #67. Grundlage: ADR-019/034/035/036/037/038.
Dieser Schritt tauscht den geheimen Sitzungsnachweis einer bestehenden Anmeldung atomar aus.
Er erteilt keine neue Authentisierungsstärke und verlängert keine Sitzungsfrist.

## Vorbedingungen und Wirkung

Die Operation erhält einen aktuellen geheimen Sitzungsnachweis, die gewünschte Einrichtung und eine neue interne Sitzungsdatensatzkennung.
Sie verlangt dieselbe vollständige aktuelle Prüfung wie session_check: lokale Bindung, Wiederherstellung, Organisation, MFA und wirksame Fristen.
Sie verwendet dieselbe Identitätssperre und eine eigene READ-COMMITTED-Transaktion außerhalb aufrufender Transaktionen.
Eine alte positive CurrentSession-Momentaufnahme wird nicht als Rotationserlaubnis angenommen.

Nach erfolgreicher Prüfung erzeugt sie 32 frische zufällige Bytes aus der bestehenden Betriebssystemquelle.
Nur die Ableitung wird gespeichert. Der Nachweis stammt niemals vom Client.
Alter Widerruf, neuer Datensatz, unveränderte Anmeldebindung, gekürzte Grenzen und Nachfolgerverknüpfung werden gemeinsam bestätigt.
Der rohe neue Nachweis wird erst nach erfolgreichem Commit ausgegeben.

Erstellzeit, absolutes Ende, ursprüngliche Inaktivitätsgrenze, letzte berechtigte Aktivität,
Authentisierungszeit, MFA-Stärke und bereits beobachtete Verschärfungen bleiben erhalten.
Rotation zählt nicht als Aktivität oder erneute Faktorprüfung.
Eine neue Policy-Version darf Grenzen weiter kürzen; eine Lockerung verlängert sie nicht.
Ein zweiter konkurrierender Rotationsversuch mit demselben alten Nachweis darf keine weitere Sitzung erzeugen.
Fehlerpfade warten den vollständigen Rollback vor der Rückgabe ab.
Eine danach offen gehaltene Verbindung darf keine Identitätssperre behalten und den Gewinner blockieren.
Abbruch, Fehler, unterdrückte Änderung oder fehlgeschlagener Commit geben keinen neuen Nachweis aus.

## Anmeldelinie und Abmeldung

Neue Migration 0007 ergänzt eine unveränderliche Nachfolgerverknüpfung zwischen zwei Sitzungsdatensätzen.
Jeder Vorgänger hat höchstens einen Nachfolger; ein Datensatz kann nicht sein eigener Nachfolger sein.
Die einmal verbrauchte Faktorprüfung bleibt der Ursprung derselben Anmeldung.
Die bisher eindeutige Vorgangsreferenz der Sitzungsbindung darf deshalb für geprüfte Rotationsnachfolger erneut vorkommen.
Dies macht den Anmeldevorgang nicht erneut verbrauchbar. Der bestehende Verbrauchs- und Sperrvertrag bleibt unverändert.

Ein Schutztrigger prüft Identität, Einrichtungs-/Anmeldebindung, Authentisierungszeit und unveränderte Ausgangsgrenzen.
Die kopierte Beobachtung darf keine zuvor gekürzte Grenze erweitern oder MFA-Pflicht entfernen.
Der Vorgänger muss widerrufen und der Nachfolger aktiv sein.
Alle neuen Tabellen bleiben durch FORCE RLS ohne erlaubende Runtime-Policy geschlossen.
Alte Migrationen bleiben unverändert; die Änderung an der bisherigen Bindungs-Eindeutigkeit erfolgt ausschließlich in Migration 0007.

Abmeldung mit einem bekannten Nachweis widerruft die lokalen Nachfolger derselben Anmeldung ebenfalls.
Damit darf eine gleichzeitig gewinnende Rotation die Abmeldung nicht umgehen.
Andere Anmeldungen derselben Identität bleiben von dieser Abmeldung unberührt.
Der lokale Aufruf benötigt weiterhin keine verfügbare Organisation. Er prüft betroffene Zeilen und bestätigt erst nach Commit.
Unbekannte Nachweise bleiben idempotent. Das spätere Gateway muss Origin/CSRF vor dem Aufruf prüfen und Nachweiswechsel an CSRF binden.

## Frist während Schreibwartezeit

Nach möglicherweise wartenden Schreibvorgängen wird die Datenbankzeit vor Commit erneut geprüft.
Ein inzwischen abgelaufener Vorgang darf keinen Rotationsnachweis ausgeben.
Ein interner Savepoint begrenzt die Kandidatenanlage.
Bei beobachtetem Ablauf werden Kandidat und alter Widerruf zurückgenommen; die wirksamen Beobachtungen bleiben erhalten.
Die ursprüngliche Sitzung wird dann dauerhaft als expired bestätigt. Andere Fehler nehmen die gesamte Rotation zurück.
Ein Commitfehler erzeugt unabhängig von seiner Ursache keinen positiven Rückgabewert.

## Nachweise und Grenzen

Echte PostgreSQL-Tests prüfen richtigen Nachfolger, alte Nachweisverweigerung, unveränderte Grenzen und Authentisierungszeit,
zwei tatsächlich wartende Rotationsversuche, Abmeldung gegen Rotation, getrennte Anmeldungen,
Ausfall, Wiederherstellung, fehlerhafte Kandidatenanlage, Commitfehler und Ablauf während Schreibwartezeit.
Migration, Rücknahme, Schutztrigger, RLS und Neustart erhalten eigene Nachweise.
Der bestehende Sitzungstest gegen Migration 0006 muss ohne Rotationsbestand weiterlaufen.
Gateway, Browser-/TLS-/mTLS-Nachweise und zweckgebundene erneute Faktorprüfung bleiben gesondert.
