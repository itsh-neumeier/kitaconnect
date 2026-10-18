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

Nächster Schritt: [KC-002](https://github.com/itsh-neumeier/kitaconnect/issues/2).
Entscheidungen zu Dienstgrenzen und Datenhoheit zuerst vorbereiten.
KC-037 und KC-038 verfolgen die verbleibenden P0-Fragen.
