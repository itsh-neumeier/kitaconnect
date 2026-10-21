# KC-037 – Entscheidungsregister und verbleibende Prüftermine

Stand: 18.10.2026. Grundlage: Spezifikation v5.9.
Issue: [KC-037](https://github.com/itsh-neumeier/kitaconnect/issues/37).
Dieses Register trennt bestätigte Ziele, technische Festlegungen und noch fehlende Freigaben.
P0 erstellt keine Laufzeitdienste und erklärt keine ungeprüfte Installation für unterstützt.

## Entscheidungsstand

| ID | Stand und Begründung | Verantwortung und spätester Prüftermin |
|---|---|---|
| O-001 | Einrichtungsschema und getrennte Dienstrechte durch `1A` bestätigt; ADR-014 | KC-005 vor Abnahme der Mandantenisolation |
| O-001-HR | Gesonderter Personalbereich je Träger durch `3A` bestätigt; ADR-016 | KC-015 vor Personalimplementierung |
| O-001-WAHL | Getrennte Wahldatenbanken und Dienstidentitäten durch `4A` bestätigt; ADR-017 | Projektleitung benennt unabhängige Fachperson vor Implementierung des Wahlprotokolls |
| O-002 | Eigenständiges Rust-Gateway durch `2A` bestätigt; ADR-015 | Rechte- und Vertragsnachweise vor Dienstkommunikation |
| O-003 | [ADR-019](architecture/ADR-019_Dienstidentitaeten.md) legt Diensttokens, mTLS und aktuelle Prüfung fest | KC-006/KC-008 und KC-032 prüfen vor Umsetzung und Einsatz |
| O-004 | [ADR-020](architecture/ADR-020_Hintergrundauftraege.md) legt Redis Streams mit PostgreSQL als dauerhafter Quelle fest | KC-028/KC-029 prüfen vor Hintergrundauftragsimplementierung |
| O-005 | Referenzprofil unten ausdrücklich von der Projektleitung bestätigt | KC-034 misst repräsentative Fachabläufe; Ziele sind keine Messergebnisse |
| O-006 | Planungsstände unten recherchiert; tatsächliche Kompatibilität und endgültige Image-Digests noch ungeprüft | KC-032 vor Infrastrukturimplementierung aktualisieren und nachweisen |
| O-007 | Rechteinhaber und juristische Prüfung auf ausdrücklichen Wunsch offen | Projektleitung benennt Verantwortliche und veranlasst Prüfung vor Nutzungsrechtseinräumung oder Softwareveröffentlichung |
| O-008 | Unabhängige Fachpersonen und konkrete rechtliche Betriebswerte auf ausdrücklichen Wunsch offen | Projektleitung verfolgt die Sperren je Datenklasse und Funktion; siehe Tabelle unten |

Die Antwort der Projektleitung vom 18.10.2026 bestätigt das vorgeschlagene Referenzprofil.
Sie lässt Rechteinhaber, juristische Lizenzprüfung und unabhängige Wahlprüfung ausdrücklich offen.
Die Antwort `@itsh-neumeier Timo Neumeier` bestätigt außerdem die vorläufige CODEOWNER-Zuordnung.
Sie ernennt Timo Neumeier nicht automatisch zum Rechteinhaber oder unabhängigen Fachprüfer.
Der anschließende Auftrag erlaubt die autonome technische Konkretisierung innerhalb der bestehenden Sicherheitsgrenzen.

## Bestätigtes Referenzprofil für eine Einrichtung

| Merkmal | Bestätigtes Ziel |
|---|---|
| Einrichtungen | 1 |
| Gleichzeitig angemeldete Benutzer | 50 |
| Gesamte angebotene Anfragelast | 10 Anfragen pro Sekunde |
| Mischung | 80 % Lesen, 20 % Schreiben |
| p95 Lesen | Höchstens 500 Millisekunden |
| p95 Schreiben | Höchstens 1.000 Millisekunden |
| Technische Fehlerrate | Unter 1 % |
| Referenzressourcen | 4 virtuelle CPU-Kerne, 8 GB RAM, 100 GB SSD |
| Aufwärmzeit | 5 Minuten |
| Messzeit nach Aufwärmen | 30 Minuten |
| Medienverarbeitung | Gesondertes Szenario; keine erfundene Leistungszusage |

p95 bezeichnet den Wert, den mindestens 95 % der gemessenen Antwortzeiten nicht überschreiten.
Die Ressourcen bilden das Referenzbudget der gesamten Kerninstallation einschließlich Datenbank und Redis.
Sie sind noch keine nachgewiesenen Mindestanforderungen für jede Installation.
Die früheren Ziele für Datenverlustzeitraum und Wiederherstellungszeit bleiben jeweils 24 Stunden.

Für den reproduzierbaren Referenzlauf wird als technischer Testbestand festgelegt:
50 synthetische Konten, 100 synthetische Kinderreferenzen, 2 Gruppen und 10.000 Anwesenheitsereignisse.
Diese Mengen sind keine fachlichen Obergrenzen. P0 erzeugt dafür noch keine Fachdaten.
KC-034 erzeugt den Bestand aus einem versionierten Generator nach Implementierung der erforderlichen Abläufe.
Die genaue Rechtezuordnung und Verteilung je Operation gehören zum versionierten Szenario.

## Reproduzierbares Messverfahren für KC-034

1. Verwenden Sie ausschließlich synthetische Konten und Fachdaten einer Einrichtung.
   Legen Sie Datensatzgröße, Rollen, Gruppen und Berechtigungsmatrix vor dem Test versioniert fest.
2. Dokumentieren Sie Commit, Betriebssystem, CPU-Modell, Containerressourcen, Softwareversionen,
   Datenbankparameter, Datensicherungskonfiguration und Ausgangsdaten.
3. Führen Sie den Lastgenerator auf einer getrennten Maschine im gleichen lokalen Netz aus.
   Erfassen Sie die Netzbedingungen. Die Messung ist keine Zusage für beliebige Internetverbindungen.
4. Nutzen Sie 50 aktive Sitzungen und eine insgesamt begrenzte Last von 10 Anfragen pro Sekunde.
   Die Mischung umfasst 8 Lese- und 2 Schreibanfragen pro Sekunde über die bestätigten Fachschnittstellen.
5. Wählen Sie berechtigte Kernabläufe, beispielsweise Kinderlisten, Speisepläne und Anwesenheitsereignisse.
   Versionieren Sie die genaue Verteilung pro Operation nach deren Implementierung.
6. Messen Sie am HTTP-Zugang einschließlich Gateway, aktueller Rechteprüfung und Datenbankzugriff.
   Führen Sie Hintergrundaufträge und Medienverarbeitung als gesonderte Messungen.
7. Verwerfen Sie die ersten 5 Minuten als Aufwärmzeit. Werten Sie danach 30 Minuten aus.
   Die Messzeit ergibt 18.000 angebotene Anfragen, davon 14.400 Lese- und 3.600 Schreibanfragen.
8. Werten Sie p95 getrennt für Lesen und Schreiben aus. Erfassen Sie zusätzlich Ergebnisse je Operation.
   Timeouts, Verbindungsfehler und unerwartete HTTP-Fehler zählen als technische Fehler.
9. Erfassen Sie Datenbankwartezeiten, Hintergrundauftragsrückstand, CPU, RAM und freien Speicher.
   Prüfen Sie nach Testende die erwartete Datenintegrität und den Abschluss angenommener Aufträge.

Ein Verlust oder Abbruch angebotener Anfragen darf nicht durch eine niedrigere tatsächliche Last verdeckt werden.
Gezielte negative Berechtigungsprüfungen werden getrennt ausgewertet.
Lastziele erlauben keine Reduzierung von Zugriffsprotokollierung, Widerruf oder Datenschutzprüfungen.
In P0 gibt es noch keine repräsentativen Fachdienste. Daher werden keine Latenzwerte als gemessen angegeben.

## Recherchierte Planungsstände für O-006

Diese Stände wurden am 18.10.2026 in Primärquellen geprüft.
Sie bilden eine nachvollziehbare Planungsbasis und keine getestete Freigabematrix.
Vor Verwendung werden Patchstand, Sicherheitsmeldungen, Lizenz und Artefakt-Digest erneut geprüft.

| Bestandteil | Recherchierter Stand | Noch notwendiger Nachweis |
|---|---|---|
| PostgreSQL | 18.6, unterstützte Hauptversion 18 | SQLx, RLS, Migrationen, Datensicherung und Wiederherstellung gemeinsam prüfen |
| Redis | 8.10.2, als stabile Version veröffentlicht | Streams, Dienstrechte, Ausfallverhalten und konkrete Lizenzwahl prüfen |
| Helm | 4.3.1, als stabile Version veröffentlicht | Chart mit gewählten Kubernetes-Versionen linten, rendern und installieren |
| Kubernetes | 1.37.1, als stabile Version veröffentlicht | Unterstützungszeitraum, Clusteranbieter, Speicher und Versionsabweichungen prüfen |
| Docker Compose / Portainer | Kein konkreter Patchstand festgelegt | Gemeinsames Installationsprofil und unterstützte Versionen in KC-032 nachweisen |
| Dateispeicher | Lokales Dateisystem und S3-kompatible Schnittstelle nach v5.9 | Konkreten Anbieter oder lokalen Betrieb, Rechte, Lizenz und Wiederherstellung in KC-032 prüfen |

Kubernetes und Helm sind ein zusätzliches Betriebsprofil. Sie ersetzen das vereinbarte Compose-/Portainer-Profil nicht.
Eine Vorabversion wird nicht allein wegen ihrer höheren Versionsnummer gewählt.
Die Lizenzwahl für Redis ist noch nicht rechtlich freigegeben. Das Register behauptet keine Lizenzkompatibilität.
Vor Aufnahme von Images oder Drittkomponenten in veröffentlichte Artefakte ist die Lizenzprüfung verpflichtend.

Quellen:

- [PostgreSQL-Versionierung](https://www.postgresql.org/support/versioning/).
- [Redis 8.10.2](https://github.com/redis/redis/releases/tag/8.10.2).
- [Redis-Lizenzinformationen](https://redis.io/legal/licenses/).
- [Helm 4.3.1](https://github.com/helm/helm/releases/tag/v4.3.1).
- [Kubernetes 1.37.1](https://github.com/kubernetes/kubernetes/releases/tag/v1.37.1).

## Ausdrücklich zurückgestellte Verantwortung und Freigaben

Die Projektleitung hat Rechteinhaber, juristische Prüfstelle und unabhängige Wahlprüfung ausdrücklich offen gelassen.
Für diese drei Rollen werden die spätesten Prüftermine nachfolgend festgehalten.
Weitere bisher offene Fachprüfungen bleiben gemäß v5.9 verbindlich.
Keine dieser Zurückstellungen ist eine erteilte Fachfreigabe.

| Offener Gegenstand | Gesperrte Aktion bis zum Nachweis | Spätester Termin / zuständiges Issue |
|---|---|---|
| Rechteinhaber und proprietärer Lizenztext | Keine neuen Nutzungsrechte zusagen oder Softwareartefakte veröffentlichen | Vor Nutzungsrechtseinräumung und vor Veröffentlichung in KC-035; Projektleitung bleibt zuständig |
| Juristische Prüfung von Drittkomponenten | Keine ungeprüfte Lizenzkompatibilität behaupten oder Komponenten zur Veröffentlichung freigeben | Vor Auswahl und Aufnahme in Artefakte in KC-032/KC-035 |
| Unabhängige Wahlprüfung | Wahlprotokoll nicht implementieren | Vor Implementierung in KC-025; ADR-017 bleibt verbindlich |
| Datenschutzverantwortliche und Datenkategoriefristen | Keine personenbezogenen Echtdaten verarbeiten und keine Fristen erfinden | Vor Umsetzung der betroffenen Lösch-/Aufbewahrungsregel; spätestens vor Produktivbetrieb |
| Regionale Personal- und Betreuungsregelwerke | Keine rechtlich ungeprüfte Regel als verbindlich anwenden | Vor Implementierungsabnahme in KC-016/KC-017 |
| Rechtswirkung digitaler Verträge | Keine Rechtswirksamkeit behaupten oder ungeprüften Vertrag produktiv verwenden | Vor Freigabe der Vertragsfunktion in KC-012 |
| Sicherheits-/Datenschutzprüfung und Branchschutz | Keine P0-Gesamtabnahme oder eigenständigen Merges ohne erforderliche Prüfung und Auftrag | KC-038; für genehmigende GitHub-Reviews gilt die später bestätigte ADR-022 |

Die Dokumentationsprüfung und P0-Arbeit mit synthetischen Daten dürfen weitergehen.
Die bewusste Zurückstellung verändert weder die Spezifikation noch die unabhängigen Prüfpflichten.

Ergänzung vom 18.10.2026: Die Projektleitung benennt Timo Neumeier, `@itsh-neumeier`,
für allgemeine Sicherheits- und Datenschutzprüfungen. Diese Zuständigkeit belegt weder eine erfolgte Prüfung
noch eine unabhängige Fachprüfung. Die später bestätigte Konto-Ausnahme nach ADR-022 betrifft ausschließlich genehmigende GitHub-Reviews.
Die Projektleitung hat persönliche/sprachliche Durchsicht und Integration #39 bis #44 anschließend ausdrücklich bestätigt.
Rechteinhaber, juristische Lizenzprüfung und unabhängige Wahlprüfung bleiben ausdrücklich gesondert offen.
Die fachlich und rechtlich zuständigen Personen müssen vor den gesperrten Aktionen benannt sein.
