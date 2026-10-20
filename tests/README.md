# Prüfungen und synthetische Testdaten

`fixtures/organizations.json` enthält erfundene technische Organisationskennungen.
Die Datei enthält keine Kinder, Namen, Kontaktdaten oder aus Echtdaten abgeleiteten Werte.
Sie bildet zwei getrennte Träger und eine unabhängige Einrichtung ab.
Das Lastprofil mit einer Einrichtung ist davon getrennt.

## Testdaten erzeugen und prüfen

Der Generator verwendet ausschließlich festgelegte technische Kennungen.
Er liest keine Importdateien, Umgebungsvariablen oder externen Datenquellen.

1. Führen Sie `npm run fixtures:generate` aus.
2. Prüfen Sie das Ergebnis mit `npm run fixtures:check`.
3. Führen Sie `npm test` aus.

Die Erzeugung MUSS bytegenau reproduzierbar sein. Der Prüfschritt verändert keine Datei.
Neue zulässige Felder benötigen eine ausdrückliche Änderung von Generator und Strukturprüfung.
Fehlende Kennzeichnung, zusätzliche Felder, doppelte Kennungen und unbekannte Träger werden abgewiesen.
Die Daten bilden zwei Träger mit je einer Einrichtung sowie eine trägerlose Einrichtung ab.

## P0-Prüfmatrix

CI bezeichnet die automatisierte Integration über GitHub Actions.
Alle fünf bestehenden Prüfschritte bleiben benannt, damit KC-038 sie als Pflichtprüfungen verwenden kann.

| Prüfschritt | Laufzeit / Eingabe | Ausgeführter Nachweis |
|---|---|---|
| `structure` | Node.js 24.19.0, versionierte Organisationsdaten | Lokale Dokumentverweise, Backlog-Abhängigkeiten, bytegenaue Erzeugung, Branchschutzplan, Struktur- und Negativtests |
| `rust` | Rust 1.99.1, Cargo.lock | Rustfmt, Clippy, Tests und Build mit festgelegtem Lockfile |
| `web` | Node.js 24.19.0, npm 12.2.0, package-lock.json | Installation ohne Paketskripte, Svelte-/TypeScript-Prüfung und Build |
| `secrets` | Gitleaks 8.30.1 | Gesamte Git-Historie; Ausgabe mit verdeckten Fundwerten |
| `dependencies` | npm-Audit und cargo-audit 0.22.2 | npm meldet hohe/kritische Befunde als Fehler; RustSec-Befunde und deren Warnungskategorien blockieren den Rust-Prüfprozess |

Buildprüfungen laufen bei Pull Requests und Pushes. Sicherheitsprüfungen laufen zusätzlich wöchentlich.
Die Workflows haben nur Leserechte. Sie veröffentlichen keine Software und schließen keine Issues.
Die bestehende Erlaubnis für npm audit lokal und in GitHub Actions gilt weiter.

## Rust-Abhängigkeiten prüfen

`cargo-audit` ist als Prüfwerkzeug auf Version 0.22.2 festgelegt.
Die CI prüft die SHA-256-Prüfsumme des offiziellen Archivs vor dem Entpacken.
Der Scanner läuft außerhalb des Checkout-Verzeichnisses und erhält ausdrücklich den Pfad zu Cargo.lock.
Eine fehlende Lockdatei wird nicht automatisch erzeugt. Sie beendet den Prüfschritt mit Fehler.
Die Meldungsdatenbank wird in einem neuen temporären Verzeichnis abgerufen.
Ihr Git-Commit wird nach erfolgreichem Scan im Workflow ausgegeben.

Ein lokaler Aufruf mit installiertem `cargo-audit` lautet:

```sh
cargo audit --file Cargo.lock --deny warnings
```

Der Abgleich mit der öffentlichen RustSec-Meldungsdatenbank erfolgt lokal.
Paketnamen und Versionen werden dafür nicht an einen Analysedienst hochgeladen.
Der Scanner lädt Meldungen und gegebenenfalls öffentliche Registry-Metadaten herunter.
Es sind keine pauschalen Ausnahmen oder automatischen Abhängigkeitsänderungen eingerichtet.
Die vorhandene Cargo.lock enthält nur den Bootstrap ohne externe Rust-Abhängigkeiten.
Der Scan bleibt auch nach Einführung externer Abhängigkeiten verpflichtend.

Quelle: [RustSec-Dokumentation zu cargo-audit](https://github.com/rustsec/rustsec/blob/cargo-audit/v0.22.2/cargo-audit/README.md).

## Grenzen

`node scripts/check-github-rules.mjs` prüft Basis- und Review-Regelsatz gemeinsam.
Der Basisregelsatz erlaubt keine Ausnahme von Pflichtprüfungen, PR-Zwang und Gesprächsklärung.
Nur der getrennte Review-Regelsatz enthält die ausdrücklich bestätigte Konto-Ausnahme für `@itsh-neumeier`.
Die 26 negativen Branchschutzfälle weisen unerlaubte Konten, Rollen, Ausnahmemodi und Abschwächungen ab.
Diese Tests beweisen keine aktive GitHub-Einstellung.
Der getrennte [API-Nachweis](../docs/18_Branchschutz_und_Reviews.md) bleibt erforderlich.

P0 prüft die Backlog-Abhängigkeiten, lokale Dokumentverweise und das Grundgerüst.
Es weist noch keine Mandantenisolation nach.

| Phase | Erforderlicher Nachweis vor Abnahme |
|---|---|
| P0 | Rust-Formatierung, Clippy, Build/Test; Svelte-Prüfung und Build; Strukturprüfung; Secret- und Abhängigkeitsprüfung |
| P1 | Mandanten-/Trägerisolation, RLS, Poolwechsel, Rechtewiderruf, Objekt-/Feldrechte, Zugriffsprotokoll |
| P2–P7 | Dienstverträge, Migrationen, Fachabläufe, negative Berechtigungstests, Einwilligungswiderruf |
| P8 | Last, WCAG 2.2 AA, Ende-zu-Ende-Abläufe, Container, Datensicherung und Wiederherstellung |
| P9 | Vollständige Abnahme nach v5.9, Software-Stückliste, Lizenzprüfung, signierte Images |

RLS bedeutet Row-Level Security. Clippy-Hinweise ersetzen keine Sicherheitsprüfung.
Fehlende spätere Prüfschritte dürfen nicht als bestanden ausgewiesen werden.

## P1-Ergänzung: Organisationsmodell

KC-004A ergänzt acht Integrationstests unter `services/organization-config/tests/` und einen
negativen Rust-Dokumentationstest zur Trennung der Kennungstypen.
`cargo test --workspace --locked` erfasst beide Testarten.
Die Tests verwenden ausschließlich synthetische Kennungen.
Sie prüfen doppelte Kennungen, unbekannte Träger und unveränderten Zustand bei fehlgeschlagener Anlage.
Sie ersetzen keine Rechte-, Datenbank- oder Migrationstests aus KC-004B/C.

## P1-Ergänzung: Organisationsdatenbank

Führen Sie unter Linux mit Node.js 24, Rust 1.99.1 und Docker den Befehl `node scripts/test-organization-db.mjs` aus.
Der Test startet ausschließlich einen eigenen PostgreSQL-18.6-Container mit festgelegtem Image-Digest.
Er öffnet weder Netzwerk noch Host-Port und entfernt am Ende nur seinen eigenen Container und dessen Volumes.
Docker lädt das offizielle Image bei Bedarf. Es werden keine Zugangsdaten benötigt oder gespeichert.

Geprüft werden ID-Grenzen, Primär-/Fremdschlüssel, Rollenrechte, erzwungene RLS ohne erlaubende Policy,
fehlgeschlagene Migration, abgebrochene Schreibtransaktion, Wiederholung der Erstmigration und echter Server-Neustart.
Nur die Test-Betriebsidentität fügt synthetische Daten ein; sie ist keine Laufzeitidentität.
Diese Prüfung ist im Pflichtstatus `rust` enthalten. Aktuelle Benutzerrechte und vollständige KC-005-Isolation folgen gesondert.

Der Läufer führt den Rust-/SQLx-Integrationstest ausdrücklich vor und nach dem Server-Neustart aus.
Ein temporärer Unix-Socket verbindet den Rust-Prozess mit dem Container. Der Container erhält weiterhin keinen Netzwerkzugang.
Eine ausschließlich im Testcontainer angelegte Loginrolle erbt nur die Laufzeitrechte.
Die ausgelieferte Provisionierung bleibt unverändert ohne Loginrollen oder Passwörter.
Der normale Cargo-Testlauf kennzeichnet den Datenbanktest als ignoriert; der verpflichtende Datenbankprüfschritt
führt ihn mit `--ignored --exact` aus und prüft, dass genau ein Test tatsächlich ausgeführt wurde.
Ein zusätzlicher Rust-Unit-Test weist ungültige gespeicherte Kennungen ohne Ersatzwert zurück.
Der Adaptertest prüft außerdem die unveränderte Darstellung der größten u128-Kennung,
typisierte Referenzfehler und die Rücknahme einer vom Aufrufer geführten Transaktion.

## KC-005A: Leere Einrichtungsschemas

Der vorhandene PostgreSQL-Läufer prüft Migration 0002 und die Rust-Wartungs-API in einer gesonderten Testdatenbank.
Ein fehlgeschlagener Migrationslauf muss Registry, Rechte und Policies vollständig zurücknehmen.
Der ausdrücklich ausgeführte Rust-Test prüft unbekannte Einrichtungen, Doppelanlage, vorhandene Schemata,
unsichere Runtime-Rollen, manipulierte Registrierung und Schemaeigentümer sowie Rücknahme nach Schemaanlage.
Nach echtem Server-Neustart müssen die geprüften Registrierungen und leeren Schemata unverändert vorhanden sein.
Die Test-Wartungsidentität ist ausschließlich im wegwerfbaren Testcontainer vorhanden.
Dies belegt noch keine Fachtabellen-, RLS- oder Poolisolation aus KC-005B/C.

## KC-005B: Dienstrechte und erzwungene RLS

Der [Testvertrag](../contracts/organization-config/service-isolation-test-v1.md) benennt das einzige SQL-Artefakt und seine Objektliste.
Der zusätzliche Rust-/SQLx-Test läuft ausdrücklich vor und nach dem PostgreSQL-Neustart im Pflichtstatus `rust`.
Zwei Dienste erhalten synthetische Proben in zwei registrierten Schemata. Die echten Test-Loginrollen haben getrennte Rechte.
Die Prüfung weist eigene Schreibzugriffe, fremde Dienst-/Einrichtungszugriffe, erzwungene RLS auch für Eigentümer,
unzulässige Rollen-/Eigentümerwechsel, CREATE/TRUNCATE und frei gesetzte Kontextwerte nach.
Ein absichtlich vorbereitetes Schattenschema zeigt die Wirkung des search_path. Qualifizierte Zugriffe müssen unverändert die eigene Probe verwenden.
Ein Zugang ohne Dienstzuordnung hat Schema-USAGE, jedoch weder Tabellenrechte noch Funktionsrechte; PUBLIC-Rechte dürfen ihn nicht freischalten.
Ein eingespielter Transaktionsfehler muss auch neu angelegte Rollen, Objekte und Datenbankrechte zurücknehmen.
Die produktive Provisionierung und bestehende Runtime bleiben unverändert. Benutzerrechte und produktive Identitätsbindung sind damit nicht abgenommen.

## KC-005C: Poolrückgabe und konkurrierende Wartung

Der [Testvertrag](../contracts/organization-config/pool-maintenance-test-v1.md) begrenzt die beiden zusätzlichen SQLx-Tests.
Der Pflichtprüfschritt führt beide ausdrücklich vor und nach dem Server-Neustart aus.
Ein Pool mit genau einer Verbindung prüft Rückgabe nach Erfolg, SQL-Fehler, verworfener Rust-Operation und PostgreSQL-Anweisungszeitlimit.
Dieselbe Prozesskennung muss die Wiederverwendung belegen. Ein eingespielter Fehler im Rückgabe-Hook muss dagegen die Verbindung verwerfen.
Rechte- und Policyentzug werden auf der bereits geöffneten Verbindung geprüft. Unbekannte und gelöschte Registryzuordnungen liefern keinen Wartungstreffer.

Die Wartungsprüfung beobachtet zwei wartende Sperranforderungen vor Freigabe der gemeinsamen Wartungssperre.
Sie prüft doppelte Provisionierung, doppelte Testmigration und gleichzeitige Provisionierung mit einer Migration.
Eine fehlerhafte Testwelle muss nur den betroffenen Schritt zurücknehmen und vor der nächsten Einrichtung stoppen.
Bestätigte Versionsstände sowie eine gezielte erfolgreiche Korrektur müssen nach Neustart erhalten bleiben.
Die Testmigrationshilfe akzeptiert nur zwei feste technische Schritte. Sie ist kein produktiver Orchestrator und kein Wiederherstellungsnachweis.

## KC-006A: Lokales Sitzungsmodell

Elf Rust-Integrationstests prüfen Kennungsgrenzen, Zeitgrenzen und dauerhafte lokale Zustandsübergänge.
Sie prüfen auch Beobachtung ohne Verlängerung, Aktivität ohne Überschreiten der absoluten Frist, rückläufige Zeit und Rechenüberlauf.
Ein negativer Dokumentationstest verweigert die Verwendung einer Identitätskennung als Sitzungsdatensatzkennung.
Die vorhandene Workspace-CI erfasst die neue Bibliothek mit Clippy, Tests und Build.
Diese Prüfungen sind keine Anmelde-, Kryptografie-, Persistenz- oder verteilten Widerrufsnachweise.
Es werden keine offenen Sitzungs-, MFA- oder Wiederherstellungsregeln als Vorgabewerte eingebaut.

## KC-006B: Aktueller Sitzungsbestand

Der vorhandene isolierte PostgreSQL-Läufer ergänzt zwei Datenbanken für die Identitätsmigration und den Speicheradapter.
Ein fehlgeschlagener Migrationslauf muss Tabellen, Funktionen, Typen und neue Rechte vollständig zurücknehmen. Wiederholung wird verweigert.
Der ausdrücklich ausgeführte SQLx-Test prüft aktuelle Zustände, gespeicherte Ableitung statt rohem Nachweis, Ablauf und idempotenten Widerruf.
Er beobachtet zwei echte Zeilensperrwartende bei konkurrierender Aktivität und Widerruf.
Eine weitere Sperrprüfung beginnt vor Ablauf und endet danach; nur eine nach Erwerb gelesene Zeit darf entscheiden.
Aktive äußere SQLx-Transaktionen, unterdrückte Zustandsupdates, rückläufige Zeit und Verbindungsverlust dürfen keinen positiven Treffer liefern.
Die Test-Laufzeitrolle bleibt durch RLS gesperrt und darf weder Eigentümer- noch Migrationsrolle annehmen.
Nach Neustart werden bestätigte Zustände erneut gelesen und neue Nachweise nur im Arbeitsspeicher erzeugt.
Es wird kein roher Nachweis in einer Datei über den Neustart hinweg gespeichert.
Die Prüfung weist keine Anmeldung, MFA, Mitgliedschaft oder Benutzerberechtigung nach.
