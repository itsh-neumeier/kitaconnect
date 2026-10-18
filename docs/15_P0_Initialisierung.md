# P0 – Technische Projektinitialisierung

Stand: 18.10.2026. Grundlage: Spezifikation v5.9.
Status: Grundgerüst zur Prüfung. P0 ist noch nicht vollständig abgenommen.

## Auftrag und Grenze

P0 bereitet Struktur, Werkzeugketten, Tests, Sicherheitsprüfungen und Entscheidungen vor.
Es gibt keine Identitätsverwaltung, Fachendpunkte, Datenbankmigrationen oder Veröffentlichung.
Die erste öffentliche stabile Version bleibt 1.0 mit dem vollständigen vereinbarten Umfang.
Die Dateien `01` bis `14` behalten ihre fachlichen Aussagen.

Vor Beginn wurden README, AGENTS, CLAUDE und alle vorhandenen Dokumente gelesen.
GitHub entsprach dem lokalen Ausgangscommit `a6fae87`.
Die GitHub-API lieferte am 18.10.2026 keine bestehenden Issues oder Pull Requests.
Der CSV-Backlog war deshalb die Importgrundlage. GitHub bleibt danach verbindlich.

## Monorepo-Struktur

| Pfad | Verantwortung | Stand in P0 |
|---|---|---|
| `services/` | Rust-Microservices mit eigenen Paketen | Nur Werkzeugketten-Grundgerüst `bootstrap` |
| `crates/` | Gemeinsame technische Bibliotheken | Keine gemeinsame Fachdomäne |
| `apps/web/` | SvelteKit, Svelte 5, TypeScript, Tailwind | Interne Hinweisseite |
| `contracts/` | OpenAPI 3.1 und Ereignisse je Dienst | Ablage- und Prüfregel |
| `infra/compose/` | Docker Compose und Portainer | Keine Dienste |
| `infra/helm/kitaconnect/` | Kubernetes/Helm | Chart ohne Laufzeitressourcen |
| `tests/fixtures/` | Synthetische Testdaten | Erfundene Organisationskennungen |
| `scripts/` | Struktur- und Dokumentverweisprüfung | Ausführbar mit Node.js |
| `.github/` | Automatisierte Integration und Vorlagen | P0-Prüfschritte ohne Veröffentlichungsrechte |
| `docs/ai-memory/` | Status und Fortsetzungsanleitung | Nachweise und offene Aufgaben |

## Dienstgrenzen als Entwurf

Diese Zuordnung konkretisiert die Skizze aus `02_Technische_Architektur.md`.
KC-002 MUSS Datenhoheit, Dienstrollen, Tabellenzuordnung und erlaubte Aufrufe bestätigen.
Bis dahin entstehen keine Fachpakete oder Datenbanktabellen.
Kein Dienst darf Tabellen eines anderen Dienstes direkt lesen oder ändern.
Die zusätzliche RLS-Prüfung bleibt verbindlich. RLS bedeutet Row-Level Security.

| Geplanter Dienst | Zuständigkeit | Sicherheitsgrenze |
|---|---|---|
| `gateway` | API-Zugang und Frontend-Schnittstelle | Kein Berechtigungs-Bypass; Laufzeitgrenze offen |
| `identity-access` | Identitäten, Anmeldung, Sitzungen, Rechteentscheidung | Plattformdaten ohne Kinderakte |
| `organization-config` | Träger, Einrichtungen, Mitgliedschaften, Konfiguration | Keine impliziten Querzugriffe |
| `children-admissions` | Kinderakte, Sorgeberechtigung, Aufnahme, Verträge | Einrichtungs-, Objekt- und Feldrechte |
| `attendance-care` | Anwesenheit, Abholung, Tagespflege, Übergabe | Gesundheitsdaten und Abholrechte |
| `staff-scheduling` | Personal, Einsatz, Dienstplan, Arbeitszeit | Begrenzte Träger-/Einrichtungsrechte |
| `portfolio-media` | Portfolio, Entwicklungsakte, Medienfreigabe | Vertrauliche Akte getrennt von Veröffentlichungen |
| `media-worker` | Lokale Maskierung und Metadatenentfernung | Isolierter Hintergrunddienst ohne Identitätserkennung |
| `meals` | Speisepläne, Bestellung, Catereransicht | Kostformenrechte; keine Abrechnung |
| `communication-events` | Kommunikation, Kalender, Veranstaltungen | Nur freigegebene Zielgruppen |
| `council-bazaar-cash` | Elternbeirat, Basar, Kasse, QR, Wissen, Inventar | Getrennte Finanzen; atomare Buchungen |
| `election` | Wahlberechtigung, geheime Stimmabgabe, Auszählung | Identität und Stimme ohne rückverfolgbaren Zusammenhang |
| `workflow-notification` | Formulare, Abläufe, Aufträge, Zustellung | Nur nötige Auftragsdaten und Rechte |
| `audit-privacy` | Zugriffsprotokoll, Datenschutzanfragen, Löschjournal | Getrennte Aufbewahrung ohne fachlichen Vollzugriff |

## Offene Entscheidungen

Offene Punkte sind keine Freigaben. Die Projektleitung benennt zuständige Fachpersonen.
KC-037 verfolgt die Entscheidungen. KC-038 verfolgt GitHub-Regeln und Zuständigkeiten.

| ID | Fehlende Entscheidung | Benötigter Nachweis / Zeitpunkt |
|---|---|---|
| O-001 | Datenhoheit und Migrationen bei Schema je Mandant und mehreren Diensten | Tabellen-/Rollenmatrix, Migrationszuständigkeit, Pool- und RLS-Negativtests vor KC-005 |
| O-002 | Laufzeitgrenze von Gateway und Frontend-Schnittstelle | Eigener Dienst oder SvelteKit-Server; Aufrufe und Rechteprüfung vor P1 bestätigen |
| O-003 | Kurzlebige Dienstidentitäten und Transportabsicherung | Aussteller, Laufzeit, Widerruf und Secret Store vor Dienstkommunikation |
| O-004 | Redis-Zustellung und Wiederanlauf | Queue-Verfahren, Zustellbestätigung, Fristen und Fehlerablage vor Hintergrundaufträgen |
| O-005 | Lastprofil und Referenzhardware | Benutzerzahl, p95-Antwortzeit in ms, Fehlerrate, Datenmenge, CPU/RAM/Speicher und Testdauer vor P0-Abnahme |
| O-006 | Datenbank-, Redis-, Speicher- und Plattformversionen | Kompatibilitätsmatrix und Patchpflege vor Infrastrukturimplementierung |
| O-007 | Proprietärer Lizenztext und Rechteinhaber | [Lizenzentwurf](16_Lizenzentwurf.md) fachlich und rechtlich freigeben |
| O-008 | Fachprüfer und Betriebswerte | Zuständige Personen, rechtliche Prüfungen, Schlüsselhoheit, Löschfristen und Abnahmetermine bestätigen |

Die Projektleitung hat in dieser Sitzung `1` zum Referenzprofil angegeben.
P0 führt dies als eine Einrichtung. Die übrigen Lastwerte bleiben offen.
RPO und RTO bleiben gemäß v5.9 jeweils Ziel 24 Stunden.
RPO bezeichnet den maximalen Datenverlustzeitraum. RTO bezeichnet die Wiederherstellungszeit.
Ohne Fachabläufe sind noch keine repräsentativen Messungen möglich.
P0 MUSS das Messverfahren festlegen. KC-034 führt später die Lastprüfung aus.

## Festgestellte Planungskonflikte

1. KC-001 fordert CODEOWNERS. Zuständige Personen waren nicht benannt.
   Die Zuordnung wird angefordert. KC-038 verfolgt Bestätigung und Schutzregeln.
2. P0 soll Messwerte und Hardwaregrößen festlegen. Eine repräsentative Anwendung fehlt.
   O-005 trennt bestätigte Ziele von späteren Messungen.
3. Die CSV-Abhängigkeiten bilden die Phasenbedingungen nicht vollständig ab.
   Die verfeinerte Planung ergänzt Voraussetzungen. Sie verkürzt keine Phase.
4. Historische Ausschlüsse in `01` nennen Abwesenheit und Abholung.
   Die späteren ausdrücklichen Entscheidungen und ADR-005 gelten unverändert.
5. Ereignisbeispiele in `04` nennen eine Korrelations-ID. Diese darf gemäß `02` und `10`
   keine Verbindung zwischen Wähler und Stimme ermöglichen. KC-025 MUSS auch
   Zeitkorrelation, Zugriffsprotokolle und Datensicherungen unabhängig prüfen lassen.

## Entwicklungsreihenfolge

1. KC-001: Monorepo, Werkzeugketten, Build und Infrastrukturstruktur prüfen.
2. KC-002: Dienstgrenzen und O-001/O-002 als Architekturentscheidungen bestätigen.
3. KC-003: Testdaten und P0-Prüfschritte abschließen. Vor externen Rust-Abhängigkeiten
   MUSS `cargo audit` als Prüfung gegen Sicherheitsmeldungen eingerichtet werden.
4. KC-037 und KC-038: Ziele, Verantwortliche, Lizenzfragen und GitHub-Regeln klären.
5. KC-036: Bestandsdokumentation in kleinen Pull Requests sprachlich prüfen.
6. P0 mit vollständigen Nachweisen abnehmen. Erst ein neuer Auftrag erlaubt P1.

P1 bis P9 bleiben nach [Roadmap](06_Entwicklungsroadmap.md) verbindlich.
Die Issues sind Planungscontainer. Vor Implementierung werden breite Module in
Arbeitspakete von 0,5–2 Tagen mit Verträgen und negativen Berechtigungstests zerlegt.
Zugriffsprotokoll und Datenschutz sind Voraussetzungen für sensible Fachdaten.

## Automatisierte Prüfungen und Grenzen

P0 prüft Struktur, lokale Dokumentverweise, Backlog-Abhängigkeiten, Rust-Formatierung,
Clippy, Build/Test, Svelte-Prüfung, Frontend-Build, Secrets und npm-Sicherheitsmeldungen.
Actions sind auf vollständige Commit-IDs festgelegt. Gitleaks nutzt eine feste Prüfsumme.
Workflow-Tokens erhalten nur Leserechte. npm-Installationsskripte sind deaktiviert.
Die Abhängigkeitsprüfung umfasst auch Frontend-Buildwerkzeuge.

P0 hat keine externen Rust-Abhängigkeiten. Ein Prüfschritt blockiert deren Einführung,
bis KC-003 `cargo audit` einrichtet. Dies ersetzt keine spätere Sicherheitsprüfung.
Quelltext-Sicherheitsanalyse, Containerprüfung, Lizenzprüfung, Software-Stückliste,
Signaturen, Mandantenisolation und Wiederherstellung folgen den Phasen aus `08`.
P0 veröffentlicht keine Tags oder Images.

Main MUSS ein unabhängiges Review und die Pflichtprüfungen
`structure`, `rust`, `web`, `secrets`, `dependencies` verlangen.
CODEOWNERS allein erzwingt diese Regeln nicht. Die GitHub-Regelsatzabfrage war leer.
Der API-Nachweis tatsächlicher Schutzregeln bleibt Teil von KC-038.
Die separate Abfrage des bisherigen Branchschutzes lieferte HTTP 403.
Der Connector darf diese Administrationseinstellung nicht lesen.
Deshalb ist der bisherige Branchschutz unbekannt.
Die CODEOWNERS-Datei enthält bis zur Bestätigung nur Hinweise und keine Zuordnung.
Die manuelle sprachliche Prüfung bleibt vor dem Merge erforderlich.

## P0-Abnahme

- [ ] Builds und P0-Prüfschritte sind für den aktuellen Commit grün.
- [ ] KC-002 bestätigt Dienstgrenzen, Datenhoheit und Architekturentscheidungen.
- [ ] O-001 bis O-008 sind entschieden oder haben einen ausdrücklich bestätigten späteren Prüftermin.
- [ ] Lastprofil und Messverfahren sind bestätigt; fehlende Messungen sind gekennzeichnet.
- [ ] CODEOWNERS und Branchschutz sind bestätigt und nachgewiesen.
- [ ] Lizenzentwurf und Dokumentationsprüfung haben den geforderten Prüfstatus.
- [ ] Checkpoint, GitHub-Issues und Pull Request stimmen überein.
