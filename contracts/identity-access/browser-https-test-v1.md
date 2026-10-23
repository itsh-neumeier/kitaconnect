# Dienstvertrag: Isoliertes HTTPS-Browserprofil v1

- Stand: 22.10.2026
- Umfang: KC-006F / #67, ausdrücklich bestätigtes zusätzliches Testprofil vor Code
- Status: Entscheidung bestätigt; noch kein Browser- oder Gateway-Nachweis

## Bestätigte Grenze

Die Projektleitung bestätigt am 22.10.2026 ausdrücklich das zusätzliche Profil `https://localhost:3443`.
HTTPS bezeichnet HTTP über eine TLS-geschützte Verbindung.
Das Profil dient ausschließlich isolierten Browser-, Cookie- und Gateway-Tests.
Die produktive Domain und der produktive Zertifikatsbetrieb bleiben offen.

Das vorhandene WebAuthn-Profil mit RP-ID `localhost` und Ursprung `http://localhost:3000` bleibt getrennt erhalten.
RP-ID bezeichnet die Kennung der WebAuthn-Gegenstelle (Relying Party).
Das neue Profil verwendet ebenfalls RP-ID `localhost`, aber ausschließlich Ursprung `https://localhost:3443`.
Ein Testvorgang und sein Bibliothekszustand bleiben an genau ihr Profil gebunden.
Es gibt keine automatische Portauswahl, kombinierte Ursprungsfreigabe oder produktive Ausweichkonfiguration.
Ist Port 3443 belegt, muss der Test diesen Zustand melden. Er darf keinen fremden Prozess beenden.

## Zertifikate und Cookies

Testzertifikate und private Schlüssel werden ausschließlich neu zur Laufzeit erzeugt.
Sie werden nicht versioniert oder als Projektgeheimnis wiederverwendet.
Testvertrauen bleibt auf den eigenen flüchtigen Testbestand begrenzt; der globale Vertrauensbestand wird nicht verändert.
Der Test entfernt seinen eigenen temporären Bestand nach Abschluss.
Die bestehenden RAM-Vorgaben der internen mTLS-Tests bleiben erhalten.
Das Profil wählt keinen produktiven Aussteller und installiert keine produktive Zertifikatskonfiguration.

Bestätigte Cookie-Regeln bleiben verbindlich: Secure, HttpOnly, SameSite=Strict, Path=/ und keine Domain-Freigabe.
Serverseitig gebundener CSRF-Nachweis und genaue Origin-Prüfung bleiben einschließlich Anmeldung und Abmeldung erforderlich.
CSRF bezeichnet hier das Fälschen browserbasierter Anfragen über eine fremde Webseite.
Das HTTPS-Profil darf keine dieser Prüfungen abschalten.
Ein Browsernachweis, ein Routertest und ein TLS-Nachweis werden getrennt ausgewiesen.

## Notwendige spätere Nachweise

Die spätere Implementierung prüft Cookies über tatsächliches HTTPS und verweigert fremden Ursprung oder fehlenden CSRF-Nachweis.
WebAuthn-Vorgänge dürfen nicht zwischen HTTP- und HTTPS-Testprofil ausgetauscht werden.
Fehlende oder falsche Zertifikatsprüfung darf nicht als bestandener TLS-Nachweis ausgegeben werden.
Es entstehen keine produktive Domain, neue Fachrechte oder Freigabe für externe SSO-Anmeldung.
SSO bezeichnet die föderierte Anmeldung und bleibt KC-007 vorbehalten.
