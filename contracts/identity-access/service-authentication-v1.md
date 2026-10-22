# Dienstvertrag: Aktuelle Dienstanmeldung v1

- Stand: 22.10.2026
- Umfang: KC-006F / #67, interner Adapter vor Routercode
- Grundlage: ausdrückliche Bestätigung „DNS-Name plus aktiven Fingerabdruck“

## Konfiguration und tatsächliche Herkunft

Eine ausdrücklich übergebene, unveränderliche Namenskonfiguration registriert genau einen erwarteten DNS-Namen je enthaltenem Dienst.
Fehlende Zuordnungen verweigern die Anmeldung. Doppelte Dienste oder mehrfach verwendete Namen verweigern Konfiguration.
Es gibt keine automatische Registrierung, Domainvorgabe oder Dateischreibfunktion.
Produktive Werte kommen später aus geschützter Betriebskonfiguration. Dieser Adapter lädt keine produktive Konfiguration.
Eine Änderung der Namenskonfiguration verlangt einen neuen Adapterbestand; aktive Zustände bleiben trotzdem bei jeder Prüfung abzufragen.

Namen verwenden ausschließlich ASCII-DNS-Labels mit höchstens 63 Zeichen je Label und 253 Zeichen insgesamt.
Leere Labels, führende oder abschließende Bindestriche, Wildcards, abschließender Punkt und IP-Adressen werden abgewiesen.
Großbuchstaben werden zu Kleinbuchstaben vereinheitlicht. Ein internationalisierter Name muss bereits als ASCII-DNS-Name vorliegen.
Es gibt keine Teilstring-, Suffix- oder Wildcardübereinstimmung und keinen Ersatz über den Common Name.
Der erwartete Name muss einem tatsächlichen `dNSName` im Subject Alternative Name (SAN) des geprüften Endzertifikats entsprechen.
Mehrere SAN-Einträge im Zertifikat erzeugen keine weiteren registrierten Dienstidentitäten.

Nur `ConnectionPeer` aus dem tatsächlichen mTLS-Handshake liefert das Endzertifikat.
Die Bibliothek liest dessen SAN aus denselben DER-Daten wie Fingerabdruck und Gültigkeitszeit.
Eine Clientkennung ist bis zur vollständigen Prüfung eine Behauptung.
Grundlage: [RFC 8705, Abschnitt 2.1](https://www.rfc-editor.org/rfc/rfc8705#section-2.1).

## Aktuelle Prüfung

1. Prüfen Sie die aktuelle Verbindung und den genau registrierten DNS-Namen.
2. Starten Sie eine eigene schreibgeschützte READ-COMMITTED-Transaktion. Bereits offene Aufrufertransaktionen werden abgewiesen.
3. Lesen Sie aktive Dienstregistrierung, nicht widerrufenen genauen Fingerabdruck und passende Installation/Aussteller gemeinsam aus dem Identitätsbestand.
4. Vergleichen Sie beide gespeicherten Zertifikatszeiten exakt mit den tatsächlichen Zertifikatszeiten.
5. Lesen Sie die Datenbankzeit nach der Abfrage erneut. Rückläufige Zeit, noch nicht gültiges oder abgelaufenes Zertifikat verweigert die Anmeldung.
6. Prüfen Sie nach den Warteoperationen erneut die Verbindung. Bestätigen Sie die eigene Lesetransaktion und prüfen Sie den Verbindungszustand erneut vor der Rückgabe.

Jede Anmeldung liest den aktuellen Bestand erneut. Es gibt keinen positiven Registrierungscache.
Ein bestätigter Widerruf muss bei der nächsten Prüfung auch auf derselben offenen Verbindung greifen.
Die maßgebliche Registrierungslesung ist ein einzelner aktueller Datenbank-Snapshot; eine dienstübergreifende Atomarität wird nicht behauptet.
Die unveränderliche Namenskonfiguration ersetzt keine aktuelle Abfrage aktiver Dienste und Zertifikate.
Eigene Fehlertransaktionen warten Rollback gemäß ADR-040 ab. Lesefehler oder fehlende notwendige Daten geben keinen Nachweis zurück.

## Ergebnis und Rechte

Nur die erfolgreiche Prüfung liefert die interne `PeerReference` für den tatsächlich geprüften Dienst und Fingerabdruck.
Sie ist kein Diensttoken und keine Benutzer-, Mitgliedschafts- oder Fachberechtigung.
Der spätere Handler prüft zusätzlich die aktuelle Freigabe und den Tokenzustand sowie alle fachlichen Rechte.
Nach weiteren Warteoperationen sind notwendige aktuelle Prüfungen erneut auszuführen.
Der Adapter öffnet keinen Listener, stellt keine Tokens aus und verändert keine Registrierung.
Die vorhandene Laufzeitrolle bleibt durch FORCE RLS gesperrt; dieser Teil ergänzt keine erlaubende Policy.

## Nachweise

Tests verbinden tatsächliches mTLS mit einer getrennten synthetischen Identitätsdatenbank.
Frische Schlüssel und Zertifikate entstehen nur im Arbeitsspeicher; freie Loopback-Ports ersetzen keine produktive Domain.
Positive und negative Tests prüfen SAN plus Fingerabdruck, falsche Dienstbehauptung, fehlende Konfiguration,
Zertifikatswechsel, abweichende registrierte Zeiten, falsche Installation/Aussteller, aktuellen Widerruf und Verbindungsende.
Ein getrennter Datenbankzeuge bestätigt abgewartete Fehlertransaktionen.
Die Laufzeitrolle muss weiterhin verweigern. Die Gruppe läuft vor und nach Datenbankneustart.
Es gibt keine neue Migration. Der Testbestand verwendet die vorhandenen Identitätsmigrationen 1 bis 9.
Das zusätzliche HTTPS-Browserprofil, produktiver Aussteller und Rollen-/Poolbindung bleiben gesondert.
