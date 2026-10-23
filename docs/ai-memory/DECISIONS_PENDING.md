# Ausstehende Entscheidungen

Stand: 18.10.2026. Grundlage: Spezifikation v5.9.

Die vollständige Liste O-001 bis O-008 steht im [P0-Plan](../15_P0_Initialisierung.md).
Die Projektleitung MUSS Entscheidungen ausdrücklich bestätigen.
Eine ausbleibende Antwort ist keine Zustimmung.

| Frage aus dieser Sitzung | Stand |
|---|---|
| Persönliche/sprachliche Durchsicht und Integration | Die Projektleitung hat mit „Ja“ Durchsicht und Integration #39 bis #44 bestätigt. Alle sechs PRs sind integriert; Main-CI grün. Nicht erneut nach dieser Freigabe fragen. |
| P0-Gesamtabnahme und P1 | „P0 abnehmen, und P1 freigeben“ am 18.10.2026 ausdrücklich bestätigt. Nicht erneut abfragen. Abschlussnachweis #49 integrieren; mit KC-004 beginnen. |
| Review-Ausnahme | „Einfache Konto-Ausnahme; Nutzung auf eigene PRs beschränken“ ausdrücklich gewählt. Konto-Ausnahme für `@itsh-neumeier` im gesonderten Review-Regelsatz aktiv. „Nur eigene PRs“ ist eine bestätigte Nutzungsregel; technisch auch bei fremden PRs möglich. [ADR-022](../architecture/ADR-022_Reviewausnahme_eigene_PRs.md). Nicht erneut abfragen. |
| Pflichtprüfungen | Basisregelsatz `25341109` bleibt ohne Ausnahme. Review-Ausnahme steht nur in Regelsatz `25346711`. Beide nach Einrichtung per API nachgewiesen. Keine zusätzliche App oder neue Secrets erforderlich. |
| Referenzprofil für die Lastprüfung | Eine Einrichtung. 50 Sitzungen, 10 Anfragen pro Sekunde, 80/20 Lesen/Schreiben, p95 500/1.000 ms, unter 1 % Fehler, 4 vCPU, 8 GB RAM, 100 GB SSD, 5 Minuten Aufwärmen und 30 Minuten Messung ausdrücklich bestätigt. |
| CODEOWNER für das Repository | Antwort `@itsh-neumeier Timo Neumeier`; vorläufige Zuordnung in CODEOWNERS eingetragen. Kein Nachweis unabhängiger Prüfung. |
| PR-Prüfung durch Projektleitung und KI | Persönliche/sprachliche Prüfung durch Timo Neumeier und technische Vorprüfung durch den KI-Assistenten für #39 bis #44 bestätigt und dokumentiert. Die bestätigte Konto-Ausnahme wurde nur für diese eigenen PRs genutzt. |
| Sicherheits- und Datenschutzprüfung | Antwort „Ich auch als Prüfer“ benennt Timo Neumeier, `@itsh-neumeier`. Keine durchgeführte oder unabhängige Prüfung behauptet. Für eigene Änderungen bleibt unabhängige Gegenprüfung offen. |
| KC-036 / F-001: Tagesereignis | Antwort „Ja“ bestätigt Prüfung vor Speichern. Ablauf in Unterlage 05 klargestellt; F-001 entschieden. [Prüfbericht](../19_Redaktionspruefung_KC-036.md). |
| Rechteinhaber, juristische Lizenzprüfung und unabhängige Wahlprüfung | Antwort `offen lassen`. Projektleitung bleibt für Besetzung vor den gesperrten Aktionen zuständig. Keine automatische Zuordnung zum CODEOWNER. |
| npm audit und externe Paketmetadaten | Die Projektleitung hat die Übermittlung von Paketnamen und Versionen an registry.npmjs.org ausdrücklich erlaubt. Gilt lokal und in GitHub Actions. |

Die ausdrückliche Freigabe für npm audit liegt nach einer automatischen Ablehnung vor.
Die ursprüngliche Ablehnung betraf die Übermittlung von Abhängigkeitsmetadaten.
Sie betraf weder Quelltext noch produktive Zugangsdaten.

## Bestätigte Entscheidungen für KC-002

Die [Dienstmatrix](../architecture/KC-002_Dienstgrenzen.md) liegt mit Datenhoheit,
Kommunikationszwecken und Testanforderungen vor.
[Pull Request #40](https://github.com/itsh-neumeier/kitaconnect/pull/40)
ist ebenso wie der Grundgerüst-PR #39 integriert.

Die Projektleitung hat am 18.10.2026 im Projektchat ausdrücklich `1A, 2A, 3A, 4A` geantwortet.
Diese Antwort bestätigt die vier zuvor einzeln erläuterten Varianten.

| Entscheidung | Bestätigter Stand |
|---|---|
| O-001 / ADR-014 | `1A`: Ein PostgreSQL-Schema je Einrichtung mit getrennten Dienstrechten und Migrationsrollen. |
| O-002 / ADR-015 | `2A`: Eigenständiges Rust-Gateway. |
| O-001-HR / ADR-016 | `3A`: Gesonderter Personalbereich je Träger; Einrichtungen erhalten nur erlaubte Einsatzdaten. Trägerlose Einrichtungen behalten eigene Bereiche. |
| O-001-WAHL / ADR-017 | `4A`: Getrennte Datenbanken und Dienstidentitäten für Berechtigung und Stimme. Unabhängige Prüfung des Wahlprotokolls vor Implementierung bleibt verbindlich. |

Die Varianten sind damit entschieden. Die Implementierungsnachweise folgen in den zugeordneten Issues.
O-003 bis O-008 sind inzwischen im [KC-037-Register](../17_P0_Entscheidungsregister.md) konkretisiert.
Technische Festlegungen und bestätigte Ziele sind von den offenen Fachfreigaben getrennt.
KC-038 hat den [Main-Regelsatz](../18_Branchschutz_und_Reviews.md) eingerichtet und per API nachgewiesen.
ADR-022 dokumentiert die spätere, ausdrücklich bestätigte Konto-Ausnahme nur vom gesonderten Review-Regelsatz.
Fachprüfungen und persönliche Durchsicht bleiben erforderlich. Die Pflichtprüfungen erhalten keine Ausnahme.
P0 implementiert keine Dienste, Datenbankrollen oder Migrationen.

## Bestätigte Zurückstellungen und technische Fortsetzung

Die Projektleitung hat nach den Rückfragen autonome Weiterarbeit beauftragt.
ADR-019 und ADR-020 konkretisieren deshalb Dienstidentitäten und Hintergrundaufträge im bestehenden Sicherheitsumfang.
Das Register hält die recherchierten Infrastrukturstände mit Prüfpflicht vor tatsächlicher Verwendung fest.
Die Versionsmatrix behauptet keine getestete Installation oder rechtliche Freigabe.

Offen bleiben Rechteinhaber, juristische Prüfung und die unabhängigen Fachpersonen.
Die Zurückstellung ist ausdrücklich gewünscht. Sie hebt die Sperren aus dem Register nicht auf.
Vor Implementierung des Wahlprotokolls ist die unabhängige Prüfung weiterhin zwingend.

## Produktive RLS-Bindung nach KC-005B

Stand: 20.10.2026. [ADR-027](../architecture/ADR-027_Dienstisolation_Testaufbau.md) verwendet eigene Loginrollen ausschließlich im Testcontainer.
Vor einer produktiven erlaubenden Policy sind Identitätsnachweis, aktuelle Widerrufsprüfung und Bindung an den Verbindungspool ausdrücklich festzulegen.
Eine frei gesetzte Einrichtungsvariable reicht dafür nicht. Bis zur Entscheidung bleiben produktive Laufzeitfreigaben gesperrt.
KC-005B prüft technische Dienstrechte; KC-006/KC-008 integrieren den Benutzerkontext. Diese offene Entscheidung blockiert keine synthetischen Negativtests.

Der Projektleitung wurde vorgeschlagen: eigene Datenbankidentität und eigener Verbindungspool je Dienst und Einrichtung.
Die Rückfrage wurde im Projektchat gestellt. Eine Antwort steht aus. Ohne Antwort gilt keine Freigabe dieser Produktionsarchitektur.

## KC-006: Angefragte Sicherheitsregeln

Stand: 20.10.2026. Drei Rückfragen wurden im Projektchat gestellt. Bisher liegt keine ausdrückliche Antwort vor.

| Thema | Angefragter Vorschlag | Stand und Sperre |
|---|---|---|
| MFA-Zuordnung | Pflicht für Konten mit Verwaltungsrechten oder Zugriff auf besonders sensible Daten; sonst zunächst optional. Sensible Aktionen benötigen zusätzliche Prüfung. | Offen. Konkrete Zuordnung später über Berechtigungen; keine abhängige Anmeldefreigabe ohne Antwort. |
| Browser-Sitzungen | 15 Minuten Inaktivität, 8 Stunden absolute Laufzeit, zusätzliche Prüfung höchstens 5 Minuten gültig; Träger dürfen verkürzen. | Offen. Projektvorschlag, keine gesetzliche Vorgabe oder bestätigte Mindestregel. Keine eingebauten Vorgabewerte. |
| Wiederherstellung | Einmalige offline aufbewahrte Codes; keine MFA-Rücksetzung allein durch E-Mail. Bestehende Sitzungen nach Wiederherstellung widerrufen und nötige Faktoren neu einrichten. Fälle ohne Nachweis bleiben bis zum geprüften Ersatzverfahren gesperrt. | Offen. KC-006E darf abhängige Abläufe erst nach ausdrücklicher Entscheidung umsetzen. |

KC-006A / #62 kann unabhängig davon nur Kennungstypen und lokale Zeit-/Zustandsregeln mit expliziten Eingaben implementieren.
Die [KC-006-Planung](../24_KC-006_Authentisierung.md) beschreibt Teilpakete und Abnahmegrenzen.
