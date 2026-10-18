# Ausstehende Entscheidungen

Stand: 18.10.2026. Grundlage: Spezifikation v5.9.

Die vollständige Liste O-001 bis O-008 steht im [P0-Plan](../15_P0_Initialisierung.md).
Die Projektleitung MUSS Entscheidungen ausdrücklich bestätigen.
Eine ausbleibende Antwort ist keine Zustimmung.

| Frage aus dieser Sitzung | Stand |
|---|---|
| Ausnahme für eigene PRs | Auf die Rückfrage mit „Ja genau“ bestätigt: nur eigene PRs von `@itsh-neumeier`, keine Ausnahme für fremde PRs und keine Ausnahme von Pflichtprüfungen. [ADR-022](../architecture/ADR-022_Reviewausnahme_eigene_PRs.md). Nicht erneut nach diesem Umfang fragen. |
| Technischer Weg der Ausnahme | Eingebaute Benutzerausnahme ist zu weitgehend. Vorschlag: gesonderte vertrauenswürdige GitHub-App mit begrenztem Merge-Verfahren. Zusätzliche App und Betriebsweg noch nicht entschieden; bisheriger Schutz bleibt aktiv. |
| Referenzprofil für die Lastprüfung | Eine Einrichtung. 50 Sitzungen, 10 Anfragen pro Sekunde, 80/20 Lesen/Schreiben, p95 500/1.000 ms, unter 1 % Fehler, 4 vCPU, 8 GB RAM, 100 GB SSD, 5 Minuten Aufwärmen und 30 Minuten Messung ausdrücklich bestätigt. |
| CODEOWNER für das Repository | Antwort `@itsh-neumeier Timo Neumeier`; vorläufige Zuordnung in CODEOWNERS eingetragen. Kein Nachweis unabhängiger Prüfung. |
| PR-Prüfung durch Projektleitung und KI | Persönliche Prüfung durch Timo Neumeier und technische Vorprüfung durch den KI-Assistenten ausdrücklich gewünscht. Aktuelle PRs sind über denselben Benutzer erstellt; GitHub-Pflichtgenehmigung bleibt unerfüllt. Keine erfolgte menschliche Prüfung behauptet. Keine Abschwächung der Regeln. |
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
baut auf dem weiterhin offenen Grundgerüst-PR #39 auf.

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
Unabhängige Review-Verantwortung bleibt offen. Der Regelsatz wird dafür nicht abgeschwächt.
P0 implementiert keine Dienste, Datenbankrollen oder Migrationen.

## Bestätigte Zurückstellungen und technische Fortsetzung

Die Projektleitung hat nach den Rückfragen autonome Weiterarbeit beauftragt.
ADR-019 und ADR-020 konkretisieren deshalb Dienstidentitäten und Hintergrundaufträge im bestehenden Sicherheitsumfang.
Das Register hält die recherchierten Infrastrukturstände mit Prüfpflicht vor tatsächlicher Verwendung fest.
Die Versionsmatrix behauptet keine getestete Installation oder rechtliche Freigabe.

Offen bleiben Rechteinhaber, juristische Prüfung und die unabhängigen Fachpersonen.
Die Zurückstellung ist ausdrücklich gewünscht. Sie hebt die Sperren aus dem Register nicht auf.
Vor Implementierung des Wahlprotokolls ist die unabhängige Prüfung weiterhin zwingend.
