# KitaConnect – Spezifikation v5.9
Die Spezifikation v5.9 aus Interviewfragen 1–51 ist verbindlich. P0 ergänzt ein technisches Grundgerüst ohne Fachfunktionen. Frühere Spezifikation v4.2 ist durch die aktuellen ADRs ersetzt.

Start: `docs/01_Fachkonzept.md`, `docs/12_Entscheidungen_und_Konflikte.md`, `docs/06_Entwicklungsroadmap.md`.

Dokumentationsstandard: `docs/13_Dokumentationsrichtlinie_DE_ASD_STE100.md` und `docs/14_Fachwortverzeichnis_DE.md`.
Die deutsche Adaption der Sprachprinzipien belegt **keine offizielle ASD-STE100-Konformität**.
Der [KC-036-Prüfbericht](docs/19_Redaktionspruefung_KC-036.md) dokumentiert den redaktionellen Durchgang für Unterlagen 01 bis 12.
Die Projektleitung hat die persönliche und sprachliche Durchsicht für PRs #39 bis #44 bestätigt.

Der [P0-Plan](docs/15_P0_Initialisierung.md) beschreibt Struktur und Entscheidungen.
Der [Projektstatus](docs/ai-memory/CURRENT_STATE.md) nennt Nachweise und nächste Schritte.
Der [technische P0-Prüfbericht](docs/20_P0_Pruefung_und_Abnahme.md) führt die Nachweise aller sechs vorbereiteten PRs zusammen.
Der [Integrationsnachweis](docs/21_P0_Integration_und_Abnahmevorbereitung.md) dokumentiert ihre Integration und die noch offene P0-Gesamtabnahme.
GitHub ist die verbindliche Quelle für Issues. Der [Backlog](docs/07_GitHub_Issue_Backlog.md)
erläutert den Importstand und die stabilen KC-Kennungen.

## Lokale Entwicklung

Voraussetzungen: Rust 1.99.1 mit Rustfmt und Clippy, Node.js 24.19.0, npm 12.2.0.
Unter Windows braucht Rust zusätzlich Visual-Studio-C++-Buildwerkzeuge und das Windows SDK.
Diese Werkzeuge gehören nicht zum Repository. Verwenden Sie nur synthetische Testdaten.

```sh
npm ci --ignore-scripts
npm run check
npm run fixtures:check
npm test
npm run check:web
npm run build:web
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --locked
npm audit --audit-level=high
```

Starten Sie die interne Hinweisseite mit `npm run dev --workspace @kitaconnect/web`.
Der Rust-Bootstrap startet keinen API-Dienst. Die Infrastruktur ist noch nicht installierbar.
P0 ist vor Bestätigung offener Entscheidungen und Review nicht abgenommen.
Der [Lizenzentwurf](docs/16_Lizenzentwurf.md) erteilt keine neuen Nutzungsrechte.

Die [Prüfmatrix](tests/README.md) beschreibt die reproduzierbare Testdatenerzeugung
und die Rust-Abhängigkeitsprüfung mit `cargo-audit` 0.22.2.
