# Transportvertrag: Begrenzter OAuth-HTTP-Client v1

- Stand: 22.10.2026
- Umfang: KC-006F / #67, Transport zu den bestehenden OAuth-Endpunkten
- Grundlage: ADR-019/044/045/050 und service-oauth-v1

## Umfang und Herkunft

Der Client verarbeitet einen ausdrücklich übergebenen verbundenen TCP-Socket.
Er öffnet weder einen Listener noch einen unbegrenzten Hintergrundauftrag.
Er verwendet die bestehende tatsächliche mTLS-Prüfung mit ausdrücklichen Vertrauensankern und genau konfiguriertem DNS-Zielnamen.
Es gibt keine Betriebssystem-Vertrauensanker, produktiven Namen oder automatischen Ausweichadressen.
TCP-Verbindungsaufbau und dessen Zeitlimit bleiben Aufgabe des Aufrufers.
Das Bibliothekspaket service-transport erhält keine Datenbankabhängigkeit.

Nur die beiden vorhandenen OAuth-Endpunkte sind auswählbar: `/oauth/token` und `/oauth/introspect`.
Anfragen verwenden POST, ein begrenztes Formular und `application/x-www-form-urlencoded`.
Die Methode erhält keine beliebigen Header, URL-Parameter, Cookies, Weiterleitungsziele oder alternativen Anmeldemittel.
Die behauptete Dienstkennung bleibt ein Formularwert bis zur aktuellen Prüfung durch den Server.

## Grenzen und Lebenszyklus

Der Aufrufer konfiguriert positive Grenzen für gleichzeitige Aufrufe, Formularbytes, Antwortbytes, Headerzahl und Headerpuffer.
Der Headerpuffer beträgt technisch mindestens 8.192 Bytes.
Handshake, Antwortheader, Antwortbody und gesamter Aufruf erhalten ausdrücklich positive, darstellbare Zeitlimits.
Es gibt keine produktiven Voreinstellungen oder unbegrenzten Wiederholungen.
Ein erschöpftes Budget verwirft den übergebenen Socket vor dem Handshake.

Jeder Aufruf besitzt genau eine neue geprüfte Verbindung. Es gibt keinen Verbindungspool.
Die HTTP-Verbindungsoperation und die Antwortverarbeitung werden innerhalb derselben aufrufenden Operation gleichzeitig abgefragt.
Ein erfolgreicher Aufruf, Fehler, Abbruch oder Zeitlimit verwirft diese Verbindung.
Der Client startet dafür keinen losgelösten Hintergrundauftrag und folgt keiner Weiterleitung.
Zertifikatszeit und gemeinsamer Sperrzustand werden nach dem Empfang erneut geprüft, solange die Verbindung noch besteht.
Eine vor diesem Abschluss geschlossene oder gesperrte Verbindung darf kein positives Transportergebnis erzeugen.

## Antwort und Grenzen der Aussage

Antworten müssen `Cache-Control: no-store` und `Pragma: no-cache` jeweils genau einmal enthalten.
Ein nicht leerer Body muss den Medientyp `application/json` haben; optional ist genau ein UTF-8-Charset zulässig.
Leere Protokollantworten bleiben mit ihrem Status erhalten. Weiterleitungen und widersprüchliche Header werden abgewiesen.
Header und Body werden vor ihrer Weitergabe begrenzt. Es gibt keine automatische Dekompression.
Der rohe Antwortbody liegt im Ergebnistyp in einem beim Freigeben überschriebenen Puffer.
Debug- und Fehlerausgaben enthalten weder Formularwerte noch Token, Header oder Diagnosedaten.
Interne Puffer der HTTP-/TLS-Bibliotheken werden dadurch nicht pauschal als überschrieben behauptet.

Ein Ergebnis bestätigt ausschließlich den Empfang über die geprüfte Verbindung.
Es bestätigt keine aktuelle Serverregistrierung, Tokenaktivität, Mitgliedschaft, MFA oder Fachberechtigung.
Der Verbraucher prüft das genaue OAuth-Format, Aussteller, Empfänger, Zweck, Zeit und tatsächliche Zertifikatsbindung nach dem jeweiligen Dienstvertrag.
Eine Fehlerantwort oder ein nicht verfügbares Transportergebnis darf keine Freigabe erzeugen.
Positive Antworten dürfen nicht als Berechtigung für spätere Aufrufe zwischengespeichert werden.
Produktiver Zertifikatsbetrieb und produktive Rollen-/Poolbindung bleiben gesondert.

## Nachweise und Abhängigkeiten

Tatsächliche Loopback-/mTLS-/HTTP-Tests prüfen beide Pfade, Formularübertragung, begrenzte Antworten, Cache-/Inhaltsregeln,
Weiterleitungsverweigerung, Zeitlimits, Budgetgrenze und Freigabe nach Abbruch.
Vorhandene CA-/DNS-/Zertifikatszeitprüfungen bleiben erhalten.
Frische Testschlüssel und Zertifikate bleiben ausschließlich im Arbeitsspeicher. Dies ist kein Browserprofil.

Neue direkte Einträge verwenden ausschließlich bereits gesperrte Paketversionen:
Hyper 1.12.0, Hyper-Util 0.1.21, http-body-util 0.1.5, Mime 0.3.17 und Zeroize 1.9.1.
Das aktivierte Hyper-Clientmerkmal ergänzt die transitiven Pakete want 0.3.2 und try-lock 0.2.5.
Beide Pakete sind MIT-lizenziert und werden im Lockfile gesperrt sowie durch die bestehende Sicherheitsprüfung geprüft.
Bestehende Paketversionen bleiben erhalten. Es entsteht keine Migration, Datenbankrolle oder erlaubende Policy.

Quellen: [Hyper-Client](https://docs.rs/hyper/1.12.0/hyper/client/conn/http1/),
[begrenzter HTTP-Body](https://docs.rs/http-body-util/0.1.5/http_body_util/struct.Limited.html).
