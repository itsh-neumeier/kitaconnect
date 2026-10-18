# Entscheidungs- und Konfliktprotokoll

**KitaConnect Spezifikation v5.9 · 17.10.2026 · Konsolidierter Planungsstand**

> Status: Anforderungsspezifikation, keine implementierte oder rechtsverbindlich geprüfte Software. Die neuesten ausdrücklichen Interviewentscheidungen gelten.

## Konfliktregel
Neueste **ausdrückliche** Interviewentscheidung hat Vorrang. Änderungen werden protokolliert; rechtliche/sicherheitskritische Widersprüche nicht stillschweigend übergangen.

| ID | Frühere Annahme | Aktuelle Entscheidung | Folge |
|---|---|---|---|
| ADR-001 | Modularer Monolith | Frühe Microservices (Q47) | Dienstgrenzen, API-Contract-Tests, Orchestrierung |
| ADR-002 | Gemeinsames Schema mit `tenant_id` + RLS | Schema je Mandant + RLS (Q47) | Schema-Migrationsorchestrator und Pool-Isolation |
| ADR-003 | Release manuell durch Owner | Automatisch nach verpflichtender CI (Q50/51) | Protected Branch, Reviews, Release-Gates |
| ADR-004 | Erste Stable mit allen Modulen | Q51: Kernmodule zuerst entwickeln, **vollständiger Umfang für 1.0** | Interne Phasen, keine öffentliche Teilversion |
| ADR-005 | Elternbeirat ohne Krank-/Abwesenheit/Abholrechte | Vollständiges Kita-Modul für Anwesenheit/Abholung (Q42/45) | Elternbeirat-only ohne Kinderakte weiterhin möglich |
| ADR-006 | Fotoeinwilligung nach Verwendungszweck | Zusätzlich pro Einzelbild/Veröffentlichung (Q44) | Consent-Matrix + Publish-Gate |
| ADR-007 | Essenspreise und Abrechnung in Funktionsliste | Explizit **keine Abrechnung** im Essensmodul (Q38) | Preise/Reports ja, Rechnungen/Payments nein |
| ADR-008 | Eltern-/Kita-Bereiche getrennt | Gemeinsame Prozesse bei strikter Datengrenze (Q41) | scoped sharing, keine impliziten Rechte |
| ADR-009 | Personalverwaltung lokal | Trägerübergreifend konfigurierbar (Q40/43) | globaler HR-Scope, Einsätze je Einrichtung |

## Noch fachlich/rechtlich zu validieren (keine offenen Grundsatzentscheidungen)
- Datenschutzrechtliche Verantwortlichkeit, Rechtsgrundlagen und landesspezifische Aufbewahrungsfristen.
- Verbindliche Betreuungsschlüssel und Arbeitszeit-/Pausenregelwerke pro Region/Träger.
- Wahlordnungen, Stimmgewicht und unabhängige Sicherheitsprüfung des Wahlprotokolls.
- Rechtswirksamkeit elektronischer Vertragsabschlüsse und Dokumentenunterzeichnung.
- Konkrete Lastziele, Infrastrukturgrößen und RPO/RTO-Nachweis.
- Proprietärer Lizenztext und Weitergabe-/Self-Hosting-Rechte juristisch prüfen.
- Exakte Definition aller Fachakzeptanzkriterien auf Issue-Ebene vor Implementierung.

## Status
Fachliche Interviews 1–51 abgeschlossen. Die Dateien sind ein konsolidierter Planungsstand, keine rechtliche Zertifizierung und keine implementierte Anwendung.

## ADR-011 – P0-Struktur und dienstbezogene Verantwortung

Datum: 18.10.2026. Status: für das technische P0-Grundgerüst entschieden.
Grundlage: ADR-001, ADR-002 und Auftrag zur Projektinitialisierung.

**Entscheidung:** Rust-Pakete liegen unter `services/`, technische Bibliotheken
unter `crates/`, das SvelteKit-Frontend unter `apps/web/`.
Verträge liegen unter `contracts/`, Infrastruktur unter `infra/`.
P0 enthält nur ein Rust-Werkzeugkettenprogramm und eine Frontend-Hinweisseite.
**Alternative:** Ein gemeinsames Fachpaket wurde verworfen. Es würde Datenhoheit
und frühe Microservice-Grenzen verschleiern.
**Folge:** Die Dienstmatrix im [P0-Plan](15_P0_Initialisierung.md) bleibt ein Entwurf.
KC-002 bestätigt Datenhoheit vor Fachimplementierung. Schema je Mandant und RLS bleiben verbindlich.

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


## Dokumentationssprache (Ergänzung v5.9)
Für dieses Dokument gilt die deutsche Projektrichtlinie `13_Dokumentationsrichtlinie_DE_ASD_STE100.md` mit dem verbindlichen Fachwortverzeichnis `14_Fachwortverzeichnis_DE.md`. Die Richtlinie orientiert sich an ASD-STE100, stellt aber **keine offizielle deutsche ASD-STE100-Konformität** dar.

## ADR-010 – Deutsche technische Dokumentationssprache (v5.9)
**Entscheidung:** Sämtliche deutschsprachigen Projektunterlagen folgen einer kontrollierten technischen Sprache nach den Prinzipien von ASD-STE100. **Abgrenzung:** ASD-STE100 ist ein englischsprachiger Standard. Die deutsche KitaConnect-Richtlinie ist eine eigene Adaption, keine offizielle Übersetzung oder formelle Konformitätsaussage. **Umsetzung:** Dokumentationsrichtlinie und Fachwortverzeichnis sind verbindlich; redaktionelle Überarbeitung des Bestands folgt als eigenes Arbeitspaket.
