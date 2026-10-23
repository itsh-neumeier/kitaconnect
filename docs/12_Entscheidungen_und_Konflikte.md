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
Der ursprüngliche Regelsatz verlangte Reviews und die fünf aktuellen Pflichtprüfungen ohne Ausnahme.
ADR-022 schreibt die Einrichtung nach der ausdrücklichen Wahl einer begrenzten Konto-Ausnahme fort.

## Interview zur Fortsetzung von KC-036 und KC-038

Datum: 18.10.2026. Quelle: ausdrückliche Antwort im Projektchat.

- Frage 1: „Ja“ bestätigt die Klarstellung F-001. Die Berechtigung wird serverseitig vor dem Speichern geprüft.
- Frage 2: `itsh-neumeier` ist als PR-Prüfer genannt. Derselbe Benutzer ist Autor der aktuellen PRs.
  Die Benennung erfüllt dort keine genehmigende GitHub-Prüfung.
- Frage 3: „Ich auch als Prüfer“ benennt Timo Neumeier für Sicherheits- und Datenschutzprüfungen.
  Die Zuständigkeit ersetzt keine durchgeführte Prüfung und belegt keine Unabhängigkeit bei eigenen Änderungen.

Zu diesem Interviewstand blieben die Review-Regeln bestehen. Die spätere ausdrückliche Zielregel steht in ADR-022.
Die gesonderte juristische Lizenzprüfung und unabhängige Wahlprotokollprüfung bleiben unverändert offen.
Es wurde keine neue Architekturentscheidung getroffen.

Ergänzende Antwort: „Ich prüfe die auch sowie du als KI-Assistent“.
Die Projektleitung übernimmt persönliche Prüfung. Der KI-Assistent übernimmt technische Vorprüfung.
Die aktuellen PRs wurden über den Zugang der Projektleitung erstellt.
Ihre persönliche Prüfung und das technisch erforderliche genehmigende GitHub-Review bleiben getrennte Nachweise.
Es wird keine abgeschlossene Prüfung oder erfüllte Pflichtgenehmigung behauptet.
Die Aufteilung allein verändert weder Branchschutz noch unabhängige Fachprüfungen.

## ADR-022 – Zielregel für eigene Pull Requests

Die Projektleitung hat anschließend ausdrücklich eine Reviewausnahme nur für eigene PRs von `@itsh-neumeier` bestätigt.
Die anschließende ausdrückliche Auswahl lautet: „Einfache Konto-Ausnahme; Nutzung auf eigene PRs beschränken“.
[ADR-022](architecture/ADR-022_Reviewausnahme_eigene_PRs.md) dokumentiert die jetzt aktive Aufteilung:
Basisschutz `25341109` ohne Ausnahme; Review-Regelsatz `25346711` mit Ausnahme nur für Benutzer-ID `77058548` über PRs.
Tests und Sicherheitsprüfungen bleiben verpflichtend. Die Beschränkung auf eigene PRs ist eine Nutzungsregel.
GitHub würde die Konto-Ausnahme technisch auch bei fremden PRs zulassen. Diese Grenze wurde ausdrücklich akzeptiert.

## Freigabe und Integration der P0-PRs

Datum: 18.10.2026. Die Projektleitung hat mit „Ja“ die persönliche/sprachliche Durchsicht
und die angebotene Integration der PRs #39 bis #44 bestätigt.
Die Integration erfolgte mit aktualisierten Basen und erfolgreichen Pflichtprüfungen.
Der [Integrationsnachweis](21_P0_Integration_und_Abnahmevorbereitung.md) enthält die Merge- und Main-CI-Stände.
Die P0-Gesamtabnahme wurde zur Vorbereitung beauftragt, aber noch nicht erklärt.
Es gibt keinen Auftrag zur Implementierung von P1.

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

## P0-Abnahme und P1-Auftrag

Datum: 18.10.2026. Ausdrückliche Antwort im Projektchat: „P0 abnehmen, und P1 freigeben“.
Die Projektleitung nimmt P0 auf Grundlage der erfolgreichen Prüfungen und dokumentierten Zurückstellungen ab.
Sie beauftragt P1. Die Arbeit beginnt mit kleinen Arbeitspaketen aus KC-004.
Die juristische Lizenzprüfung, die unabhängige Wahlprüfung und alle Sperren aus KC-037 bleiben bestehen.
Der Abschlussnachweis in PR #49 wird nach erfolgreicher CI integriert.

## ADR-023 – Erstes P1-Modell

[ADR-023](architecture/ADR-023_Organisationsmodell.md) legt das interne Organisationsmodell im zuständigen Dienst fest.
KC-004A beginnt mit typisierten Kennungen und Modellinvarianten. Persistenz und aktuelle Benutzerrechte folgen in getrennten Arbeitspaketen.
Diese Entscheidung gewährt keine neuen Berechtigungen und ersetzt keine Dienst- oder Datenbanktests.

## ADR-024 – Organisationspersistenz ohne vorgezogene Rechtefreigabe

Am 19.10.2026 konkretisiert [ADR-024](architecture/ADR-024_Organisationspersistenz.md) den Speichervertrag, getrennte Datenbankrollen und transaktionssichere Migrationen.
Die Laufzeit bleibt bis zur aktuellen Identitäts-/Policy-Prüfung durch RLS gesperrt. Echte PostgreSQL-Tests laufen innerhalb der verpflichtenden Rust-CI.
Die technische Festlegung verändert keine fachlichen Zugriffsrechte.

## ADR-025 – SQLx-Adapter

[ADR-025](architecture/ADR-025_SQLx_Adapter.md) ergänzt am 19.10.2026 die Rust-Anbindung für KC-004B.
Der Adapter verwendet gebundene Parameter, exakte Kennungswerte und bestehende Verbindungen.
Er führt keine Rollenwechsel, Migrationen oder Rechtevergabe aus. Die RLS-Sperre bleibt unverändert.
Die vorhandenen PostgreSQL-Tests prüfen ihn vor und nach einem echten Server-Neustart.

## ADR-026 – Einrichtungsschema-Registry und technische Reihenfolge

[ADR-026](architecture/ADR-026_Schema_Provisionierung.md) konkretisiert am 20.10.2026 atomare Wartungsoperationen für leere Einrichtungsschemas.
Wartungsrechte bleiben getrennt von Laufzeitrechten. Die Schema-Registry wird mit einer neuen Migration ergänzt.
Der Backlog trennt Grundmodell und Gesamtabnahme von KC-004; Teilpaketabhängigkeiten werden einschließlich Gesamtabnahme auf Zyklen geprüft.
Die ursprünglichen Fach- und Abnahmekriterien bleiben bestehen.

## ADR-027 – Technischer Nachweis der Dienstisolation

[ADR-027](architecture/ADR-027_Dienstisolation_Testaufbau.md) legt für KC-005B synthetische Proben mit getrennten Dienstrechten und erzwungener RLS fest.
Die Bindung an Test-Loginrollen gilt nur im isolierten Container. Produktive Identitätsnachweise und erlaubende Policies bleiben gesondert zu entscheiden.
Es werden keine Fachobjekte oder produktiven Zugänge angelegt.

## ADR-028 – Pool- und Wartungsnachweise

[ADR-028](architecture/ADR-028_Pool_und_Wartungstests.md) konkretisiert die technischen Tests aus KC-005C.
Wiederverwendung wird anhand derselben Datenbankprozesskennung geprüft. Konkurrierende Wartung muss beobachtbar an PostgreSQL-Sperren warten.
Testmigrationen bleiben von produktiver Provisionierung getrennt. Die produktive Identitäts-/Poolbindung bleibt offen.

## ADR-029 – Identitäts- und Sitzungsmodell

[ADR-029](architecture/ADR-029_Identitaets_und_Sitzungsmodell.md) beginnt KC-006 mit einem internen Modell im zuständigen Dienst.
Die sechs Teilpakete erhalten alle ursprünglichen Anmelde- und Widerrufsanforderungen.
Konkrete Sitzungswerte, MFA-Zuordnung und Wiederherstellung bleiben bis zur ausdrücklichen Antwort offen.
Das Modell erhält keine eingebauten Vorgabewerte und erteilt keine Anmeldung oder Berechtigung.

## ADR-030 – Sitzungsbestand und Widerruf

[ADR-030](architecture/ADR-030_Sitzungsspeicher.md) konkretisiert den internen Sitzungsbestand in KC-006B.
Zufällige Nachweise bleiben getrennt von Datensatzkennungen. Die Datenbank speichert nur deren Ableitung.
Aktuelle Prüfung und Widerruf konkurrieren über dieselbe Zeile. Die produktive Anwendungsrolle bleibt durch RLS gesperrt.

## ADR-031 – Passwortprüfung und dauerhafte Versuchslimits

[ADR-031](architecture/ADR-031_Passwortpruefung.md) begrenzt Argon2id-Parameter und parallele Hasharbeit für KC-006C.
Versuche werden vor der Prüfung dauerhaft reserviert. Produktive Grenzwerte werden nicht vorgegeben.
Ein positives Passwortergebnis ersetzt weder MFA noch eine aktuelle Freigabe. Die offenen Interviewfragen bleiben offen.

## ADR-032 – WebAuthn-Vorbereitung ohne vorgezogene Sicherheitsfreigabe

[ADR-032](architecture/ADR-032_WebAuthn_Vorbereitung.md) dokumentiert die Bibliotheksprüfung und den Vertragsentwurf für KC-006D.
RP-ID/Ursprung, Benutzerverifikation und synchronisierte Passkeys sind ausdrücklich angefragt.
Die Rückfragen sind offen. Der Entwurf enthält keine neue Laufzeitimplementierung oder bestätigte Passkey-Regel.

Am 20.10.2026 bestätigt die Projektleitung ausdrücklich alle drei WebAuthn-Vorschläge.
Für KC-006D gelten ausschließlich localhost im isolierten Test, erforderliche Benutzerverifikation und erlaubte synchronisierte Passkeys.
Produktive Domain und die übrigen offenen Sicherheitsentscheidungen bleiben offen. Vertrag und ADR-032 werden vor Implementierung aktualisiert.

## ADR-033 – TOTP und Wiederherstellung vorbereiten

[ADR-033](architecture/ADR-033_TOTP_und_Wiederherstellung.md) dokumentiert die Bibliotheksprüfung und den Vertragsentwurf für KC-006E.
MFA-Zuordnung und Wiederherstellung wurden gemeinsam erneut zur ausdrücklichen Entscheidung gestellt.
Die Benachrichtigungsregel bei Faktoränderungen und Wiederherstellung ist zusätzlich angefragt.
Alle Antworten stehen aus. Der Entwurf aktiviert keine dieser Regeln und enthält keine Laufzeitimplementierung.

Am 21.10.2026 bestätigt die Projektleitung ausdrücklich alle vorgelegten MFA-/Wiederherstellungs- und Benachrichtigungsregeln.
Vertrag und ADR-033 werden vor Code aktualisiert. Rechtezuordnung, Browser-Sitzungswerte und produktiver Schlüsselbetrieb bleiben gesondert.

## ADR-034 – Browseranmeldung und Nachweisübergabe vorbereiten

[ADR-034](architecture/ADR-034_Browseranmeldung_und_Nachweisuebergabe.md) beginnt KC-006F mit einem begrenzten Vertragsentwurf.
Der Entwurf behandelt atomare Nachweisübergabe, Sitzungserzeugung, aktuelle Mitgliedschaft und die Gateway-Grenze.
Er ändert keine Datenhoheit: identity-access verwaltet Identitäten und Sitzungen; organization-config verwaltet Mitgliedschaften.
Eine lokale Transaktion ersetzt keine aktuelle dienstübergreifende Prüfung.
Sitzungsfristen, die Erfüllung der MFA-Pflicht durch Passkeys und Cookie-/CSRF-Regeln sind ausdrücklich angefragt und noch offen.
Keine abhängige Laufzeitfunktion, Migration oder produktive Freigabe wird daraus abgeleitet.

Am 21.10.2026 bestätigt die Projektleitung beide MFA-Varianten und den Cookie-/CSRF-Vorschlag.
Die Fristen sind pro Einrichtung einstellbar. In einer Folgeantwort bestätigt die Projektleitung die vorgeschlagenen Voreinstellungen und Obergrenzen.
[ADR-035](architecture/ADR-035_Sitzungsfristen_je_Einrichtung.md) und der Einstellungsspeichervertrag werden vor Code versioniert.

## ADR-036 – Einmaliger Anmeldeabschluss

[ADR-036](architecture/ADR-036_Einmaliger_Anmeldeabschluss.md) konkretisiert die atomare Übergabe aus bestehenden Faktorprüfungen.
Der lokale Sitzungsabschluss bindet Identität, Einrichtung, tatsächlichen Faktorbestand und ursprüngliche sowie aktuelle Fristen.
Ein gemeinsamer Sperrvertrag serialisiert Faktoränderungen und Abschluss. Organisation bleibt eine getrennte geprüfte Dienstgrenze.
Der Vertrag wird vor Code versioniert. Gateway, Cookies und vollständige Browser-/mTLS-Nachweise folgen weiter in KC-006F.

## ADR-037 – Aktuelle Sitzungsprüfung

[ADR-037](architecture/ADR-037_Aktuelle_Sitzungspruefung.md) konkretisiert aktuelle Prüfung und lokale Abmeldung.
Ein eigener Beobachtungsbestand erhält einmal verschärfte Grenzen dauerhaft. Die aktuelle Organisationsantwort bleibt verpflichtend.
Der Prüfvertrag wird vor Code versioniert. Rotation, erneute Faktorprüfung und Gateway folgen getrennt.

## ADR-038 – Atomare Sitzungsrotation

[ADR-038](architecture/ADR-038_Atomare_Sitzungsrotation.md) beschreibt den atomaren Nachweiswechsel innerhalb derselben Anmeldung.
Fristen und Authentisierungszeit bleiben erhalten. Abmeldung widerruft auch deren Rotationsnachfolger.
Der Rotationsvertrag wird vor Code versioniert. Browser- und Dienstidentitätsnachweise bleiben gesondert.

## ADR-039 – Zusätzliche Authentisierung

[ADR-039](architecture/ADR-039_Zweckgebundene_Zusatzauthentisierung.md) trennt zusätzliche Faktorprüfung und normale Anmeldung.
Der neue Zweck bindet die genaue Ziel-Sitzung. Abschluss und Nachweiswechsel sollen atomar sein; absolute Frist und Herkunft bleiben erhalten.
Der Vertrag wird vor Code versioniert. Es entsteht keine neue MFA-Variante oder fachliche Berechtigung.

## ADR-040 – Abschluss eigener Transaktionen

[ADR-040](architecture/ADR-040_Abschluss_eigener_Transaktionen.md) hält die Korrektur nach dem Rotationsnachweis fest.
Eigene Fehlerpfade warten Rollback ab. Bereits bestätigte Versuchslimits und dauerhafte Ablaufzustände bleiben erhalten.
Die weiteren Adapter werden in kleinen Paketen mit getrennten Datenbankzeugen geprüft.

## ADR-041 – Interner Mitgliedschaftsbestand

[ADR-041](architecture/ADR-041_Interner_Mitgliedschaftsbestand.md) konkretisiert die interne aktuelle Abfrage im Organisationsdienst.
Aktive Zuordnung und Einrichtungseinstellungen werden gemeinsam gelesen. Die Laufzeit bleibt durch RLS gesperrt.
Der Datenstand ersetzt weder MFA-/Rechteentscheidung noch geprüfte Dienstkommunikation.

## ADR-042 – Diensttoken-Grundlage

[ADR-042](architecture/ADR-042_Diensttoken_Grundlage.md) konkretisiert die bestätigte ADR-019 für den internen Tokenbestand.
Aktuelle Zustandsprüfung und echte mTLS-Prüfung erhalten getrennte Nachweise.
Produktiver Schlüsselbetrieb und Domain bleiben offen. Ein Diensttoken erteilt keine Benutzer- oder Fachrechte.

## Bestätigte MFA-Zuordnung für KC-008 und KC-006F

Am 21.10.2026 bestätigt die Projektleitung ausdrücklich „Diese Zuordnung bestätigen“.
Aktuelle zusätzliche MFA gilt für Rollen-/Rechteverwaltung, sicherheitsrelevante Einstellungen,
Gesundheitsdaten, Sorgerechtsbeschränkungen, Personalakten und interne Entwicklungsakten.
Unbekannte Einstufungen verweigern Zugriff. Normale Anmeldung allein gewährt keinen solchen Zugriff.
Veröffentlichte Portfolios und einfache Kontaktdaten behalten ihre gesonderten Objekt-/Feldrechte.
Diese Entscheidung gewährt keine neue Rolle oder Fachberechtigung. Der konkrete Umsetzungsvertrag folgt vor Code.

## ADR-043 – Aktionsgebundene MFA-Zuordnung

[ADR-043](architecture/ADR-043_Aktionsgebundene_MFA_Zuordnung.md) konkretisiert die bestätigte Zuordnung vor abhängigem Code.
Die tatsächliche Aktion bestimmt die aktuelle Authentisierungsanforderung. Unbekannte Einstufungen verweigern Zugriff.
Aktuelle MFA erteilt keine Objekt-/Feldrechte. Die Gesamtabnahme von KC-008 bleibt gesondert.

## ADR-044 – Gemeinsame mTLS-Grundlage

[ADR-044](architecture/ADR-044_Gemeinsame_mTLS_Grundlage.md) konkretisiert die gemeinsame tatsächliche Zertifikatsprüfung vor Code.
Die Metadaten stammen aus dem Handshake. Beide Endzertifikate werden auch auf offenen Verbindungen zeitlich geprüft.
Registrierung, Introspektion, Fachrechte und das noch offene Browserprofil bleiben getrennte Nachweise.

## Bestätigte Klarstellung: Fehlerformate der OAuth-Protokollendpunkte

Am 22.10.2026 bestätigt die Projektleitung ausdrücklich „Begrenzte OAuth-Ausnahme bestätigen“.
Die allgemeine Problem-Details-Regel aus Spezifikation v5.9 / Dokument 04 erhält ausschließlich für OAuth-Protokollendpunkte eine Ausnahme.
Diese Endpunkte verwenden ihre protokollspezifischen standardisierten Fehlerformate.
Die Fach-APIs behalten Problem-Details nach RFC 9457. Keine fachliche Zugriffsregel wird geändert.
Grundlage: [RFC 6749, Abschnitt 5.2](https://www.rfc-editor.org/rfc/rfc6749#section-5.2) verlangt das OAuth-JSON-Fehlerformat.
[RFC 9457](https://www.rfc-editor.org/rfc/rfc9457) beschreibt Problem-Details mit eigenem Medientyp.
Die Klarstellung wird vor abhängigem HTTP-Code dokumentiert. Andere Antworten und das offene Browserprofil bleiben getrennt.

## ADR-045 – HTTP-Verbindungsbindung

[ADR-045](architecture/ADR-045_HTTP_Verbindungsbindung.md) dokumentiert den geteilten Prüfzugang vor Code.
Handler und Datenstrom erhalten denselben Zeit- und Sperrzustand der tatsächlichen mTLS-Verbindung.
Der vollständige OAuth-Endpunktvertrag, aktive Registrierung und Fachrechte folgen getrennt.
