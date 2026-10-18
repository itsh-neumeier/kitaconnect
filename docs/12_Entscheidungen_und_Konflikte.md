# Entscheidungs- und Konfliktprotokoll

**KitaConnect Spezifikation v5.9 · 17.10.2026 · Konsolidierter Planungsstand**

> Status: Anforderungsspezifikation, keine implementierte oder rechtsverbindlich geprüfte Software. Die neuesten ausdrücklichen Interviewentscheidungen gelten.

## Konfliktregel

Die neueste **ausdrückliche** Interviewentscheidung hat Vorrang.
Änderungen werden protokolliert. Rechtliche und sicherheitskritische Widersprüche werden nicht stillschweigend übergangen.

| ID | Frühere Annahme | Aktuelle Entscheidung | Folge |
|---|---|---|---|
| ADR-001 | Modularer Monolith | Frühe Microservices (Q47) | Dienstgrenzen, Dienstvertragstests, Orchestrierung |
| ADR-002 | Gemeinsames Schema mit `tenant_id` + RLS | Schema je Mandant + RLS (Q47) | Schema-Migrationsorchestrator und Pool-Isolation |
| ADR-003 | Veröffentlichung manuell durch den Verantwortlichen | Automatisch nach verpflichtender CI (Q50/51) | Geschützter Branch, Reviews, Veröffentlichungsprüfungen |
| ADR-004 | Erste stabile Version mit allen Modulen | Q51: Kernmodule zuerst entwickeln, **vollständiger Umfang für 1.0** | Interne Phasen, keine öffentliche Teilversion |
| ADR-005 | Elternbeirat ohne Krankmeldungen, Abwesenheitsmeldungen oder Abholrechte | Vollständiges Einrichtungsmodul für Anwesenheit/Abholung (Q42/45) | Betrieb nur für Elternbeirat ohne Kinderakte weiterhin möglich |
| ADR-006 | Fotoeinwilligung nach Verwendungszweck | Zusätzlich pro Einzelbild/Veröffentlichung (Q44) | Einwilligungsmatrix und Veröffentlichungsprüfung |
| ADR-007 | Essenspreise und Abrechnung in Funktionsliste | Ausdrücklich **keine Abrechnung** im Essensmodul (Q38) | Preise/Berichte ja, Rechnungen/Zahlungsabwicklung nein |
| ADR-008 | Eltern- und Einrichtungsbereiche getrennt | Gemeinsame Prozesse bei strikter Datengrenze (Q41) | Freigabe mit begrenztem Berechtigungsumfang, keine impliziten Rechte |
| ADR-009 | Personalverwaltung lokal | Trägerübergreifend konfigurierbar (Q40/43) | Übergreifender Personalverwaltungsbereich, Einsätze je Einrichtung |

## Noch fachlich/rechtlich zu validieren (keine offenen Grundsatzentscheidungen)

- Datenschutzrechtliche Verantwortlichkeit, Rechtsgrundlagen und landesspezifische Aufbewahrungsfristen.
- Verbindliche Betreuungsschlüssel und Arbeitszeit-/Pausenregelwerke pro Region/Träger.
- Wahlordnungen, Stimmgewicht und unabhängige Sicherheitsprüfung des Wahlprotokolls.
- Rechtswirksamkeit elektronischer Vertragsabschlüsse und Dokumentenunterzeichnung.
- Konkrete Lastziele, Infrastrukturgrößen und RPO/RTO-Nachweis.
- Proprietärer Lizenztext und Weitergabe-/Self-Hosting-Rechte juristisch prüfen.
- Exakte Definition aller Fachakzeptanzkriterien auf Issue-Ebene vor Implementierung.

## Status

Die fachlichen Interviews 1–51 sind abgeschlossen.
Die Dateien beschreiben einen konsolidierten Planungsstand. Sie belegen weder rechtliche Zertifizierung noch eine implementierte Anwendung.
ADR bezeichnet eine dokumentierte Architekturentscheidung. Q bezeichnet die jeweilige Interviewfrage.
CI bezeichnet die automatisierte Integration. RLS bezeichnet Row-Level Security.
RPO bezeichnet den angestrebten maximalen Datenverlustzeitraum, RTO die angestrebte Wiederherstellungsdauer.

## ADR-011 – P0-Struktur und dienstbezogene Verantwortung

Datum: 18.10.2026. Status: für das technische P0-Grundgerüst entschieden.
Grundlage: ADR-001, ADR-002 und Auftrag zur Projektinitialisierung.

**Entscheidung:** Rust-Pakete liegen unter `services/`, technische Bibliotheken
unter `crates/`, das SvelteKit-Frontend unter `apps/web/`.
Verträge liegen unter `contracts/`, Infrastruktur unter `infra/`.
P0 enthält nur ein Rust-Werkzeugkettenprogramm und eine Frontend-Hinweisseite.
**Alternative:** Ein gemeinsames Fachpaket wurde verworfen. Es würde Datenhoheit
und frühe Microservice-Grenzen verschleiern.
**Folge:** KC-002 konkretisiert die Dienstmatrix durch ADR-014 bis ADR-017.
Schema je Mandant und RLS bleiben verbindlich.

## ADR-012 – Reproduzierbare P0-Werkzeugketten

Datum: 18.10.2026. Status: für P0 entschieden.
**Entscheidung:** Rust 1.99.1, Node.js 24.19.0 und npm 12.2.0 sind festgelegt.
Cargo.lock und package-lock.json werden versioniert. Frontend-Pakete erhalten exakte Versionen.
Die Versionen wurden gegen die offiziellen Paketquellen aufgelöst und werden mit Builds geprüft.
**Alternative:** Veränderliche Werkzeugversionen ohne Lockfiles wurden verworfen.
**Folge:** Aktualisierungen brauchen einen geprüften Pull Request.
Die Versionen von PostgreSQL, Redis und Infrastruktur bleiben O-006.

## ADR-013 – P0-Prüfschritte ohne Veröffentlichungsrechte

Datum: 18.10.2026. Status: für P0 entschieden.
**Entscheidung:** Die automatisierte Integration verwendet nur Leserechte,
auf Commit-IDs festgelegte Actions und blockierende Build-/Sicherheitsprüfungen.
P0 hat keinen Veröffentlichungsworkflow. npm-Installationsskripte bleiben deaktiviert.
**Alternative:** Ein früher Tag-/Image-Workflow wurde verworfen. P0 kann die
vollständige Abnahme für 1.0 nicht leisten.
**Folge:** Fehlende spätere Prüfschritte bleiben offen.
KC-038 weist GitHub-Regeln und Review-Verantwortung nach.
Die Ziele aus ADR-003 und ADR-004 bleiben bestehen.

## Bestätigte Architekturentscheidungen für KC-002

Stand: 18.10.2026. Die Projektleitung hat im Projektchat ausdrücklich
`1A, 2A, 3A, 4A` gewählt. Die folgenden Architekturentscheidungen sind damit bestätigt.

- [ADR-014 – Datenhoheit und Migrationen](architecture/ADR-014_Datenhoheit_und_Migrationen.md):
  Ein Einrichtungsschema mit getrennten Dienstrechten und Migrationsrollen (`1A`).
- [ADR-015 – Gateway-Grenze](architecture/ADR-015_Gateway_Grenze.md):
  Eigenständiges Rust-Gateway (`2A`).
- [ADR-016 – Personalbereich](architecture/ADR-016_Personalbereich.md):
  Gesonderter Personalbereich je Träger; eigene Personalbereiche für trägerlose Einrichtungen (`3A`).
- [ADR-017 – Wahltrennung](architecture/ADR-017_Wahltrennung.md):
  Getrennte Datenbanken und Dienstidentitäten für Wahlberechtigung und Stimme (`4A`).

Die [Dienstmatrix](architecture/KC-002_Dienstgrenzen.md) ergänzt Entitäten,
Kommunikationszwecke und negative Testanforderungen.
Die unabhängige Prüfung des Wahlprotokolls ist vor dessen Implementierung verpflichtend.
Die Entscheidungen belegen keine implementierte Sicherheit und erteilen keinen Auftrag für P1 bis P9.


## ADR-018 – Reproduzierbare Testdaten und RustSec-Prüfung in P0

Datum: 18.10.2026. Status: im autorisierten technischen Umfang von KC-003 festgelegt.
**Grundlage:** KC-003 verlangt synthetische Testdaten und Build-/Sicherheitsprüfungen.
**Entscheidung:** Ein lokaler Generator erzeugt ausschließlich feste Organisationskennungen.
CI prüft die Ausgabe bytegenau und weist unbekannte Felder sowie ungültige Zuordnungen ab.
`cargo-audit` 0.22.2 prüft die Cargo.lock gegen RustSec. Version und Archivprüfsumme sind festgelegt.
Bekannte RustSec-Schwachstellen und die mit `--deny warnings` erfassten Warnungen blockieren den Scannerprozess.
**Alternative:** Eine dauerhafte Sperre aller externen Rust-Abhängigkeiten erfüllt die spätere Abhängigkeitsprüfung nicht.
**Folge:** Der pauschale Sperrprüfschritt entfällt. Sicherheitsprüfung, Lockfile und Leserechte bleiben verpflichtend.
Die [Prüfmatrix](../tests/README.md) beschreibt Befundbehandlung und Grenzen.

## Technische Konkretisierung und bestätigte Ziele aus KC-037

Stand: 18.10.2026. Die Projektleitung hat nach den Rückfragen die autonome Weiterarbeit beauftragt.

- [ADR-019 – Dienstidentitäten](architecture/ADR-019_Dienstidentitaeten.md):
  Kurzlebige Diensttokens, mTLS und aktuelle serverseitige Token-/Rechteprüfung im delegierten technischen Umfang.
- [ADR-020 – Hintergrundaufträge](architecture/ADR-020_Hintergrundauftraege.md):
  Redis Streams mit PostgreSQL als dauerhafter Auftragsquelle und idempotenter Verarbeitung.

Das [KC-037-Register](17_P0_Entscheidungsregister.md) enthält das ausdrücklich bestätigte Lastprofil,
recherchierte Versionsstände und begründete späteste Prüftermine.
Die Projektleitung lässt Rechteinhaber, juristische Prüfung und unabhängige Wahlprüfung ausdrücklich offen.
Sie hat `@itsh-neumeier Timo Neumeier` als vorläufigen CODEOWNER bestätigt.
Keine dieser Entscheidungen ersetzt die erforderlichen unabhängigen Prüfungen oder eine Lizenzfreigabe.

## ADR-021 – Verbindliche Reviews und Main-Branchschutz

[ADR-021](architecture/ADR-021_Branchschutz_und_Review.md) dokumentiert den aktiven Main-Regelsatz aus KC-038.
Er verlangt Reviews und die fünf aktuellen Pflichtprüfungen ohne eingetragene Merge-Ausnahme.
Unabhängige Prüfer bleiben bis zur ausdrücklichen Benennung offen.

## Interview zur Fortsetzung von KC-036 und KC-038

Datum: 18.10.2026. Quelle: ausdrückliche Antwort im Projektchat.

- Frage 1: „Ja“ bestätigt die Klarstellung F-001. Die Berechtigung wird serverseitig vor dem Speichern geprüft.
- Frage 2: `itsh-neumeier` ist als PR-Prüfer genannt. Derselbe Benutzer ist Autor der aktuellen PRs.
  Die Benennung erfüllt dort keine unabhängige Genehmigung. Die Klärungsfrage bleibt offen.
- Frage 3: „Ich auch als Prüfer“ benennt Timo Neumeier für Sicherheits- und Datenschutzprüfungen.
  Die Zuständigkeit ersetzt keine durchgeführte Prüfung und belegt keine Unabhängigkeit bei eigenen Änderungen.

Die Review-Regeln bleiben bestehen. Eine Ausnahme oder Selbstfreigabe wurde nicht bestätigt und wird nicht eingerichtet.
Die gesonderte juristische Lizenzprüfung und unabhängige Wahlprotokollprüfung bleiben unverändert offen.
Es wurde keine neue Architekturentscheidung getroffen.

## Dokumentationssprache (Ergänzung v5.9)

Es gelten die [deutsche Dokumentationsrichtlinie](13_Dokumentationsrichtlinie_DE_ASD_STE100.md)
und das [Fachwortverzeichnis](14_Fachwortverzeichnis_DE.md).
Die Richtlinie adaptiert Sprachprinzipien von ASD-STE100.
Sie ist keine offizielle deutsche Übersetzung und belegt keine formelle ASD-STE100-Konformität.

## ADR-010 – Deutsche technische Dokumentationssprache (v5.9)

**Entscheidung:** Alle deutschsprachigen Projektunterlagen folgen einer kontrollierten technischen Sprache nach den Prinzipien von ASD-STE100.
**Abgrenzung:** ASD-STE100 ist ein englischsprachiger Standard.
Die deutsche KitaConnect-Richtlinie ist eine eigene Adaption. Sie ist keine offizielle Übersetzung und keine formelle Konformitätsaussage.
**Umsetzung:** Dokumentationsrichtlinie und Fachwortverzeichnis sind verbindlich.
KC-036 dokumentiert die redaktionelle Prüfung des Bestands als eigenes Arbeitspaket.
