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
