# ADR-039 – Zweckgebundene zusätzliche Authentisierung

Datum: 21.10.2026. Status: technische Konkretisierung vor Code für KC-006F.
Grundlage: bestätigte MFA-Kombinationen und Fristen, ADR-036/037/038.

## Entscheidung

Der [Vertrag](../../contracts/identity-access/step-up-v1.md) ergänzt einen ausdrücklich zweckgebundenen Faktorprüfvorgang.
Er bindet genau die aktive Ziel-Sitzung und kann nicht als normaler Anmeldeabschluss verwendet werden.
Die tatsächlichen Faktoradapter bleiben die Quelle des neuen Nachweises.
Eine eingeschränkte interne Zielprüfung erlaubt den Beginn einer erneuten Faktorprüfung, ohne die normale MFA-Pflicht abzuschwächen.

Ein unveränderlicher zusätzlicher Beleg gehört zur ursprünglichen Anmeldung.
Seine tatsächliche Faktorzeit darf die zusätzliche Authentisierung erneuern; absolute Laufzeit und strengste Fristen bleiben erhalten.
Abschluss und Rotation sind atomar, damit der alte Sitzungsnachweis die erneuerte Stärke nicht weiterverwenden kann.

## Alternativen und Folgen

Die Übernahme alter positiver Ergebnisse wurde verworfen: Sie beweisen keine aktuelle zweckgebundene Prüfung.
Eine vollständig neue Anmeldung wurde verworfen: Zusätzliche Authentisierung soll die bestehende absolute Frist nicht erneuern.
Die Änderung der ursprünglichen Faktorbindung wurde verworfen: Ein eigener Beleg erhält deren unveränderlichen Herkunftsnachweis.
Ein bloßer MFA-Zeitstempel ohne tatsächliche Faktortransaktion ist unzulässig.

## Grenzen

Keine neue MFA-Variante, Rechtezuordnung, Mitgliedschaftsvergabe oder produktive Schlüssel-/Domainentscheidung.
Die bestehenden Offline-Wiederherstellungsregeln bleiben unverändert.
Neue Implementierung und Migration folgen erst nach diesem versionierten Vertrag.
Browser, Gateway und Dienstidentitäten behalten ihre gesonderten Nachweise.
