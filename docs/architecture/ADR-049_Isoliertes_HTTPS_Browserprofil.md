# ADR-049: Zusätzliches isoliertes HTTPS-Browserprofil

- Datum: 22.10.2026
- Status: Ausdrücklich bestätigt, vor abhängigem Code dokumentiert
- Umfang: KC-006F, zusätzliche lokale Testumgebung

## Entscheidung

Die Projektleitung bestätigt ausdrücklich `https://localhost:3443` mit ausschließlich zur Laufzeit erzeugten Testzertifikaten.
Der [Testprofilvertrag](../../contracts/identity-access/browser-https-test-v1.md) hält die Grenze fest.
Das bestehende WebAuthn-Profil `http://localhost:3000` bleibt getrennt erhalten.
Die produktive Domain und der produktive Zertifikatsbetrieb bleiben offen.

## Begründung und Grenzen

Das zusätzliche Profil ermöglicht spätere tatsächliche HTTPS-Prüfungen der bestätigten Secure-Cookies.
Es erlaubt keine gemeinsame Ursprungsfreigabe, produktive Ausweichkonfiguration oder Wiederverwendung von Testschlüsseln.
Die Implementierung benötigt weiterhin getrennte Browser-, Router- und TLS-Nachweise.
Diese Entscheidung bestätigt die Testumgebung und behauptet noch keine implementierte Browseranmeldung.
MFA, Sitzungsfristen, CSRF und genaue Origin-Prüfung bleiben unverändert verbindlich.
