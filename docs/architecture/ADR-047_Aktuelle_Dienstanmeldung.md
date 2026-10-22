# ADR-047: Aktuelle Dienstanmeldung mit DNS und Fingerabdruck

- Datum: 22.10.2026
- Status: Bestätigte Namens-/Fingerabdruckentscheidung vor Code konkretisiert
- Umfang: KC-006F, interner Adapter ohne HTTP-Laufzeitfreigabe

## Entscheidung

Der [Dienstanmeldevertrag](../../contracts/identity-access/service-authentication-v1.md) verbindet tatsächliches mTLS und aktuellen Registrierungsbestand.
Die ausdrücklich bestätigte Namenszuordnung ist zunächst eine unveränderliche, von außen übergebene Konfiguration.
Aktiver Dienst, exakter Fingerabdruck, Zeiten und Aussteller bleiben im bestehenden Identitätsbestand.
Jede Prüfung liest diese Zustände erneut. Fehlende Namenszuordnung verweigert Zugriff.
Der erwartete DNS-Name wird ausschließlich gegen den tatsächlichen SAN geprüft.
Es gibt keine Wildcards, automatische Registrierung oder produktiven Namensvorgaben.

## Begründung und Alternative

Die Namenskonfiguration ist ein ausdrücklich statisch registrierter Teil der Clientkonfiguration nach RFC 8705.
Sie benötigt keine zusätzliche Verwaltungsoberfläche oder Schreibrechte der Laufzeit.
Eine dynamische Namensverwaltung wird in diesem Schritt nicht eingeführt.
Ein Fingerabdruck ohne registrierten Namen oder ein Zertifikatsname ohne aktiven Fingerabdruck genügt nicht.
Die vorhandene bestätigte Zuordnung wird so ohne neue Tabellen oder erlaubende RLS-Policies umgesetzt.

## Folgen und Grenzen

Änderungen der Namenskonfiguration verlangen einen neuen Konfigurationsbestand.
Widerruf aktiver Dienste und genauer Zertifikate bleibt dagegen bei der nächsten aktuellen Prüfung wirksam.
Alle Zertifikatsmetadaten stammen aus dem tatsächlichen Handshake.
Eigene schreibgeschützte Transaktionen erhalten dieselben abgewarteten Fehlerpfade wie andere Adapter.
Diensttokens, Benutzerrechte, produktiver Schlüsselbetrieb und HTTP-Endpunkte bleiben gesondert.
Die Konfiguration erteilt keine fremden Datenbankrechte und ersetzt keine aktuelle Introspektion.
