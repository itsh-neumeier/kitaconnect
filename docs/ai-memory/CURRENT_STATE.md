# Projektstatus und Fortsetzung

Aktualisiert: 18.10.2026, Europe/Berlin. Grundlage: Spezifikation v5.9.
Status: KC-003 ergänzt reproduzierbare Testdaten und die RustSec-Prüfung.
P0 ist insgesamt nicht abgenommen. P1 bis P9 haben weiterhin keinen Implementierungsauftrag.

## Arbeitsstand

- `issue_id`: [KC-003 / #3](https://github.com/itsh-neumeier/kitaconnect/issues/3).
- `branch`: `codex/p0-test-security-foundations`.
- `head_commit`: Ausgangscommit `d4de932c83fcb0cd987e7e979dceab9a90e660cd`; KC-003 folgt als eigener Commit.
  Aktuellen Branchkopf mit `git rev-parse HEAD` prüfen.
- `objective`: Reproduzierbare synthetische Organisationsdaten und vollständige P0-Prüfmatrix.
- `migration_state`: Keine Migrationen, Datenbankrollen oder Fachdatenspeicherung angelegt.
- `pull_request`: Eigener KC-003-PR auf Basis von `codex/p0-service-boundaries`; Link im GitHub-Issue #3.
- `dependencies`: PR #39 enthält das Grundgerüst. PR #40 enthält die bestätigten Dienstgrenzen.
  Beide PRs waren beim Start noch offen. Nicht ungeprüft zusammenführen.

## Bereits bestätigte Architektur

Die ausdrückliche Antwort `1A, 2A, 3A, 4A` vom 18.10.2026 bleibt gültig:
Einrichtungsschemas mit getrennten Dienstrechten, Rust-Gateway, gesonderte Personalbereiche
und getrennte Datenbanken/Dienstidentitäten für Wahlberechtigung und Stimme.
ADR-014 bis ADR-017 dokumentieren diese Entscheidungen. Die Fragen nicht erneut stellen.
Die unabhängige Prüfung des Wahlprotokolls bleibt vor Implementierung verpflichtend.

## Erledigt in KC-003

1. `scripts/fixtures.mjs` erzeugt ausschließlich feste Organisationskennungen ohne externe Eingaben.
2. `fixtures:check` vergleicht die versionierten Testdaten bytegenau mit dem Generator.
3. Die Strukturprüfung weist unbekannte Felder, ungültige Datentypen, doppelte Kennungen und falsche Zuordnungen ab.
4. Neun Testdatenprüfungen ergänzen die vier bisherigen Backlog-Tests. Keine Fach- oder Isolationstests vorgetäuscht.
5. Der CI-Prüfschritt `structure` prüft zusätzlich die bytegenaue Testdatenerzeugung.
6. `dependencies` prüft Cargo.lock mit cargo-audit 0.22.2 gegen RustSec.
   Die bisherige pauschale Sperre externer Rust-Abhängigkeiten ist dadurch ersetzt.
7. Offizielles Scannerarchiv, feste SHA-256-Prüfsumme, explizite Lockdatei und temporäre Meldungsdatenbank festgelegt.
8. ADR-018, P0-Plan und [Prüfmatrix](../../tests/README.md) beschreiben Umfang, Befundbehandlung und Grenzen.

## Lokale Prüfungen

| Befehl / Nachweis | Ergebnis |
|---|---|
| `node scripts/fixtures.mjs --check` | Ausgabe stimmt bytegenau mit Generator überein |
| `node scripts/check-project.mjs` | 38 Issues, 24 Dokumente, lokale Verweise und Testdaten geprüft |
| `node --test scripts/*.test.mjs` | 13 von 13 Tests erfolgreich |
| Actionlint 1.7.12 | Beide Workflows ohne Fehler geprüft |
| `npm run check:web` | Keine Fehler oder Warnungen |
| `npm run build:web` | Erfolgreich |
| cargo-audit 0.22.2 | 1338 RustSec-Meldungen geladen; Cargo.lock ohne Befund geprüft |
| `git diff --check` | Erfolgreich |

Der erste Generatorvergleich erkannte die bisherige abweichende JSON-Formatierung.
Die Datei wurde danach aus dem Generator neu erzeugt; ihre Organisationsdaten bleiben gleich.
Der erste lokale RustSec-Aufruf fand Cargo für den Registry-Abgleich nicht.
Nach Setzen des lokalen Cargo-Pfads war der erneute Scan ohne diese Warnung erfolgreich.
Es gibt keine bekannten verbleibenden lokalen Prüffehler.
Secret-Scan und GitHub-Ergebnisse des finalen Commits werden im PR nachgewiesen.
Rust-Quelltext bleibt unverändert. Build und Tests laufen zusätzlich in der Linux-CI.

## Sicherheitsumfang

Nur P0-Prüfwerkzeuge und erfundene Organisationskennungen. Keine Kinderdaten oder produktiven Secrets.
Die Testdaten sind kein Nachweis implementierter Mandantenisolation.
Workflow-Tokens behalten Leserechte. Es gibt keine Veröffentlichung und keine neuen Laufzeitdienste.
Die vorhandene Erlaubnis für npm audit lokal und in GitHub Actions gilt weiter.
cargo-audit lädt öffentliche Meldungen und Registry-Metadaten herunter; der Paketabgleich erfolgt lokal.
Der Rust-Workspace enthält noch keine externen Abhängigkeiten. Die Prüfung gilt auch für spätere Lockfile-Erweiterungen.

## Offene Aufgaben und nächste drei Aktionen

1. KC-003-PR mit aktuellem CI-Stand prüfen. Erst nach Review und Integration Issue #3 schließen.
2. [KC-037 / #37](https://github.com/itsh-neumeier/kitaconnect/issues/37) fortsetzen:
   O-003 bis O-008, Lastwerte, Versionen, Lizenzprüfung und unabhängige Fachpersonen klären.
3. [KC-038 / #38](https://github.com/itsh-neumeier/kitaconnect/issues/38) und KC-036 klären:
   bestätigte CODEOWNERS, nachweisbarer Branchschutz und sprachliche Bestandsprüfung.

Für die Integration zuerst PR #39, danach PR #40, anschließend den KC-003-PR prüfen.
Nach Integration einer Basis den folgenden PR auf `main` umstellen und erneut prüfen.
Keine Merge-Freigabe aus diesem Checkpoint ableiten.
Die Lastprofilangabe `1` bezeichnet eine Einrichtung. Weitere Lastwerte fehlen.
CODEOWNERS ist noch nicht bestätigt. Bisheriger Branchschutz bleibt wegen HTTP 403 unbekannt.

## Fortsetzungsanleitung

Lesen Sie AGENTS, CLAUDE, diesen Checkpoint, die Entscheidungsliste und die aktuellen GitHub-Issues/PRs.
Prüfen Sie Branch, Arbeitsbaum, Commits und aktuelle CI, bevor Sie weiterarbeiten.
Wiederholen Sie weder die bestätigten Architekturfragen noch das vorhandene Grundgerüst.
Bearbeiten Sie weiter P0. P1 benötigt dessen Abnahme und einen eigenen Auftrag.
