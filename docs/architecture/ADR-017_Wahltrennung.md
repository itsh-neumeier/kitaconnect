# ADR-017 – Getrennte Wahlberechtigung und Stimmabgabe

Datum: 18.10.2026. Status: von der Projektleitung entschieden; Protokollprüfung ausstehend.
Bezug: ADR-014, ADR-015, O-001-WAHL und KC-002.

## Grundlage und Entscheidung

Die Spezifikation fordert Wahlgeheimnis auch gegenüber Systemadministratoren.
Die Projektleitung hat am 18.10.2026 Variante `4A` gewählt.
Quelle: ausdrückliche Antwort `1A, 2A, 3A, 4A` im Projektchat.

Wahlberechtigung und Stimmabgabe verwenden getrennte Datenbanken und getrennte Dienstidentitäten.
Das Wahlprotokoll MUSS vor seiner Implementierung unabhängig geprüft werden.
Diese Architekturentscheidung allein beweist kein Wahlgeheimnis.

## Datenhoheit und Rollen

Die Domäne `election` besitzt zwei getrennte Laufzeitbereiche:

| Bereich | Datenhoheit | Ausgeschlossene Zugriffe |
|---|---|---|
| Wahlberechtigung | `election_eligibility`, Berechtigungs- und Verwaltungsstatus | Keine Stimmen, keine Stimmzettelinhalte und kein lesbarer Zuordnungsschlüssel zur Stimme |
| Stimmabgabe und Auszählung | `anonymous_ballots`, `election_tallies` | Keine Wähleridentitäten, Sitzungskennungen oder lesbare Zuordnung zur Berechtigung |

Es gibt keine gemeinsame Laufzeitidentität mit Rechten auf beide Datenbanken.
Allgemeine Fachdienste erhalten keine Tabellenrechte auf diese Bestände.
Migrations- und Betriebsidentitäten bleiben von Laufzeitidentitäten getrennt.
Ein privilegiertes Administrationskonto kann durch Datenbanktrennung allein nicht als ungefährlich gelten.
Der unabhängige Protokollentwurf MUSS deshalb auch Insider und privilegierte Betriebsrollen berücksichtigen.

Mandantenzuordnung und RLS bleiben dort verbindlich, wo sie keine verbotene Verbindung erzeugen.
RLS bedeutet Row-Level Security. Das konkrete Datenmodell wird durch die Protokollprüfung festgelegt.
P0 erzeugt keine gemeinsamen Kennungen, Schnittstellen oder Datenstrukturen zur Wahl.

## Aufrufe und Betriebsgrenzen

Nur der Bereich Wahlberechtigung darf die erforderlichen Mitgliedschaften und Wahlrechte prüfen.
Der Bereich Stimmabgabe darf keine Personen über Plattformdienste oder allgemeine Audits auflösen.
Der spätere Übergang von bestätigter Berechtigung zu anonymer Stimme benötigt ein unabhängig geprüftes Verfahren.
P0 entscheidet kein kryptografisches Verfahren und implementiert diesen Übergang nicht.

Transport, Gateway, Warteschlangen und Telemetrie dürfen keinen Zusammenhang zwischen Person und Stimme erzeugen.
Allgemeine Ereignis-, Sitzungs- oder Korrelationskennungen dürfen keine gemeinsame Spur bilden.
Die getrennten Datenbanken dürfen nicht über Exporte, Diagnosen oder Datensicherungen wieder verknüpft werden.
Gemeinsame Server, Netzwerkmetadaten und Zugriffszeiten bleiben mögliche Korrelationsquellen.
Die erforderliche Trennung von Infrastruktur und Betriebsrollen ist Teil der unabhängigen Prüfung.
Separate Datenbankserver werden durch die Entscheidung `4A` weder vorausgesetzt noch ausgeschlossen.

## Verpflichtende unabhängige Prüfung vor Implementierung

KC-025 MUSS vor Implementierung des Wahlprotokolls folgende Ergebnisse vorlegen:

1. Geprüftes Verfahren für Wahlberechtigung, geheime Stimmabgabe und Verhinderung doppelter Stimmen.
2. Abdeckung von Online-, Papier- und Hybridabläufen einschließlich Abbruch und Wiederanlauf.
3. Prüfung von Kennungen, Zeitkorrelation, Gateway, Telemetrie, Warteschlangen, Exporten und Datensicherungen.
4. Prüfung privilegierter Betriebsrollen sowie des Einflusses gemeinsamer Infrastruktur.
5. Nachvollziehbarer unabhängiger Prüfbericht mit behobenen blockierenden Befunden.

Ohne diesen Nachweis darf das Wahlprotokoll nicht implementiert werden.
Ein grüner CI-Lauf oder eine allgemeine Sicherheitsprüfung ersetzt diese Fachprüfung nicht.
Die zuständige unabhängige Fachperson wird über KC-037 benannt.

## Alternative und Folgen

Die Alternative, die gesamte Speicher- und Diensttrennung erst nach der Fachprüfung zu wählen,
wurde nicht gewählt. Die Trennung ist jetzt verbindliche Architekturvorgabe für den späteren Entwurf.
Falls die Fachprüfung stärkere Grenzen verlangt, MUSS die Entscheidung nachvollziehbar ergänzt werden.
KC-002 dokumentiert die Grenze. KC-025 erbringt die spätere fachliche und technische Absicherung.

**Umsetzung in P0:** nur Dokumentation; keine Wahldatenbanken oder Wahlfunktionen angelegt.
