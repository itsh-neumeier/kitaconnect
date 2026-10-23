# Sicherheits-Patches im offenen PR-Stapel

Stand: 22.10.2026. Umfang: KC-006F / #67, technische Abhängigkeitspflege.

## Anlass und Grenze

Tokio 1.54.0 wurde im Paketregister zurückgezogen. Die vor Änderung dokumentierte Entscheidung in ADR-044 / 1637b8f verlangt den gezielten Patch 1.54.1 ohne Warnungsausnahme.
Alle 21 betroffenen eigenen älteren PR-Branches erhalten einen getrennten Patch-Commit.
Die erneute Prüfung fand in sieben älteren Lockfiles zusätzlich [RUSTSEC-2026-0382](https://rustsec.org/advisories/RUSTSEC-2026-0382) und [RUSTSEC-2026-0381](https://rustsec.org/advisories/RUSTSEC-2026-0381).
Die Korrekturversionen rustls-webpki 0.103.16 und webpki-roots 1.0.10 stammen aus dem bereits geprüften PR #85. Die Folgeentscheidung wurde in cf3b363 vor den getrennten TLS-Patches dokumentiert.
Kein Befund wird unterdrückt. Kein Branch wurde zurückgesetzt und kein Force-Push verwendet. Keine Integration in main.
PR #85 hatte die korrigierten Versionen bereits; #53/#54 und die Dependabot-PRs enthalten diese Tokio-Abhängigkeit nicht.

## Lokale Prüfung je Branch

- Der ursprüngliche lokale Kopf stimmt mit dem zuvor gelesenen GitHub-Kopf überein.
- Der Tokio-Patch ändert ausschließlich die direkten Tokio-Pins und im Lockfile die Tokio-Version samt Prüfsumme.
- Die sieben TLS-Folgepatches ändern ausschließlich die betroffenen Paketversionen, Prüfsummen und ihre erforderlichen Lockfile-Verweise.
- Gesperrte Offline-Metadatenauflösung, git diff --check und Gitleaks sind erfolgreich.
- Fachlicher Code und Migrationen bleiben unverändert. Native Tests und Sicherheitsprüfungen werden für jeden aktuellen Kopf erneut ausgeführt.

## Aktuelle Köpfe und native Nachweise

Alle 21 aktuellen Köpfe bestehen jeweils die vier aktuellen Workflow-Läufe für Push und PR mit insgesamt zehn Prüfungen.
Die maschinenlesbare Datei [TOKIO_BACKPORT_STATE.json](ai-memory/TOKIO_BACKPORT_STATE.json) enthält vollständige Commitkennungen, alle vier Laufverweise und die geprüften Grenzen.
Die folgende Tabelle verlinkt die jeweiligen aktuellen PR-Läufe.

| PR | Ursprünglicher Kopf | Tokio-Patch | Aktueller Kopf | Erfolgreiche aktuelle PR-Läufe |
|---|---|---|---|---|
| [#55](https://github.com/itsh-neumeier/kitaconnect/pull/55) | a8db9ca | c06961d | 4d1cd42 | [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723967657), [Sicherheit](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723967624) |
| [#59](https://github.com/itsh-neumeier/kitaconnect/pull/59) | 87cebbb | 6c98092 | 4cdb380 | [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723966918), [Sicherheit](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723966925) |
| [#60](https://github.com/itsh-neumeier/kitaconnect/pull/60) | 3dcf78f | d79b160 | 39345e8 | [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723967456), [Sicherheit](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723967386) |
| [#61](https://github.com/itsh-neumeier/kitaconnect/pull/61) | 4fa55b0 | 03e7633 | b3221df | [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39724512020), [Sicherheit](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39724512040) |
| [#68](https://github.com/itsh-neumeier/kitaconnect/pull/68) | c7711e8 | 4ff782a | f63fd5e | [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723966314), [Sicherheit](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723966428) |
| [#69](https://github.com/itsh-neumeier/kitaconnect/pull/69) | e0e1214 | 80d10c3 | ad945d8 | [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723966490), [Sicherheit](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723966558) |
| [#70](https://github.com/itsh-neumeier/kitaconnect/pull/70) | e003eb0 | 6bca7ce | cd0cd91 | [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723968341), [Sicherheit](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723968278) |
| [#71](https://github.com/itsh-neumeier/kitaconnect/pull/71) | 74c5e17 | 6f63b17 | 5154c4e | [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39724513852), [Sicherheit](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39724513846) |
| [#72](https://github.com/itsh-neumeier/kitaconnect/pull/72) | cb3d8f6 | 2bf99c8 | 2bf99c8 | [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723594296), [Sicherheit](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723594728) |
| [#73](https://github.com/itsh-neumeier/kitaconnect/pull/73) | f5f0586 | 1313b31 | 1313b31 | [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723617355), [Sicherheit](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723617336) |
| [#74](https://github.com/itsh-neumeier/kitaconnect/pull/74) | 58ea681 | 6eaec90 | 6eaec90 | [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723559338), [Sicherheit](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723559228) |
| [#75](https://github.com/itsh-neumeier/kitaconnect/pull/75) | 82c3a3a | 9195028 | 9195028 | [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723596154), [Sicherheit](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723596257) |
| [#76](https://github.com/itsh-neumeier/kitaconnect/pull/76) | 846d199 | 2f13d6d | 2f13d6d | [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723567565), [Sicherheit](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723567495) |
| [#77](https://github.com/itsh-neumeier/kitaconnect/pull/77) | 5927631 | bbf1e5c | bbf1e5c | [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723605819), [Sicherheit](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723605894) |
| [#78](https://github.com/itsh-neumeier/kitaconnect/pull/78) | ad150c9 | d2302cc | d2302cc | [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723577194), [Sicherheit](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723577134) |
| [#79](https://github.com/itsh-neumeier/kitaconnect/pull/79) | 088d24f | 0429f45 | 0429f45 | [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723614054), [Sicherheit](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723614171) |
| [#80](https://github.com/itsh-neumeier/kitaconnect/pull/80) | 5a4d5aa | 964323b | 964323b | [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723556201), [Sicherheit](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723556248) |
| [#81](https://github.com/itsh-neumeier/kitaconnect/pull/81) | 7813bcb | 288749d | 288749d | [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723566034), [Sicherheit](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723565971) |
| [#82](https://github.com/itsh-neumeier/kitaconnect/pull/82) | 9ad5c23 | 5d67c72 | 5d67c72 | [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723551350), [Sicherheit](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723551355) |
| [#83](https://github.com/itsh-neumeier/kitaconnect/pull/83) | 00bad74 | 0ccc890 | 0ccc890 | [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723617555), [Sicherheit](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723617598) |
| [#84](https://github.com/itsh-neumeier/kitaconnect/pull/84) | 06e28bb | ce79d6d | ce79d6d | [CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723612942), [Sicherheit](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39723612917) |

## Basis-Merges

Bei PR #61 und #71 verhinderte ein Paketmanifest-Konflikt die PR-Prüfung.
Die aktualisierte Basis wurde mit einem normalen Merge übernommen.
Die Git-Dateibäume vor und nach jedem dieser beiden Merges sind identisch. Die bereits korrigierten Paketversionen und zusätzlichen Laufzeitfeatures bleiben erhalten.
Auch die neuen Mergeköpfe bestehen eigene Push- und PR-Prüfungen. Die exakten Eltern, Konfliktpfade und Baumkennungen stehen im maschinenlesbaren Nachweis.

## Fortsetzung und Abnahmegrenze

Die erforderliche Sicherheits-Rückübernahme ist technisch geprüft. Die ursprünglichen fachlichen Akzeptanzkriterien bleiben bestehen.
Diese Nachweise erteilen keinen Auftrag zur Integration. Persönliche und sprachliche Durchsicht erfolgen weiterhin gesammelt.
Die bekannten Ablehnungsempfehlungen für #47/#48 bleiben bestehen.
Danach folgt in KC-006F der HTTP-/Introspektionsvertrag. Die begrenzte Ausnahme für standardisierte OAuth-Protokollfehler wurde ausdrücklich bestätigt und vor abhängigem Code dokumentiert.
Gateway, Organisationsadapter, Browsernachweise und das noch offene zusätzliche HTTPS-Browserprofil bleiben gesondert.
