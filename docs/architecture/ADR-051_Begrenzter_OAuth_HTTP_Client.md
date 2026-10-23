# ADR-051: Begrenzter OAuth-HTTP-Client

- Datum: 22.10.2026
- Status: Technische Konkretisierung vor Code
- Umfang: KC-006F, tatsächlicher Transport zu vorhandenen OAuth-Endpunkten

## Entscheidung

Der [Clientvertrag](../../contracts/identity-access/service-http-client-v1.md) ergänzt das vorhandene Transportpaket.
Ein Aufruf besitzt genau eine tatsächliche mTLS-Verbindung und begrenzte Formular-, Header- und Antwortdaten.
Die Verbindungsoperation läuft innerhalb desselben Aufrufs. Abbruch hinterlässt keinen losgelösten Auftrag.
Pfad und Methode sind auf die vorhandene OAuth-Schnittstelle begrenzt. Weiterleitungen sind ausgeschlossen.
Zeitlimits und Parallelität sind ausdrücklich konfiguriert; produktive Voreinstellungen entstehen nicht.

## Begründung und Grenzen

Die nachfolgenden Organisationsaufrufe benötigen aktuelle Introspektion ohne fremde Datenbankzugriffe.
Der erste Clientbaustein transportiert dafür ausschließlich die bestehenden OAuth-Nachrichten.
Er darf eine HTTP-Antwort nicht in eine Token- oder Benutzerfreigabe umdeuten.
Protokollprüfung und zuständige Dienstrechte bleiben außerhalb des Transportpakets.
Ein eigener Verbindungspool oder Hintergrundauftrag ist für diesen ersten begrenzten Client nicht erforderlich.
Die Mehrkosten neuer Verbindungen gehören vor produktiver Nutzung in das bestätigte Lastprofil.
Produktiver Zertifikatsbetrieb, Serverregistrierung und Fachrechte bleiben eigene Nachweise.
