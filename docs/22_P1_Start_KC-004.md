# P1 – Start mit KC-004

Stand: 18.10.2026. Grundlage: Spezifikation v5.9.
Auftrag der Projektleitung: „P0 abnehmen, und P1 freigeben“.
P0-Abschlussnachweis #49 ist über `eaca2cb` integriert. Alle fünf Main-Pflichtprüfungen sind erfolgreich.
KC-038 ist abgeschlossen. Die zurückgestellten Fachprüfungen behalten ihre Sperren.

## Kleine Arbeitspakete

| Reihenfolge | Issue | Ergebnis | Abhängigkeit |
|---|---|---|---|
| 1 | [KC-004A / #50](https://github.com/itsh-neumeier/kitaconnect/issues/50) | Interner Rust-Modellvertrag, getrennte Kennungen, Träger und Einrichtungen, negative Modelltests | Abgenommenes P0 |
| 2 | [KC-004B / #51](https://github.com/itsh-neumeier/kitaconnect/issues/51) | Persistenz, dienstbezogene Datenbankrechte und Migrationstests | Modell; Abstimmung mit KC-005 |
| 3 | [KC-004C / #52](https://github.com/itsh-neumeier/kitaconnect/issues/52) | Aktuelle Rechteprüfung und negative Diensttests fremder Trägerzuordnung | Persistenz; Identitäts-/Policy-Grundlagen aus KC-006/KC-008 |

Jedes Paket benötigt einen eigenen prüfbaren PR. KC-004 bleibt als Gesamtissue offen.
Die weitere P1-Reihenfolge folgt dem verfeinerten Backlog und der Roadmap.
KC-005 und KC-006 können nach dem Organisationsgrundmodell vorbereitet werden.
Ihr technischer Kontext darf nicht von einer bereits öffentlich verfügbaren Organisationsverwaltung abhängen.
KC-008 ergänzt danach die gemeinsame Policy-Prüfung. KC-004C verwendet diese Prüfung vor dem Speichern.

## Umfang des ersten Pakets

Die Bibliothek liegt im zuständigen Dienst `services/organization-config/`.
Der [interne Vertrag](../contracts/organization-config/model-v1.md) wurde vor der Implementierung versioniert.
[ADR-023](architecture/ADR-023_Organisationsmodell.md) begründet Paketgrenze und Kennungstypen.
Das Modell erfasst zwei Träger mit jeweils eigener Einrichtung und eine unabhängige Einrichtung.
Es weist doppelte Kennungen und unbekannte Träger ohne Zustandsänderung ab.
Eine erneute Registrierung verändert keine bestehende Zuordnung.

Das Paket ist eine interne Bibliothek ohne Netzwerk oder Speicherung.
Die Tests belegen Modellinvarianten. Sie belegen keine Benutzerberechtigung oder Datenbankisolation.
Öffentliche Endpunkte, Mitgliedschaften, Transfers und Berechtigungsvergabe werden hier nicht eingeführt.
Die fachliche Bedeutung weiterer Organisationsebenen bleibt vor deren Umsetzung zu konkretisieren.

## Prüfung und Abnahmegrenze

Die vorhandene Workspace-CI erfasst das neue Rust-Paket mit Formatierung, Clippy, Tests und Build.
Acht Rust-Integrationstests prüfen das Modell. Ein negativer Dokumentationstest prüft die Trennung der Kennungstypen.
Die Bibliothek hat keine externen Abhängigkeiten. Alle Kennungen in Tests sind synthetisch.
Windows kann lokal ohne C++-Linker Typprüfung und Clippy ausführen. Ausführbare Tests und Build müssen in der Linux-CI erfolgreich sein.
KC-004 insgesamt wird erst nach Persistenz- und Rechteprüfungen aus #51 und #52 abgeschlossen.
