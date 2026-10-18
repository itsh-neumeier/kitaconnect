# Prüfungen und synthetische Testdaten

`fixtures/organizations.json` enthält erfundene technische Organisationskennungen.
Die Datei enthält keine Kinder, Namen, Kontaktdaten oder aus Echtdaten abgeleiteten Werte.
Sie bildet zwei getrennte Träger und eine unabhängige Einrichtung ab.
Das Lastprofil mit einer Einrichtung ist davon getrennt.

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
