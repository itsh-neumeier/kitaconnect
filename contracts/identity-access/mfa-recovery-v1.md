# Interner Vertrag: TOTP und Wiederherstellung v1

Stand: 21.10.2026. KC-006E / #66. Sicherheitsregeln ausdrücklich bestätigt.
Bibliotheksprüfung und Antwortstand stehen in [ADR-033](../../docs/architecture/ADR-033_TOTP_und_Wiederherstellung.md).
Die Projektleitung hat „Ja, alle genannten Regeln bestätigen“ geantwortet.
MFA ist bei Verwaltungsrechten und besonders sensiblen Daten verpflichtend. Die konkrete Rechtezuordnung folgt KC-008.
Der interne Adapter gewährt keine produktive Anmeldung und ersetzt diese Zuordnung nicht.

## Zuständigkeit und Voraussetzungen

Der Dienst identity-access verwaltet zeitbasierte Einmalpasswörter (TOTP) und einmalige Wiederherstellungscodes.
Eine Einrichtung oder Änderung benötigt einen aktuell geprüften, dafür berechtigten Benutzerkontext.
Eine alte Sitzung oder eine vom Client übermittelte Rollenliste genügt dafür nicht.
Welche Konten MFA benötigen und welche Wiederherstellung zulässig ist, muss ausdrücklich bestätigt sein.
Die technischen Operationen ersetzen weder die spätere Zuordnung über KC-008 noch die vollständige Anbindung in KC-006F.
Ein einzelner TOTP- oder Wiederherstellungsnachweis erzeugt keine normale Sitzung und keine fachliche Berechtigung.

## TOTP-Ablauf

1. Prüfen Sie den zulässigen Einrichtungskontext. Erzeugen Sie ein eigenes zufälliges Geheimnis auf dem Server.
2. Speichern Sie das Geheimnis verschlüsselt als noch nicht aktivierten Faktor. Binden Sie es an Identität, Faktor und Schlüsselversion.
3. Geben Sie das Geheimnis nur am geschützten Einrichtungsrand aus. Bestätigen Sie die Einrichtung mit einem passenden TOTP-Nachweis.
4. Aktivieren Sie den Faktor atomar. Speichern Sie den dafür bereits verbrauchten Zeitschritt.
5. Prüfen Sie spätere Eingaben mit aktueller Serverzeit, begrenztem Zeitfenster und dem aktuellen Faktorbestand.
6. Verbrauchen Sie einen gültigen Zeitschritt atomar. Gleiche oder ältere bereits verbrauchte Schritte dürfen keinen zweiten Erfolg erzeugen.

Die technische Ausgangswahl ist HMAC-SHA-1 mit sechs Ziffern, 30 Sekunden je Schritt und einem ausdrücklich übergebenen begrenzten Toleranzfenster.
Die Wahl dient der TOTP-Kompatibilität. Sie ersetzt keine bestätigte Anwendungspolitik oder Geräteprüfung.
Es werden keine selbst entwickelten HMAC-, TOTP- oder Vergleichsverfahren verwendet.
Zeitwerte stammen aus dem geprüften Serverbestand. Rückläufige Zeit und ungültige Grenzwerte müssen ohne Freigabe scheitern.
Prüfversuche benötigen eine dauerhafte Reservierung und ausdrückliche Mengengrenzen; es gibt keine produktiven Vorgabewerte.
Ein Ausfall der erforderlichen Datenbank- oder Schlüsselprüfung darf keinen positiven Zwischenspeicher freigeben.

Ein Geheimnis darf weder als Klartext noch über Debugausgaben, Datenbankfehler, Ereignisnutzlasten oder Protokolle offengelegt werden.
Die Verschlüsselung muss auch eine Verschiebung zu einer anderen Identität oder einem anderen Faktor erkennen.
Nonce-Eindeutigkeit, Schlüsselversion und Verhalten nach Wiederanlauf müssen im späteren Implementierungsvertrag konkret geprüft werden.
Produktiver Schlüsselanbieter und Rotation sind noch nicht festgelegt. Tests verwenden ausschließlich zufällige eigene Schlüssel im Arbeitsspeicher.

## Wiederherstellungsablauf

Codes werden mit einer Betriebssystem-Zufallsquelle erzeugt. Rohe Codes werden einmal zur geschützten Offline-Aufbewahrung ausgegeben.
Im Bestand liegt ausschließlich eine geprüfte Ableitung, gebunden an Identität und aktuellen Wiederherstellungssatz.
Ein ersetzter Satz darf nicht parallel weiter gültig bleiben.

Verbrauch und nötige Zustandsänderungen benötigen eine gemeinsame atomare Prüfung.
Zwei konkurrierende Verwendungen desselben Codes dürfen höchstens einen Erfolg ergeben.
Widerrufene, verbrauchte, fremde oder falsch geformte Codes werden mit derselben begrenzten Fehlerdarstellung abgewiesen.
Der vorgeschlagene Ablauf widerruft bestehende Sitzungen und führt in einen eingeschränkten Zustand zur erforderlichen Faktorneueinrichtung.
Er gibt keine normale Sitzung oder Erlaubnis zum Überspringen einer bestätigten MFA-Pflicht aus.
E-Mail allein und ein nicht freigegebenes Ersatzverfahren dürfen keine MFA-Rücksetzung auslösen.
Diese Regeln sind ausdrücklich bestätigt.

## Faktorwechsel und Benachrichtigung

Ein Faktorwechsel benötigt einen aktuellen Nachweis des zulässigen Änderungszwecks.
Der bisherige Faktor darf nicht durch eine unbestätigte neue Einrichtung stillschweigend ersetzt werden.
Widerruf, Wiederherstellungssatz und Sitzungen müssen nach der bestätigten Regel konsistent geändert werden.
Aus alten positiven TOTP-, Passkey- oder Sitzungsentscheidungen darf keine neue Freigabe entstehen.
Die konkrete Prüfung der übergreifenden Anmelde- und Änderungszwecke folgt KC-006F.

Bestätigt sind ein Sicherheitsprotokoll und ein atomar vorgemerkter Benachrichtigungsauftrag für Einrichtung, Wechsel, Widerruf und Wiederherstellung.
Das Ziel muss bereits bestätigt sein. Eine ungeprüfte neue Kontaktadresse darf das bisherige Ziel nicht ersetzen.
Aufträge enthalten keine TOTP-Geheimnisse, Codes, Sitzungsnachweise oder vollständigen Fachinhalte.
KC-006E würde nur den Auftrag speichern. Der spätere Nachrichtendienst übernimmt den Versand.
Der interne Adapter speichert ausschließlich Aufträge und versendet keine Nachricht.

## Technische Konkretisierung vor Implementierung

TOTP verwendet ein eigenes zufälliges Geheimnis mit 20 Bytes, sechs ASCII-Ziffern und 30 Sekunden je Schritt.
Das explizite Toleranzfenster darf höchstens einen Schritt vor und nach der Serverzeit umfassen.
Einrichtungslaufzeit, Wiederherstellungszeit, Zahl der Offline-Codes und Versuchslimits werden ausdrücklich übergeben.
Die Zahl der Offline-Codes ist auf 1 bis 16 begrenzt. Jeder Code enthält 32 zufällige Bytes.
Die vorhandene dauerhafte Versuchsreservierung aus KC-006C wird gemeinsam verwendet. KC-006E benötigt deshalb technisch auch KC-006C.
Alle beteiligten Aufrufer müssen dieselbe Grenzkonfiguration verwenden; abweichende Konfigurationen werden abgewiesen.

Der Test-Schlüsselanbieter erzeugt Schlüsselmaterial und Schlüsselkennung frisch im Arbeitsspeicher. Es gibt keine Import-, Export- oder produktive Ladestelle.
Ein dauerhafter Datenbankzähler reserviert jede AES-GCM-Nonce vor der Verschlüsselung in einer eigenen bestätigten Operation.
Je Schlüsselkennung sind höchstens 2^32-1 Verschlüsselungen erlaubt. Die 96-Bit-Nonce kodiert den eindeutigen Zähler; verbrauchte Zähler werden nicht zurückgesetzt.
Identität, Faktor und Schlüsselkennung werden als zusätzliche authentisierte Daten gebunden.
Fehlender Schlüssel, falscher Bezug und beschädigter Chiffretext werden ohne Ausweichschlüssel abgewiesen.

Je Identität gibt es höchstens einen aktiven und einen noch nicht bestätigten TOTP-Faktor.
Ein neuer Einrichtungsversuch ersetzt nur den noch nicht bestätigten Faktor. Die Aktivierung eines Ersatzfaktors verlangt zusätzlich einen gültigen aktuellen Code des bisherigen TOTP-Faktors.
Nach Wiederherstellung verlangt die Faktorneueinrichtung stattdessen einen begrenzten zufälligen Wiederherstellungsnachweis.
Im Bestand liegt nur dessen Ableitung. Dieser Nachweis ist keine normale Sitzung.
Ein anderer noch unbenutzter Offline-Code kann bei verlorenem Wiederherstellungsnachweis einen neuen begrenzten Vorgang eröffnen.
Die erfolgreiche Faktoraktivierung ersetzt den Offline-Code-Satz vollständig und gibt den neuen Satz einmal aus.

Wiederherstellung widerruft bestehende Sitzungen, entfernt den bisherigen TOTP-Bestand und sperrt normale Anmeldeabschlüsse bis zur Faktorneueinrichtung.
Die Sperre muss in KC-006F auch bei Passwort- und Passkey-Nachweisen aktuell geprüft werden.
Änderungen des MFA-Bestands entwerten außerdem laufende Passkey-Vorgänge über deren aktuelle Versionskennung.
Die Sperrreihenfolge ist MFA-Konto, Passkey-Konto, Sitzungsbestand und Ereignisbestand. Verbrauch und Ereignisauftrag liegen in derselben Fachtransaktion.
Die Versuchszählung und Nonce-Reservierung bleiben davon getrennt dauerhaft verbraucht.

Der Adapter liest das Benachrichtigungsziel ausschließlich aus einem serverseitig bestätigten Kontaktverweis.
Der Test-Betriebszugang legt solche synthetischen Verweise an. Der Adapter bietet keine Funktion zur ungeprüften Kontaktbestätigung.
Fehlt der Verweis oder scheitert der Ereignisauftrag, wird die zugehörige Faktoränderung vollständig zurückgenommen.
Der spätere Versand benötigt den eigenen Vertrag des Nachrichtendienstes; er findet hier nicht statt.

## Datenbank- und Prüfgrenzen

Neue Objekte liegen im getrennten Plattform-Schema. Sie erhalten getrennte Wartungsrechte und erzwungene RLS ohne erlaubende Runtime-Policy.
Positive Speichertests verwenden ausschließlich die isolierte Test-Betriebsidentität.
SQL-Werte müssen gebunden sein. Kein Aufruf darf eigene Rollenwechsel oder vom Client bestimmte Zeit- und Rechtewerte verwenden.
Alle schutzrelevanten Änderungen benötigen den aktuellen Faktorbestand und eine eindeutige Sperrreihenfolge.
Ein Erfolg darf erst nach bestätigtem Commit zurückgegeben werden.
Ein fehlgeschlagener Prüfversuch darf durch Abbruch oder Rücknahme einer späteren Operation keinen zusätzlichen Versuch freigeben.

## Testplan

| Bereich | Erforderlicher Nachweis |
|---|---|
| Bibliothek | Geeignete öffentliche RFC-Testvektoren, richtige und falsche Codes sowie begrenzte Parameter. |
| Einrichtung | Noch nicht aktivierter Faktor wird nicht als Anmeldung akzeptiert; Bestätigung aktiviert genau den zugeordneten Faktor. |
| Zeit und Einmaligkeit | Fenstergrenzen, Ablauf, rückläufige Zeit, wiederholter Schritt und zwei tatsächlich konkurrierende Prüfungen. |
| Geheimnisschutz | Kein Klartext im Bestand; falscher Schlüssel, manipulierte Nutzlast und vertauschter Identitäts-/Faktorbezug werden abgewiesen. |
| Wiederherstellung | Richtiger Code einmalig; wiederholte, konkurrierende, fremde und widerrufene Codes sowie alter Satz abgewiesen. |
| Atomarität | Verbrauch, bestätigter Sitzungswiderruf, Faktorstatus und Benachrichtigungsauftrag bleiben bei Fehler gemeinsam konsistent. |
| Wiederanlauf | Verbrauchte Zustände vor und nach PostgreSQL-Neustart; fehlende Schlüssel führen zur Verweigerung. |
| Rechte und Fehler | Runtime-/Eigentümer-RLS, ungültige Eingaben und einheitliche Fehler ohne Geheimnisse. |

Tests verwenden nur synthetische Identitäten und eigene Zufallswerte beziehungsweise ausdrücklich öffentliche RFC-Testvektoren.
Produktive Konten, Schlüssel, Geheimnisse und Kontaktziele werden nicht angelegt.
Der vollständige Projekt-Lockfile, RustSec, Build und echte Datenbanktests müssen nach Übernahme gesondert bestehen.
Dieser Entwurf ist kein Implementierungs- oder Abnahmenachweis.
