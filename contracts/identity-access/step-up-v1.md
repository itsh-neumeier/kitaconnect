# Zusätzliche Authentisierung v1 – interner Vertrag

Stand: 21.10.2026. KC-006F / #67. Grundlage: Spezifikation v5.9, bestätigte MFA-Regeln und ADR-039.
Dieser Schritt erneuert die zusätzliche Authentisierung einer aktiven Anmeldung.
Er ist keine neue Anmeldung und verlängert deren absolute Laufzeit nicht.

## Zweck und Beginn

Ein neuer einmaliger Vorgang bindet den Zweck zusätzliche Authentisierung, die genaue aktuelle Sitzung,
deren Nachweisableitung, die Identität und die Einrichtung.
Er kann nicht durch den normalen Anmeldeabschluss verbraucht werden.
Ein normaler Anmeldevorgang kann nicht als zusätzliche Authentisierung übernommen werden.
Eine alte CurrentSession- oder Faktor-Momentaufnahme erzeugt keinen solchen Vorgang.

Vor Beginn müssen Sitzung, Bindung, Mitgliedschaft, Wiederherstellung und Fristen aktuell geprüft werden.
Eine bisher nur einfache Authentisierung darf ausschließlich einen neuen Faktorprüfvorgang beginnen.
Sie erhält dadurch keinen positiven MFA-Kontext oder Zugriff auf sensible Aktionen.
Die öffentliche normale Sitzungsprüfung behält ihre MFA-Anforderungen unverändert.

Die gemeinsame Reihenfolge lautet globaler Vorgangsmengenlock, Identitätssperre, Sitzungsbestand und Vorgang.
Diese Reihenfolge muss mit dem bestehenden Anmeldebeginn übereinstimmen und darf keinen neuen Sperrzyklus erzeugen.
Der neue Vorgang verwendet dieselben ausdrücklich übergebenen Lebensdauer- und Mengenlimits.
Es entstehen keine zusätzlichen produktiven Vorgabewerte. Beginn und Zweckbindung werden gemeinsam bestätigt.

## Tatsächliche Faktorprüfung

Zulässig sind die bereits bestätigten Kombinationen: Passwort plus TOTP oder Passkey mit zwingender Benutzerverifikation.
TOTP allein und Passwort allein genügen nicht für zusätzliche MFA-Authentisierung.
Die bestehenden Bibliotheksadapter merken den Nachweis weiterhin in ihrer tatsächlichen Faktortransaktion vor.
Der Vorgang bindet denselben aktuellen Faktorbestand und bleibt einmalig.
Bei einem zweckgebundenen Vorgang muss auch die gebundene Ziel-Sitzung noch lokal gültig sein.
Widerruf, Rotation, Wiederherstellung oder geänderte Zielbindung verweigern den Abschluss.
Dauerhaft reservierte Prüfversuche und verbrauchte WebAuthn-Challenges werden nicht zurückgegeben.

## Abschluss und neue Nachweisbindung

Der Abschluss prüft Zweck, genaue Ziel-Sitzung, Fristen, aktuellen Faktorbestand und die aktuelle Organisationsantwort erneut.
Die vollständige MFA-Kombination muss in diesem Vorgang tatsächlich nachgewiesen sein.
Ein unveränderlicher Authentisierungsbeleg bindet den geprüften Vorgang an dieselbe ursprüngliche Anmeldung.
Seine Zeit stammt aus der tatsächlichen neuen Faktorprüfung. Die erste Vormerkzeit bleibt erhalten.
Die erlaubte zusätzliche Frist ist weiterhin durch das bisher strengste Sitzungsminimum begrenzt.

Die Erstellzeit und absolute Laufzeit der Anmeldung bleiben erhalten.
Ein bestätigter neuer Beleg darf die aktuelle MFA-Stärke und zusätzliche Authentisierungszeit erneuern.
Er entfernt keine Wiederherstellungssperre, gewährt keine Mitgliedschaft und ersetzt keine Objekt-/Feldrechte.
Aktuelle Sitzungsprüfung liest den Beleg ohne positiven Zwischenspeicher.
Die ursprüngliche Anmeldebindung bleibt als Herkunftsnachweis unverändert.

Erfolgreicher Abschluss und Sitzungsrotation müssen in derselben lokalen Transaktion bestätigt werden.
Ein alter eventuell gestohlener Sitzungsnachweis darf nach dem neuen MFA-Erfolg nicht von der höheren Stärke profitieren.
Der neue rohe Nachweis wird ausschließlich nach Commit ausgegeben.
Auch eine nachfolgende Rotation darf weder Belegzeit noch Fristen verjüngen.
Abmeldung erfasst weiterhin die gesamte lokale Anmeldung einschließlich Rotationsnachfolger.

Fehler, Abbruch, Commitfehler und Ablauf während einer Sperrwartezeit dürfen keinen Beleg oder neuen Nachweis zurücklassen.
Ein beobachteter Ablauf bleibt gemäß Sitzungs- und Rotationsvertrag endgültig.
Die lokale Transaktion schließt den Organisationsbestand nicht atomar ein. Jede spätere geschützte Aktion prüft erneut.

## Datenbank und Kompatibilität

Eine neue Migration ergänzt Zweckbindung und unveränderliche Authentisierungsbelege.
Die Zweckunterscheidung ist beim Anmeldevorgang selbst sichtbar und wird durch normale Sitzungsabschlüsse ausdrücklich abgewiesen.
Neue Zwecke erhalten keine unverbindliche Nachklassifizierung nach einer bereits bestätigten Anlage.
Neue Tabellen erhalten FORCE RLS ohne erlaubende Runtime-Policy. Alte Migrationen bleiben unverändert.
Es gibt keine dienstübergreifenden Fremdschlüssel, eigenen Kryptoverfahren oder neuen Projekt-Schlüssel.
Frühere isolierte Testdatenbanken behalten ihren ausdrücklich geprüften Versionsstand.
Die Implementierung muss deren bisherigen Anmelde-, Sitzungs- und Widerrufsverträge erhalten.

## Nachweise

Tests verwenden tatsächliche Passwort-, TOTP- und Passkey-Bibliotheksprüfungen.
Zu prüfen sind beide MFA-Kombinationen, fehlender Faktor, falscher Zweck, falsche Sitzung/Einrichtung,
abgelaufene oder widerrufene Ziele, Rotation zwischen Faktor und Abschluss, geänderter Faktorbestand,
Einmaligkeit unter echten Sperrwartenden, unveränderte absolute Frist und alte Nachweisverweigerung.
Commitfehler, unterdrückte Beleganlage, Kandidatenfehler, Ablauf während Wartezeit,
Migration, RLS und Neustart benötigen eigene negative Nachweise.
Synthetische Organisationsantworten bleiben ausdrücklich Testdoppel; sie ersetzen keine mTLS- oder produktive Mitgliedschaftsprüfung.
Gateway und Browser-/TLS-Nachweise bleiben gesondert. Das zusätzliche HTTPS-Testprofil benötigt weiterhin eine ausdrückliche Antwort.

## Erster überprüfbarer Teil: Speichergrundlage

Der erste Folge-PR legt ausschließlich die Speichergrundlage an.
Migration 0008 ergänzt die Routen `reauth_password` und `reauth_passkey` sowie unveränderliche Zielbindungen und Authentisierungsbelege.
Der bisherige Rust-Adapter weist diese unbekannten Routen weiterhin ab.
Dieser Schritt erzeugt keine neue öffentliche Operation und erneuert keine tatsächliche Authentisierung.

Eine Zielbindung darf nur vor dem ersten Faktor an einen neuen Vorgang und die genaue aktive Sitzung angelegt werden.
Ein Beleg erfordert einen verbrauchten starken Zusatzvorgang, die Zielbindung und die bereits atomar angelegte Rotationsverknüpfung.
Er bindet den Nachfolger und die ursprüngliche Anmeldung. Fremde Ziele, normale Vorgänge, fehlende MFA und veränderte Zeiten werden abgewiesen.
Neue Tabellen erhalten ausschließlich SELECT/INSERT für die weiterhin gesperrte Laufzeitrolle.
Trigger verweigern spätere Änderung und Löschung auch durch die Eigentümerrolle bei sichtbaren Zeilen.

Die isolierten Speicherprüfungen verwenden ausdrücklich synthetische Faktor-Markierungen.
Sie beweisen Datenbankbedingungen, Migration, Rücknahme, Einmaligkeit, RLS und Neustart.
Tatsächliche Faktorprüfung, aktuelle Zielprüfung bei jedem Faktor, atomarer Abschluss mit Rotation und Verwendung des Belegs bleiben im folgenden Rust-Adapter zu implementieren und gesondert zu testen.
