# Ausstehende Entscheidungen

Stand: 18.10.2026. Grundlage: Spezifikation v5.9.

Die vollständige Liste O-001 bis O-008 steht im [P0-Plan](../15_P0_Initialisierung.md).
Die Projektleitung MUSS Entscheidungen ausdrücklich bestätigen.
Eine ausbleibende Antwort ist keine Zustimmung.

| Frage aus dieser Sitzung | Stand |
|---|---|
| Referenzprofil für die Lastprüfung | Antwort `1`; als eine Einrichtung dokumentiert. Benutzerzahl, Antwortzeit und Hardware fehlen. |
| CODEOWNER für Sicherheit, Datenschutz und Repository | Vorschlag `@itsh-neumeier` angefragt; noch nicht bestätigt. CODEOWNERS enthält keine aktive Zuordnung. |
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
O-003 bis O-008 sowie die Benennung der unabhängigen Fachperson bleiben offen.
KC-037 und KC-038 verfolgen diese übrigen P0-Fragen.
P0 implementiert keine Dienste, Datenbankrollen oder Migrationen.
