# Dienstvertrag: Gemeinsame mTLS-Grundlage v1

- Stand: 22.10.2026
- Umfang: KC-006F / #67, technischer Folgebaustein nach ADR-019/042
- Status: Vertrag vor Implementierung; kein öffentlicher Endpunkt

## Grenze

Eine gemeinsame technische Rust-Bibliothek prüft TLS mit beidseitigen Zertifikaten (mTLS).
Sie enthält keine Fachmodelle, Datenbankzugriffe, Dienstregistrierung oder Benutzerberechtigungen.
Eine geprüfte Zertifikatskette beweist noch keine aktive Dienstregistrierung.
Der zuständige Dienst muss danach die aktuelle Registrierung, den Diensttoken und den konkreten Berechtigungsumfang prüfen.
Der bisherige synthetische `PeerReference`-Konstruktor wird durch diesen Teil noch nicht als produktive Autorität veröffentlicht.

## Konfiguration

Jede Seite erhält ausdrücklich eigene Zertifikatskette, privaten Schlüssel und ausschließlich die erlaubten Vertrauensanker ihrer Installation.
Leere Vertrauensanker, fehlende Schlüssel oder Zertifikate und ungültige Zertifikatsdaten verweigern Konfiguration.
Es gibt keine Projektzertifikate, Betriebssystem-Vertrauensanker oder produktiven Vorgabedomains.
Die Bibliothek verwendet Rustls 0.23.46 mit dem bereits verwendeten Ring-Kryptografieanbieter und Tokio-Rustls 0.26.6.
Die vorhandene X.509-Bibliothek 0.16.0 liest nur Zertifikatszeiten; Rustls prüft Kette, Signatur und Verwendungszweck.
Die Testabhängigkeit rcgen 0.14.10 erzeugt ausschließlich flüchtige Testzertifikate.
Quellen: [Rustls](https://docs.rs/rustls/0.23.46/rustls/), [Tokio-Rustls](https://docs.rs/tokio-rustls/0.26.6/tokio_rustls/), [rcgen](https://docs.rs/rcgen/0.14.10/rcgen/).
Der spätere HTTP-Router bleibt gemäß Spezifikation Axum; dieses Paket enthält noch keinen HTTP-Router.

## Verbindungsprüfung

Rustls muss auf beiden Seiten die vollständige konfigurierte Vertrauenskette prüfen.
Der Server verlangt ein Clientzertifikat. Der Client prüft den ausdrücklich übergebenen Servernamen.
Es gibt keine abschaltbare Zertifikatsprüfung und keine anonyme Ersatzverbindung.
Das tatsächliche Endzertifikat muss aktuell gültig sein und darf gemäß ADR-019 höchstens 86.400 Sekunden gelten.
Die Laufzeit prüft dies für das eigene und das empfangene Endzertifikat.
Die Bibliothek bietet zunächst ausschließlich das Anwendungsprotokoll `http/1.1` an und verlangt diese Aushandlung.
TLS verwendet die sicheren Rustls-Voreinstellungen. Frühe Anwendungsdaten und Sitzungswiederaufnahme werden deaktiviert.
Der Aufrufer übergibt für jeden Handshake ein positives Zeitlimit. Es gibt keinen produktiven Vorgabewert.
Die Bibliothek öffnet keinen Listener, startet keine unbeschränkte Annahmeschleife und verwaltet keinen globalen Pool.
Annahmegrenzen und Zeitlimits späterer HTTP-Anfragen erhalten ihren eigenen Vertrag.

## Geprüfte Herkunft und Zeit

Nur ein erfolgreicher Handshake kann eine geprüfte Verbindung erzeugen.
Die SHA-256-Ableitung stammt aus dem tatsächlichen DER-Endzertifikat dieser Verbindung.
Ein HTTP-Header oder ein vom Aufrufer übergebener Fingerabdruck darf diese Ableitung nicht ersetzen.
Der Typ besitzt keinen öffentlichen Konstruktor aus behaupteten Metadaten.
Vor jeder späteren geschützten Aktion muss der Adapter die Verbindung erneut auf aktuelle Zertifikatszeit prüfen.
Dies gilt für beide Endzertifikate auch auf wiederverwendeten Verbindungen.
Beobachteter Ablauf oder rückläufige Zeit sperrt die Metadatenprüfung dieser Verbindung endgültig.
Eine alte positive Zertifikatsmomentaufnahme ist kein wiederverwendbarer Berechtigungsschein.
Der technische Datenstrom allein erteilt kein Recht auf eine Aktion.

## Nachweise und offene Folgearbeit

Tests verwenden wirkliche TCP-Verbindungen ausschließlich auf `127.0.0.1` und vom Betriebssystem zugewiesene freie Ports.
Testnamen wie `organization.test` werden nur für die TLS-Namensprüfung verwendet; es erfolgt keine DNS-Auflösung.
Dies ist kein Browser-/WebAuthn-Profil und implementiert nicht das noch offene Profil `https://localhost:3443`.
Alle Schlüssel und Zertifikate entstehen neu im Arbeitsspeicher und gelangen nicht in Dateien, Ausgabe oder Git.
Positive Tests prüfen beide tatsächlichen Ableitungen und verschlüsselten Datenaustausch.
Negative Tests prüfen fehlende Clientzertifikate, fremde Vertrauensanker, falschen Servernamen,
abgelaufene und noch nicht gültige Endzertifikate, überlange Gültigkeit, fehlende Protokollaushandlung und Handshake-Zeitlimit.
Eine offene Verbindung muss nach Zertifikatsablauf ihre aktuelle Metadatenprüfung verweigern.
Zertifikatswechsel muss eine andere tatsächliche Ableitung ergeben.

Registrierungswiderruf und aktuelle Tokenintrospektion folgen auf dieser Grundlage getrennt.
HTTP-Verträge, OrganisationAuthority-Adapter, Gateway, Browserprüfungen und Gesamtabnahme bleiben offen.
Produktiver Aussteller, Schlüsselbetrieb, Domain und Rollen-/Poolbindung bleiben gesondert.
Die Bibliothek stellt keine Zertifikate aus; die Testabhängigkeit wählt keinen produktiven Aussteller.
