# Organisationsmodell v1 – interner Rust-Vertrag

Stand: 18.10.2026. Issue: [KC-004A / #50](https://github.com/itsh-neumeier/kitaconnect/issues/50).
Dieser Vertrag wird vor der Implementierung des Modells festgelegt.

## Zweck und Daten

Der Dienst `organization-config` besitzt das Organisationsmodell gemäß ADR-014.
Das Modell enthält ausschließlich interne Kennungen für Träger und Einrichtungen.
Eine Einrichtung verweist auf einen registrierten Träger oder ist unabhängig.
Träger- und Einrichtungskennungen sind verschiedene Rust-Typen. Der Wert null ist ungültig.
Die interne Darstellung verwendet positive 128-Bit-Werte. Sie legt kein öffentliches ID-Format fest.
ID-Erzeugung, Namen, Adressen, Mitgliedschaften und weitere Organisationshierarchien folgen gesondert.

## Operationen und Fehler

| Operation | Voraussetzung | Ergebnis |
|---|---|---|
| Träger registrieren | Kennung noch nicht registriert | Träger vorhanden |
| Einrichtung registrieren | Kennung frei; gewählter Träger vorhanden | Einrichtung mit unveränderlicher Zuordnung vorhanden |
| Einrichtung nach Kennung lesen | Interner Aufrufer | Modellwert oder kein Treffer |
| Trägerbestand prüfen | Interner Aufrufer | Wahrheitswert |

Doppelte Trägerkennungen, doppelte Einrichtungskennungen und unbekannte Träger sind Fehler.
Eine fehlgeschlagene Operation verändert den Modellzustand nicht.
Eine doppelte Einrichtungskennung wird zuerst geprüft. Sie ersetzt niemals eine bestehende Zuordnung.
Das Modell bietet keine Übertragung, Löschung oder automatische Rechtevererbung an.
Dies legt keine fachliche Transferregel fest. Eine solche Operation benötigt einen eigenen geprüften Vertrag.

## Sicherheitsgrenze

Die Bibliothek ist eine interne Modellkomponente ohne Netzwerk, Datenbank oder Benutzeridentität.
Ihre Methoden sind keine öffentlichen Dienstendpunkte und prüfen keine Benutzerberechtigung.
Eine vorhandene Kennung oder Trägerzuordnung erteilt kein Zugriffsrecht.
Das Modell darf erst hinter aktueller Identitäts- und Rechteprüfung als Dienst verfügbar werden.
KC-004B (#51) ergänzt Persistenz und Migrationstests.
KC-004C (#52) ergänzt Dienstvertrag, aktuelle Rechteprüfung und negative Tests fremder Trägerzuordnung.
KC-004 bleibt bis zu diesen Nachweisen offen.

## Prüffälle

1. Zwei Träger, jeweils eine Einrichtung und eine unabhängige Einrichtung gleichzeitig anlegen.
2. Gleiche numerische Kennung in unterschiedlichen ID-Typen ohne Vermischung verwenden.
3. Nullkennungen abweisen; größte gültige Kennung unverändert erhalten.
4. Doppelte Träger und Einrichtungen ohne Zustandsänderung abweisen.
5. Unbekannten Träger ohne Zustandsänderung abweisen.
6. Bestehende Einrichtung auch durch erneute Anlage mit anderem oder ohne Träger nicht umordnen.
7. Fremden Kennungstyp beim Aufruf durch den Rust-Compiler abweisen.

Alle Prüfdaten sind synthetisch. Die Tests belegen Modellinvarianten, keine durchgesetzte Benutzerberechtigung.
