# Sicherheitskorrektur: ctutils 0.4.4

Stand: 23.10.2026. Umfang: KC-006F / #67 und betroffene ältere eigene PRs.

## Anlass und Entscheidung

Die verpflichtende Prüfung von PR #94 meldet am 23.10.2026 RUSTSEC-2026-0385 in ctutils 0.4.3.
Die Schwachstelle betrifft zeitabhängige Maskenerzeugung und u128-Vergleiche.
Die [RustSec-Meldung](https://github.com/RustSec/advisory-db/blob/main/crates/ctutils/RUSTSEC-2026-0385.md)
und die [Korrektur des Herstellers](https://github.com/RustCrypto/utils/pull/1569) benennen Version 0.4.4 als Korrektur.
Entscheidung 698ddf8 steht vor der Paketänderung. Keine Warnung wird unterdrückt.

## Geprüfter Änderungsumfang

Dreißig ältere eigene PRs von #55 bis #93 enthalten ctutils 0.4.3.
#53 und #54 enthalten dieses Paket nicht. Die Abfragen verwenden die aktuellen GitHub-Köpfe und gleichen lokale Remote-Referenzen ab.
Jeder betroffene Branch erhält ausschließlich die Änderung von Version und Prüfsumme im Lockfile.
Der Vergleich des vollständigen Lockfiles bestätigt unveränderte übrige Paketblöcke.
Gesperrte Offline-Metadaten ohne Abhängigkeitsausgabe, git diff --check und Gitleaks prüfen jeden Patch vor dem Commit.
Veröffentlichung erfolgt ohne Force-Push. Keine Integration in main findet statt.

PR #72 benötigt zusätzlich eine normale Übernahme der aktualisierten Basis aus PR #71.
Der einzige Konflikt betrifft den benachbarten ctr-Block im Lockfile.
Der vollständige Dateiinhalt nach Auflösung ist bitgleich zum vorherigen Patchkopf 2152d7d.
Der Merge erhält damit ausschließlich die richtige Elternbeziehung. Er verändert keine Implementierung.
Die vollständigen Köpfe und der gemeinsame Dateibaum sind im Nachweis verzeichnet.

## Verbindlicher Nachweis

[CTUTILS_BACKPORT_STATE.json](ai-memory/CTUTILS_BACKPORT_STATE.json) enthält ursprüngliche und korrigierte Köpfe sowie die aktuellen Workflow-Verweise.
Dreißig von dreißig Rückübernahmen bestehen alle vier aktuellen Läufe.
Die zugehörigen 300 maßgeblichen Prüfungen sind anhand der vollständigen Köpfe und ausgewählten Laufkennungen geprüft.
Je Branch müssen die aktuellen CI- und Sicherheitsläufe für Push und PR erfolgreich sein.
Durch Basisänderungen abgebrochene ältere Läufe werden durch den neuesten Lauf desselben Workflow-/Ereignispaares ersetzt.
Ein alter abgebrochener Lauf gilt nicht als neuer Erfolg. Maßgeblich sind die ausgewählten vollständigen Köpfe und vier aktuellen Läufe.
Solange ein solcher Lauf fehlt oder nicht erfolgreich ist, bleibt der Abschluss offen.
Frühere grüne Nachweise vor der neuen RustSec-Veröffentlichung sind historische Ergebnisse.

Die gesammelte persönliche und sprachliche Durchsicht bleibt ausstehend.
Dieser technische Nachweis ersetzt keine Fachfreigabe und keinen gesonderten Merge-Auftrag.
