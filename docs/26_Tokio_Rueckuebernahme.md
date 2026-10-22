# Tokio-Patch im offenen PR-Stapel

Stand: 22.10.2026. Umfang: KC-006F / #67, technische Abhängigkeitspflege.

## Anlass und Grenze

Die Sicherheitsprüfung von PR #85 meldete Tokio 1.54.0 als zurückgezogen.
Die vor Änderung dokumentierte Entscheidung in ADR-044 / 1637b8f verlangt den gezielten Patch 1.54.1 ohne Warnungsausnahme.
Der Patch ist in den folgenden 21 bestehenden eigenen PR-Branches als jeweils eigener Commit übernommen.
Es wurde kein Branch zurückgesetzt und kein Force-Push verwendet. Keine Integration in main.
PR #85 hatte den Patch bereits; #53/#54 und die Dependabot-PRs enthalten diese Tokio-Abhängigkeit nicht.

## Lokale Prüfung je Branch

- Ursprünglicher lokaler Kopf stimmt mit dem zuvor gelesenen GitHub-Kopf überein.
- Änderungen betreffen ausschließlich die aufgeführten Cargo-Manifeste und Cargo.lock.
- Das Lockfile ändert ausschließlich Tokio-Version und deren Paketprüfsumme. Abhängigkeiten des Tokio-Pakets bleiben gleich.
- Gesperrte Offline-Metadatenauflösung, git diff --check und Gitleaks sind erfolgreich.
- Native Tests und Sicherheitsprüfungen werden für jeden neuen Kopf erneut ausgeführt. Frühere grüne Ergebnisse ersetzen diese Prüfungen nicht.

## Übernommene Köpfe

| PR | Ursprünglicher Kopf | Patchkopf | Geänderte Manifeste |
|---|---|---|---|
| [#55](https://github.com/itsh-neumeier/kitaconnect/pull/55) | a8db9ca | c06961d | `services/organization-config/Cargo.toml` |
| [#59](https://github.com/itsh-neumeier/kitaconnect/pull/59) | 87cebbb | 6c98092 | `services/organization-config/Cargo.toml` |
| [#60](https://github.com/itsh-neumeier/kitaconnect/pull/60) | 3dcf78f | d79b160 | `services/organization-config/Cargo.toml` |
| [#61](https://github.com/itsh-neumeier/kitaconnect/pull/61) | 4fa55b0 | 03e7633 | `services/organization-config/Cargo.toml` |
| [#68](https://github.com/itsh-neumeier/kitaconnect/pull/68) | c7711e8 | 4ff782a | `services/organization-config/Cargo.toml` |
| [#69](https://github.com/itsh-neumeier/kitaconnect/pull/69) | e0e1214 | 80d10c3 | `services/identity-access/Cargo.toml`, `services/organization-config/Cargo.toml` |
| [#70](https://github.com/itsh-neumeier/kitaconnect/pull/70) | e003eb0 | 6bca7ce | `services/identity-access/Cargo.toml`, `services/organization-config/Cargo.toml` |
| [#71](https://github.com/itsh-neumeier/kitaconnect/pull/71) | 74c5e17 | 6f63b17 | `services/identity-access/Cargo.toml`, `services/organization-config/Cargo.toml` |
| [#72](https://github.com/itsh-neumeier/kitaconnect/pull/72) | cb3d8f6 | 2bf99c8 | `services/identity-access/Cargo.toml`, `services/organization-config/Cargo.toml` |
| [#73](https://github.com/itsh-neumeier/kitaconnect/pull/73) | f5f0586 | 1313b31 | `services/identity-access/Cargo.toml`, `services/organization-config/Cargo.toml` |
| [#74](https://github.com/itsh-neumeier/kitaconnect/pull/74) | 58ea681 | 6eaec90 | `services/identity-access/Cargo.toml`, `services/organization-config/Cargo.toml` |
| [#75](https://github.com/itsh-neumeier/kitaconnect/pull/75) | 82c3a3a | 9195028 | `services/identity-access/Cargo.toml`, `services/organization-config/Cargo.toml` |
| [#76](https://github.com/itsh-neumeier/kitaconnect/pull/76) | 846d199 | 2f13d6d | `services/identity-access/Cargo.toml`, `services/organization-config/Cargo.toml` |
| [#77](https://github.com/itsh-neumeier/kitaconnect/pull/77) | 5927631 | bbf1e5c | `services/identity-access/Cargo.toml`, `services/organization-config/Cargo.toml` |
| [#78](https://github.com/itsh-neumeier/kitaconnect/pull/78) | ad150c9 | d2302cc | `services/identity-access/Cargo.toml`, `services/organization-config/Cargo.toml` |
| [#79](https://github.com/itsh-neumeier/kitaconnect/pull/79) | 088d24f | 0429f45 | `services/identity-access/Cargo.toml`, `services/organization-config/Cargo.toml` |
| [#80](https://github.com/itsh-neumeier/kitaconnect/pull/80) | 5a4d5aa | 964323b | `services/identity-access/Cargo.toml`, `services/organization-config/Cargo.toml` |
| [#81](https://github.com/itsh-neumeier/kitaconnect/pull/81) | 7813bcb | 288749d | `services/identity-access/Cargo.toml`, `services/organization-config/Cargo.toml` |
| [#82](https://github.com/itsh-neumeier/kitaconnect/pull/82) | 9ad5c23 | 5d67c72 | `services/identity-access/Cargo.toml`, `services/organization-config/Cargo.toml` |
| [#83](https://github.com/itsh-neumeier/kitaconnect/pull/83) | 00bad74 | 0ccc890 | `services/identity-access/Cargo.toml`, `services/organization-config/Cargo.toml` |
| [#84](https://github.com/itsh-neumeier/kitaconnect/pull/84) | 06e28bb | ce79d6d | `services/identity-access/Cargo.toml`, `services/organization-config/Cargo.toml` |

## Aktueller Nachweis und Fortsetzung

Die erste erneute Sicherheitsprüfung zeigt in älteren Lockfiles zwei zusätzliche Befunde:
[RUSTSEC-2026-0382](https://rustsec.org/advisories/RUSTSEC-2026-0382) betrifft rustls-webpki 0.103.15;
[RUSTSEC-2026-0381](https://rustsec.org/advisories/RUSTSEC-2026-0381) betrifft webpki-roots 1.0.9.
Die bereits in PR #85 geprüften Korrekturversionen 0.103.16 und 1.0.10 werden deshalb zusätzlich in den tatsächlich betroffenen älteren Lockfiles übernommen.
Diese Folgeentscheidung wird vor der Änderung dokumentiert. Kein Befund wird unterdrückt.
Die erste Tokio-Änderung bleibt als eigener Commit nachvollziehbar; der zusätzliche TLS-Patch erhält einen getrennten Commit.

Die aktuelle native CI läuft noch. Abschlussstände werden mit vollständigen Commitkennungen und direkten Prüfverweisen in `docs/ai-memory/TOKIO_BACKPORT_STATE.json` nachgeführt.
Kein Abschluss wird aus einem lediglich vorgemerkten oder laufenden Prüfschritt abgeleitet.
Die Akzeptanzkriterien der ursprünglichen PRs und Issues bleiben erhalten.

Vor Integration muss jede betroffene Basis den Patch enthalten und ihre aktuellen Pflichtprüfungen bestehen.
Die persönliche und sprachliche Durchsicht erfolgt wie beauftragt gesammelt. Eine Integration benötigt weiterhin einen gesonderten Auftrag.
Danach folgt in KC-006F der HTTP-/Introspektionsvertrag; Gateway, Organisation und Browser bleiben weitere Nachweise.
