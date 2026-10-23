# Interner Vertrag: WebAuthn v1

Stand: 20.10.2026. KC-006D / #65. Sicherheitsprofil am 20.10.2026 ausdrücklich bestätigt.
Dieser Vertrag ist keine Freigabe eines produktiven Anmeldewegs.
Die Bibliotheksprüfung und offenen Punkte stehen in [ADR-032](../../docs/architecture/ADR-032_WebAuthn_Vorbereitung.md).

## Zuständigkeit und Grenzen

Der Dienst identity-access verwaltet WebAuthn-Registrierung und Nachweisprüfung.
Passkeys bleiben ein eigenständiger Weg ohne vorausgesetzte Passwortprüfung.
Ein positives Ergebnis ist ein aktueller lokaler Identitätsnachweis und keine fachliche Berechtigung.
Sitzungserzeugung, Cookies, aktuelle Mitgliedschaft und Objekt-/Feldrechte folgen den gesonderten Verträgen in KC-006F und KC-008.
Die Registrierung benötigt einen bereits geprüften Zuständigkeitsnachweis. Dieser Entwurf führt keinen ungeprüften Ersteinrichtungs- oder Wiederherstellungsweg ein.

## Bestätigtes Prüfprofil

Eine produktive RP-ID und ein erlaubter Ursprung sind noch nicht festgelegt.
Die Projektleitung hat „Alle drei Vorschläge bestätigen“ geantwortet.
Der Adapter dieses Arbeitspakets verwendet ausschließlich RP-ID `localhost` und Ursprung `http://localhost:3000` im isolierten Test.
Ein solches Profil darf keine produktive Ausweichkonfiguration sein.
Weitere Ports, Subdomains, eingebettete Zugangsdaten und unvollständige Ursprungsangaben werden nicht automatisch zugelassen.
Client-Header dürfen die serverseitig konfigurierte RP-ID oder den Ursprung nicht ersetzen.

Benutzerverifikation für jeden Passkey-Vorgang und die Zulassung synchronisierter Passkeys sind ausdrücklich bestätigt.
webauthn-rs 0.5.5 übernimmt die Protokollprüfung. Die allgemeinen MFA-Regeln anderer Anmeldewege bleiben offen.
Ein Timeout der Browserabfrage ersetzt keine serverseitige Ablaufprüfung.
Der Aufrufer muss positive Laufzeit- und Mengengrenzen ausdrücklich übergeben; produktive Vorgabewerte werden hier nicht festgelegt.
Der Adapter begrenzt Antworten auf 65.536 Bytes und höchstens 32 Credentials je Identität.
Eine feste Obergrenze von 32 Credentials gilt auch für die explizite Mengenkonfiguration.

## Geplanter serverseitiger Ablauf

1. Prüfen Sie den bereits authentisierten Registrierungskontext beziehungsweise den angefragten internen Anmeldekontext.
2. Erzeugen Sie Challenge und Bibliothekszustand auf dem Server. Binden Sie beide an Identität, Zweck und geprüftes RP-/Ursprungsprofil.
3. Speichern Sie den Prüfzustand ausschließlich auf dem Server. Geben Sie an den Client nur die benötigten WebAuthn-Optionen und eine zufällige Vorgangsreferenz aus.
4. Reservieren und verbrauchen Sie den Vorgang beim Abschluss atomar. Prüfen Sie dafür Datenbankzeit, Ablauf, Zweck und aktuellen Zustand.
5. Prüfen Sie die Antwort mit der Bibliothek. Eine ungültige Antwort darf den verbrauchten Vorgang nicht wieder freigeben.
6. Prüfen Sie vor dem positiven Ergebnis den aktuellen Credential-Bestand erneut. Übernehmen Sie zulässige Zustandsänderungen atomar.

Die Referenz muss von der Challenge und der internen Datensatzkennung getrennt sein.
Ein Neustart oder zwei konkurrierende Abschlüsse dürfen keine zweite Verwendung ermöglichen.
Ein Abbruch nach Verbrauch kann einen neuen Vorgang erfordern. Er darf keinen alten Vorgang reaktivieren.
Prüfzustand darf weder in Client-Cookies noch in versteckten Formularfeldern abgelegt werden.
Falls Bibliothekszustand serialisiert wird, bleibt das Ergebnis ausschließlich im geschützten Serverbestand.

Der Bestand muss Credential-Kennungen eindeutig zuordnen. Ein Credential einer anderen Identität darf nicht übernommen werden.
Ein zwischenzeitlicher Widerruf oder eine Änderung des Credential-Bestands muss ein altes positives Ergebnis entwerten.
Zähler und Sicherungskennzeichen benötigen eine an die bestätigte Passkey-Regel gebundene Prüfung.
Eine Challenge bleibt unabhängig vom Verhalten eines Authentikatorzählers einmalig.
Jeder Vorgang speichert die aktuelle Versionskennung des zugeordneten Credential-Bestands.
Registrierung, Widerruf und erfolgreiche Anmeldung ändern diese Kennung unter derselben Kontosperre.
Alle Credential-Schreibzugriffe müssen diesen Ablauf verwenden. Externe direkte Schreibzugriffe ohne Kontosperre und Versionsänderung sind nicht erlaubt.
Ein Vorgang mit altem Bestand muss neu begonnen werden. Auch parallel begonnene Anmeldungen können dadurch einen neuen Vorgang benötigen.
Fehlgeschlagene Abschlussprüfungen bleiben verbraucht. Ein Fehler nach Verbrauch nimmt diesen Verbrauch nicht zurück.
Abgelaufene Vorgänge dürfen bei neuer Anlage entfernt werden; der Server begrenzt die Zahl gespeicherter Vorgänge ausdrücklich.
Signaturprüfungen laufen begrenzt außerhalb des asynchronen Ausführungsfadens.
Neue Datenbankobjekte erhalten erzwungene RLS ohne erlaubende Runtime-Policy.
Positive Speichertests verwenden ausschließlich die isolierte Test-Betriebsidentität.
Bibliothekszustände und Antworten dürfen auch über Bibliotheks-Debugausgaben nicht protokolliert werden.

## Testplan vor Abnahme

| Bereich | Erforderlicher Nachweis |
|---|---|
| Registrierung | Gültiger synthetischer Vorgang; danach dauerhaft zugeordnetes Credential; doppelte und fremde Credential-Zuordnung abgewiesen. |
| Anmeldung | Gültiger unabhängiger Passkey-Nachweis ohne Passwortbestand; keine Sitzung oder Fachberechtigung allein durch den Bibliotheksaufruf. |
| Challenge | Falsche, vertauschte, abgelaufene und wiederverwendete Challenge abgewiesen; falscher Zweck und fremde Identität abgewiesen. |
| RP und Ursprung | Falsche RP-ID, falscher Ursprung, zusätzlicher Port und nicht freigegebene Subdomain abgewiesen. |
| Antwortprüfung | Manipulierte Clientdaten, Authentikatordaten und Signatur abgewiesen; Benutzerverifikation gemäß bestätigter Regel geprüft. |
| Konkurrenz | Zwei tatsächlich wartende Abschlüsse ergeben höchstens einen Erfolg; aktueller Credential-Widerruf verhindert Erfolg aus altem Prüfzustand. |
| Persistenz | Einmaligkeit und Ablauf vor und nach PostgreSQL-Neustart; gescheiterte neue Migration wird vollständig zurückgenommen. |
| Rechte und Fehler | Runtime und Eigentümer-RLS negativ geprüft; einheitliche Fehler ohne Nachweise, Kontenkennungen oder interne Datenbankdetails. |

Tests verwenden Bibliotheks-Authentikatoren oder geeignete bestehende Browser-Testwerkzeuge.
Es wird keine eigene Test-Kryptografie entwickelt. Private Schlüssel und rohe Vorgangsreferenzen bleiben nur im Arbeitsspeicher.
Eine simulierte erfolgreiche Benutzerverifikation belegt lediglich die serverseitige Prüfung des Protokollmerkmals.
Tests mit echten Browsern und Geräten sowie die Anbindung aus KC-006F bleiben davon getrennt nachzuweisen.

## Reihenfolge nach Entscheidung

1. Aktualisieren und versionieren Sie diesen Vertrag sowie ADR-032 mit den ausdrücklichen Antworten.
2. Prüfen Sie Bibliotheksmerkmale, vollständigen Lockfile-Diff, native Voraussetzungen und RustSec-Ergebnis.
3. Implementieren Sie den begrenzten Serveradapter und eine neue Migration. Ändern Sie keine vorhandene Migration.
4. Führen Sie die Protokoll- und Datenbanktests aus. Dokumentieren Sie verbleibende Grenzen in Issue, PR und Checkpoint.
