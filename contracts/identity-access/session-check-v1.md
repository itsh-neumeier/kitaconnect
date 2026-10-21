# Aktuelle Sitzungsprüfung v1 – interner Vertrag

Stand: 21.10.2026. KC-006F / #67. Grundlage: ADR-019/034/035/036/037.
Dieser Teil ergänzt aktuelle lokale Sitzungsprüfung und serverseitige Abmeldung.
Rotation, erneute Faktorprüfung, Gateway und der echte Organisationsadapter folgen getrennt.

## Prüfung und Grenzen

Die Operation erhält eine bestehende geprüfte Verbindung, einen geheimen Sitzungsnachweis,
die gewünschte Einrichtung und eine ausdrücklich erforderliche Authentisierungsstärke.
Zulässig sind normale Authentisierung, MFA sowie kürzlich geprüfte MFA.
Diese Auswahl stammt vom vertrauenswürdigen aufrufenden Dienst, niemals aus einer ungeprüften Browserangabe.
Objekt- und Feldrechte bleiben KC-008. Eine Mitgliedschaft allein erteilt keine Fachberechtigung.

Eine gültige Sitzung benötigt die gespeicherte Bindung aus dem einmaligen Anmeldeabschluss.
Alte allein über create_session angelegte Sitzungen besitzen diese Bindung nicht und werden abgewiesen.
Die Prüfung sperrt zuerst die Identität und danach den aktuellen lokalen Sitzungsbestand.
Sie liest die Wiederherstellungssperre und fragt den Organisationsadapter für jede Prüfung neu ab.
Eine fehlende, falsch gebundene oder fehlerhafte Antwort verweigert Zugriff. Positive Antworten werden nicht zwischengespeichert.
Das vorhandene OrganizationAuthority bleibt eine vertrauenswürdige interne Integrationsgrenze.
Ein synthetisches Testdoppel belegt keinen echten Mitgliedschafts-, Transport- oder mTLS-Nachweis.
Eine lokale Transaktion schließt den Organisationsbestand nicht atomar ein; die nächste geschützte Aktion muss erneut prüfen.

## Dauerhafte Fristverkürzung

Neue Migration 0006 ergänzt eine Beobachtung je gebundener Sitzung.
Sie speichert die zuletzt geprüfte Policy-Version, deren unveränderte Rohwerte und die bisher strengsten Sitzungsgrenzen.
Die erste Beobachtung benötigt ebenfalls eine aktuelle Organisationsantwort.
Sie darf keine Voreinstellungen aus fehlenden Daten auswählen.
Die Versionsmarke der Antwort darf nicht unter der bei Anmeldung gespeicherten Version liegen.
Nach einer Beobachtung führen rückläufige Versionen oder andere Rohwerte derselben Version zum Fehler.

Bereits bei Anmeldung gekürzte Werte sind Obergrenzen. Sie sind nicht mit den Rohwerten der damals aktuellen Policy gleichzusetzen.
Inaktivitätsfrist, absolute Laufzeit und Gültigkeit zusätzlicher Authentisierung werden jeweils durch das Minimum verbunden.
Eine später gelockerte Policy verlängert die zuvor gekürzten Grenzen nicht.
Einmal beobachtete MFA-Pflicht bleibt für diese Sitzung erforderlich.
Die erste Authentisierungszeit wird durch Prüfung oder Aktivität nicht verjüngt.
Kürzlich geprüfte MFA verlangt MFA und ein Alter kleiner als die gekürzte zusätzliche Frist.

Die Datenbankzeit wird nach dem Erwerb der Sperren und nach der Organisationswartezeit gelesen.
Rückläufige Zeit, Zeitüberlauf und ungültige Bestandswerte verweigern Zugriff.
Beobachteter Ablauf wird vor der Fehlerausgabe dauerhaft als expired bestätigt.
Eine später gelockerte Policy oder ein Zeitrücksprung kann diese Sitzung nicht reaktivieren.
Beobachtungen werden auch bei fehlender MFA oder zu alter zusätzlicher Authentisierung bestätigt.
Dabei wird keine Aktivität aufgezeichnet und kein positiver Kontext ausgegeben.

## Aktivität und Abmeldung

Die normale Prüfung verlängert keine Inaktivitätsfrist.
Eine gesonderte Aktivitätsoperation prüft denselben aktuellen Bestand vollständig erneut.
Sie ist ausschließlich für aktuell berechtigte Aktivität vorgesehen; der Aufrufer muss zusätzlich seine Fachberechtigung prüfen.
Beliebige Browserabfragen dürfen die Aktivitätsoperation nicht auswählen.
Erfolg gibt erst nach Commit eine begrenzte Momentaufnahme aus; sie ist kein wiederverwendbarer Berechtigungsnachweis.

Abmeldung benötigt den geheimen Sitzungsnachweis und widerruft dessen lokale Anmeldung einschließlich vorhandener Rotationsnachfolger.
Andere Anmeldungen derselben Identität bleiben unverändert; der Rotationsvertrag konkretisiert die Nachfolgerbindung.
Sie bleibt bei Organisationsausfall und Wiederherstellung möglich. Ein unbekannter Nachweis wird idempotent behandelt.
Sie konkurriert über dieselbe Identitäts- und Sitzungssperre. Unterdrückter Widerruf führt zum Fehler.
Das spätere Gateway muss vor Abmeldung CSRF und Origin prüfen sowie das Cookie passend entfernen.

Alle mehrteiligen Operationen verlangen Autocommit als Aufrufzustand und verwenden eigene READ-COMMITTED-Transaktionen.
Neue Tabellen erhalten FORCE RLS ohne erlaubende Runtime-Policy.
Alte Migrationen, Rollenentscheidungen und externe Abhängigkeiten bleiben unverändert.

## Nachweise

Echte PostgreSQL-Tests prüfen richtige und fremde Einrichtung, fehlende Bindung, aktuellen Organisationsausfall,
Mitgliedschaftsentzug über das Testdoppel, Wiederherstellung, MFA-Stärke und zusätzliche Frist.
Sie prüfen dauerhafte Verschärfung und spätere Lockerung, Versionsrücklauf und Widerspruch derselben Version,
Ablauf während externer Wartezeit, rückläufige Zeit, reale Sperrkonkurrenz mit Abmeldung,
unterdrückte Beobachtung und Widerruf, Commitfehler, äußere Transaktion, RLS und Neustart.
Der vorhandene echte Login-Test bleibt der gesonderte Nachweis der Faktorprüfung.
Synthetisch angelegte Sitzungszustände in diesen Tests behaupten keine erneute Faktorprüfung.
