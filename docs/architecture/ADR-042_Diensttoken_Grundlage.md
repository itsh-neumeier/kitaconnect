# ADR-042: Diensttoken-Grundlage

- Datum: 21.10.2026
- Status: Technische Konkretisierung der bestätigten ADR-019 vor Code
- Umfang: KC-006F, interner Zustand und getrennte spätere Transportnachweise

## Entscheidung

Der [Dienstvertrag](../../contracts/identity-access/service-tokens-v1.md) trennt internen Tokenbestand und echte mTLS-Prüfung.
Registrierung und begrenzte Zweckfreigabe bleiben außerhalb der Laufzeitverwaltung.
Ein Token bindet Installation, Aussteller, registrierten Dienst, genaues Zertifikat, Zieldienst und Zweck.
Die Ausgabe verwendet vorhandene Zufalls- und Ableitungsbibliotheken; es entsteht kein eigenes Kryptoverfahren.
Aktuelle Prüfung und endgültiger Entzug werden vor der Transportanbindung separat nachgewiesen.

## Begründung

Die Aufteilung erlaubt kleine überprüfbare Schritte und erhält die Grenzen aus ADR-019.
Ein synthetischer vertrauenswürdiger Test-Aufrufer darf nicht mit einer geprüften TLS-Verbindung verwechselt werden.
Eine positive Zustandsabfrage erteilt allein keine Benutzer- oder Fachrechte.
Die spätere Introspektion verwendet gegenseitige Zertifikatsprüfung ohne rekursive Tokenabfrage.

## Folgen und Grenzen

Die Standards, Datenfelder und Nachweise stehen im Vertrag. Die HTTP-/TLS-Auswahl folgt vor ihrem Code getrennt.
Produktive Domain, Zertifikatsaussteller und Schlüsselbetrieb bleiben offen.
Es werden keine privaten Schlüssel, Zertifikate oder produktiven Registrierungen eingecheckt.
Die maximale Token- und Zertifikatslaufzeit stammt bereits aus ADR-019 und ist keine neue Fachentscheidung.
MFA-/Rechtezuordnung, Browserprofil und anonyme Wahlen bleiben gesonderte Grenzen.
