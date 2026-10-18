# KC-036 – Redaktionelle Prüfung der Bestandsunterlagen

Stand: 18.10.2026. Grundlage: Spezifikation v5.9.
Ausgangscommit: `bdb3f9fcf1c077c2359e17488062bf6c1d2a8c60` aus PR #43.
Umfang: ursprüngliche Unterlagen 01 bis 12, Fachwortverzeichnis und Nachweis der Redaktion.
Die Änderungen sind zur Prüfung vorbereitet. Unabhängiges Review und Integration stehen aus.

## Verfahren

1. Die vollständige Ausgangsfassung der zwölf Unterlagen wurde gelesen.
2. Lange Stichwortketten wurden in Sätze, Listen oder Tabellen aufgeteilt.
3. Deutsche Fachbegriffe wurden mit dem Fachwortverzeichnis abgeglichen.
4. Technische Bezeichner, API-Pfade, Ereignisnamen und Berechtigungswerte wurden erhalten.
5. Jeder geänderte Abschnitt wurde am Ausgangstext auf Aussage, Einschränkungen und Zahlen geprüft.
6. Offene fachliche Formulierungen wurden dokumentiert. Es wurde keine neue Fachentscheidung getroffen.

Die manuelle redaktionelle Eigenprüfung umfasst die folgende Tabelle.
Sie ist keine unabhängige Genehmigung des Pull Requests.
Vor Merge muss eine andere berechtigte Person die sprachliche Prüfung bestätigen.
Die Rollenklärung bleibt Teil von KC-038.

## Nachweis je Unterlage

| Unterlage | Redaktionelle Prüfung | Erhaltene Grenzen |
|---|---|---|
| [01 Fachkonzept](01_Fachkonzept.md) | Funktionsketten gegliedert; Begriffe vereinheitlicht; historische Abschnitte als 12.1–12.13 nummeriert | Drei Betriebsmodelle; alle genannten Funktionen und Interviewumfänge; keine Essensabrechnung; Wahlgeheimnis; bedingte Zweitfreigabe; 1.0 mit vollem Umfang |
| [02 Architektur](02_Technische_Architektur.md) | Transport, Identität, Migration und Wiederherstellung in Sätze aufgeteilt; Abkürzungen erläutert | Werkzeugwahl; Schemas/RLS; kein pauschaler Administrationszugriff; Wahltrennung; RPO und RTO jeweils Ziel 24 Stunden; Vorbehalte bei FORCE RLS und Hochverfügbarkeit |
| [03 Datenmodell](03_Datenmodell_und_Berechtigungen.md) | Legende und Sicherheitsinvarianten erläutert; Tabellen unverändert | Sämtliche konzeptionellen Tabellen und Berechtigungszellen einschließlich Sternbedingungen; sofortiger Widerruf; Freigabe für Fachaktentransfer |
| [04 API](04_API_und_Integrationen.md) | Leitlinien und Integrationen in Sätze aufgeteilt; Begriffe erklärt | Methoden, Pfade, Sicherheitsregeln und sieben Ereignisnamen unverändert; serverseitiger Kontext; Idempotenz und Widerruf |
| [05 Nutzung](05_UX_UI_Konzept.md) | Navigation als Tabelle; Interaktionsmuster und Abläufe als Sätze | Alle sechs Abläufe; WCAG 2.2 AA; kein stiller Offline-Zugriff; Ablaufreihenfolge bis zur Antwort auf die Rückfrage erhalten |
| [06 Roadmap](06_Entwicklungsroadmap.md) | Phasenergebnisse und Abnahmebegriffe vereinheitlicht; riskante Bereiche als Liste | P0–P9 und sämtliche Abhängigkeiten; 0,5–2 Entwicklungstage; 100 % Pflichtumfang; keine öffentliche Vorabversion |
| [07 Backlog](07_GitHub_Issue_Backlog.md) | Veralteten Verweis auf das nächste Issue durch Checkpoint-Verweis ersetzt; Sprachregel ergänzt | GitHub bleibt verbindlich; Importdateien bleiben historische Nachweise; kein Auftrag für P1 |
| [08 Prüfkonzept](08_Test_und_Release_Konzept.md) | Testbereiche und Veröffentlichungsschritte gegliedert; bestätigte Lastziele als vorhanden gekennzeichnet | Sämtliche Testbereiche; blockierende Sicherheitsprüfungen; mindestens ein Review; keine Veröffentlichung bei Fehler; keine echten Kinderdaten |
| [09 Betrieb](09_Self_Hosting_Betrieb.md) | Soll-Abläufe als Anweisungen; Datensicherung und Betrieb erklärt | Manuelle Installationsfreigabe; konsistente Sicherung; Lösch-/Einwilligungsabgleich; RPO/RTO; keine getestete Installationsfreigabe behauptet |
| [10 Datenschutz](10_Datenschutz_und_Sicherheit.md) | Schutzklassen, Rechte und Bedrohungen gegliedert; Fachbegriffe erklärt | Fachprüfung vor Produktivbetrieb; alle Protokollfelder; nur ergänzbare/manipulationserkennbare Speicherung; Notfallgrenzen und Wahlgeheimnis |
| [11 Fortsetzung](11_KI_Fortsetzung.md) | Handlungen als nummerierte Anweisungen formuliert | Kleine Issues; Dienstvertrag vor Implementierung; Testpflicht; keine Nutzungsumgehung; Checkpoint und nächste drei Aktionen |
| [12 Entscheidungen](12_Entscheidungen_und_Konflikte.md) | Konfliktregel, historische Tabelle und Sprachentscheidung erläutert | Vorrang ausdrücklicher Entscheidungen; ADR-/Interviewkennungen; alle bestätigten technischen Entscheidungen und offenen Fachprüfungen |

## Technischer Vergleich

Ein lokaler Vergleich gegen den Ausgangscommit prüfte:

- Bytegleichheit aller Entitäts- und Berechtigungstabellen aus Unterlage 03.
- Bytegleichheit der vollständigen API-Tabelle aus Unterlage 04.
- Gleiche Ereignisnamen, Phasenabhängigkeiten und ADR-/Interviewkennungen.
- Erhalt der festgelegten Rust-, Node.js- und npm-Versionen im Entscheidungsprotokoll.

Strukturprüfung, lokale Dokumentverweise und bestehende Tests werden zusätzlich ausgeführt.
Diese Vergleiche beweisen keine vollständige fachliche Gleichheit.
Sie unterstützen die manuelle Prüfung und ersetzen kein unabhängiges Review.
Es wurden keine zusätzlichen Tests geschrieben, die nur einzelne Formulierungen festschreiben.

Prüfen Sie die sprachlichen Unterschiede mit `git diff --word-diff=plain bdb3f9f HEAD -- docs`.
Die ersten drei Redaktionsgruppen sind in den Commits `ebaa1d2`, `4804d1c` und `7e49a66` getrennt.
Der folgende Nachweiscommit ergänzt Begriffe, Sprachhinweise, diesen Bericht und den Checkpoint.

## Offene Formulierung und erhaltene Konfliktgrenzen

**F-001 – Tagesereignis:** Der ursprüngliche Ablauf in Unterlage 05 nennt die Berechtigungsprüfung nach dem Dokumentieren.
Die Projektleitung wurde gefragt, ob Eingabe, Prüfung vor dem Speichern und Speichern getrennt benannt werden sollen.
Bis zur Antwort bleibt diese Reihenfolge erhalten und als offene Formulierung dokumentiert.
Die verbindliche aktuelle serverseitige Rechteprüfung aus ADR-019 bleibt bestehen.
Dieser Redaktionsstand erlaubt kein Speichern unberechtigter Daten.
Die spätere Umsetzung darf aus der knappen Ablaufskizze keine abweichende Sicherheitsregel ableiten.

Die historische Ausschlussliste aus Unterlage 01 ist weiterhin als historischer Stand enthalten.
Die bereits vorhandene Vorrangregel und die späteren Anforderungen zu Krank-/Abwesenheitsmeldungen und Abholrechten bleiben wirksam.
Der Ausschluss des Fundbüros wird nicht eigenmächtig aufgehoben.

Die Formulierung zur trägerübergreifenden Personalverwaltung bleibt erhalten.
Sie erteilt keine pauschalen Querzugriffsrechte. Die bestätigte Trennung und begrenzte Einsatzsicht aus ADR-016 gelten weiter.
Die frühe allgemeine mTLS-Vorgabe bleibt als Ausgangsvorgabe erkennbar. ADR-019 konkretisiert die bestätigte Dienstkommunikation.
Der allgemeine Korrelationsbegriff aus Unterlage 04 schwächt die Wahlgrenzen aus ADR-017 nicht ab.

## Entscheidungen und Abnahme

Es wurde keine neue Architekturentscheidung getroffen. ADR-010 bleibt die Grundlage der Redaktion.
Zuständigkeiten, Rechte, Fristen, Betriebsziele und Entwicklungsphasen werden durch KC-036 nicht neu festgelegt.
Das Fachwortverzeichnis ergänzt deutsche Begriffe und Erläuterungen vorhandener Abkürzungen.
Die deutsche Richtlinie ist eine eigene Adaption. Es wird keine formelle ASD-STE100-Konformität behauptet.

KC-036 bleibt bis zur Antwort auf F-001, unabhängigem sprachlichem Review und Integration offen.
P0-Gesamtabnahme, Fachfreigaben und Freigabe einer weiteren Entwicklungsphase stehen weiterhin aus.
