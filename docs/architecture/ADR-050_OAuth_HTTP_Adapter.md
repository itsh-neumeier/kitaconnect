# ADR-050: Tatsächlicher OAuth-HTTP-Adapter

- Datum: 22.10.2026
- Status: Technische Konkretisierung der bestehenden Dienstidentitätsentscheidung vor Code
- Umfang: KC-006F, interne HTTP-/TLS-/Datenbankanbindung

## Entscheidung

Der [HTTP-Dienstvertrag](../../contracts/identity-access/service-http-v1.md) verbindet Axum und Hyper mit tatsächlichem mTLS.
Jede Routerinstanz erhält ausschließlich den Prüfzugang ihrer tatsächlich geprüften Verbindung.
Begrenzungen und Zeitlimits sind ausdrücklich übergeben. Der Adapter öffnet keinen Listener und startet keine unbeschränkte Annahme.
Die vorhandenen Formular-, Registrierungs-, Freigabe-, Token- und Zeitprüfungen bleiben getrennte Schritte.
Neue Softwareabhängigkeiten werden exakt gebunden und vollständig auditiert.

Jede Anfrage besitzt zunächst eine frische eigene Datenbankverbindung.
Diese wird bei Abbruch physisch verworfen und kann nicht mit ungeklärtem Transaktionszustand in einen Pool zurückkehren.
Ein produktiver Pool wurde in diesem Schritt wegen der noch offenen Rollen-/Poolbindung nicht gewählt.
Die Mehrkosten gehören vor produktiver Nutzung in das bestätigte Lastprofil.

Der tatsächliche Sperrtest zeigt, dass Socketfreigabe allein einen wartenden Datenbankprozess nicht zuverlässig sofort beendet.
Die übergebenen Verbindungsoptionen erhalten deshalb zusätzlich Anweisungs-, Transaktions- und Leerlaufgrenzen aus derselben Anfragefrist.
Der Adapter verwendet die vorhandene PostgreSQL-18-Grundlage. Er ändert weder Datenbankrollen noch globale Servervorgaben.
Der Prozessnachweis wird nicht abgeschwächt. Diese Präzisierung steht vor der zugehörigen Korrektur.

## Fehler und Grenzen

Der Adapter verwendet die bereits bestätigte OAuth-Fehlerformatausnahme.
HTTP 401 bei versuchter Headerauthentisierung erhält die protokollgemäße Herausforderung; es entsteht kein alternatives Anmeldeverfahren.
Die Introspektion beschränkt sich auf die schon vorgesehenen Zieldienste.
Momentaufnahmen und vorläufige Freigabeabfragen ersetzen keine autoritative Tokenprüfung.
Es gibt keine neue Migration, Datenbankrolle oder erlaubende Laufzeitpolicy.
Tests mit synthetischer Administration sind keine produktive Berechtigungsfreigabe.
Browser, Gateway, Organisation und Gesamtabnahme bleiben getrennte Nachweise.
