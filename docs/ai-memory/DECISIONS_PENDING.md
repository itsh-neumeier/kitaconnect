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

## Konkrete Vorschläge für KC-002

Die [Dienstmatrix](../architecture/KC-002_Dienstgrenzen.md) liegt mit Datenhoheit,
Kommunikationszwecken und Testanforderungen als Entwurf vor.
[Pull Request #40](https://github.com/itsh-neumeier/kitaconnect/pull/40)
baut auf dem weiterhin offenen Grundgerüst-PR #39 auf.

| Entscheidung | Vorschlag und Stand |
|---|---|
| O-001 / ADR-014 | Ein PostgreSQL-Schema je Einrichtung mit getrennten Dienstrechten und Migrationsrollen. Alternative: eigene Datenbank je Dienst. Entscheidung angefragt; ausstehend. |
| O-002 / ADR-015 | Eigenständiges Rust-Gateway. Alternative: SvelteKit als Backend für das Frontend. Entscheidung angefragt; ausstehend. |
| O-001-HR | Speicher- und Rechtebereich trägerweiter Personalakten ausdrücklich offen. Die allgemeine Datenbankentscheidung entscheidet diesen Punkt nicht. |
| O-001-WAHL | Physische Trennung von Wahlberechtigung und Stimme ausdrücklich offen. Unabhängige Fachprüfung erforderlich. |

Nächster Schritt: Die Projektleitung entscheidet die beiden angefragten Varianten.
KC-002 bleibt bis zu den erforderlichen Sicherheitsentscheidungen offen.
Es werden keine Dienste, Datenbankrollen oder Migrationen implementiert.
KC-037 und KC-038 verfolgen die übrigen P0-Fragen.
