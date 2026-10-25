# ADR-055: Begrenzter Mitgliedschaftsverbraucher

- Datum: 25.10.2026
- Status: Technische Konkretisierung vor Code
- Umfang: KC-006F / #67; interner lesender HTTP-Verbraucher

## Entscheidung

Der [Dienstvertrag](../../contracts/identity-access/membership-consumer-v1.md) beschreibt den festen Transport
und die Antwortprüfung im Identitätsdienst vor Code.
Das bestehende Transportpaket erhält einen Client ausschließlich für die Mitgliedschaftsroute.
Die tatsächliche TLS-Grundlage wird gemeinsam verwendet. Ein vertrauenswürdig konfigurierter Serverfingerabdruck
wird vor dem geheimen Anfrageheader geprüft. Der Client öffnet keine beliebigen HTTP-Ziele.
Der Identitätsdienst prüft das vollständige Antwortformat und die genaue Anfragenzuordnung.
Er erhält keine normale Abhängigkeit vom Organisationsdienst und liest keine fremden Tabellen.

## Begründung und Grenzen

Die bestehende Route hat tatsächliche Dienst- und Datenbanknachweise.
Ein begrenzter Verbraucher ergänzt diesen Pfad ohne neue fachliche Zugriffsregel.
Der äußere Aufruf begrenzt den TCP-Verbindungsaufbau. Der Transport begrenzt den weiteren Lebenszyklus.
Ein Ergebnis bleibt ein Datenstand. Fehlende Mitgliedschaft wird nicht durch Voreinstellungen ersetzt.
Fehlende Rechte-/MFA-Daten werden nicht erfunden. OrganizationAuthority folgt gesondert.
Es entstehen keine Migration, produktive Rolle oder erlaubende Policy.
Der Vertrag steht vor abhängigem Code. Produktiver Zertifikatsbetrieb und Gesamtabnahme bleiben offen.

## Präzisierung vor Korrektur des Fehlertransports

Der Router schließt Fehlerantworten ausdrücklich. Eine vollständig empfangene begrenzte Fehlerantwort
darf nach diesem Abschluss ihre feste Ablehnungsklasse behalten.
Ein vorzeitig abgebrochener Body ist weiterhin ein Prüfausfall.
Positive Antworten verlangen erneut den offenen Prüfzugang; ein abgeschlossener Verbindungsbetrieb
oder angekündigter Abschluss darf kein positives Transportergebnis erzeugen.
Diese Präzisierung steht vor der abhängigen Transportkorrektur und erteilt keine Berechtigung.
