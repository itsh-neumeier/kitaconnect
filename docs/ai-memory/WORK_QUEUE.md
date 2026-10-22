# Arbeitsfolge für den offenen Backlog

Stand: 21.10.2026. Quelle: aktuelle offene GitHub-Issues und versionierter Abhängigkeitsplan.
Die Projektleitung beauftragt ausdrücklich die automatische weitere Bearbeitung aller offenen Issues und PRs.
Die Arbeit erfolgt weiter in kleinen abhängigen Paketen. Erledigte Implementierungen werden nicht wiederholt.
Diese Liste ist eine Momentaufnahme und kein Abschlussnachweis. GitHub bleibt verbindlich.

## Aktueller Schritt

KC-006F / #67: PR #76 bestätigt aktuelle Sitzungsprüfung und lokale Abmeldung mit zehn grünen Prüfungen.
PR #77 / 5927631 bestätigt Rotation und deren Fehlerkorrektur mit zehn aktuellen grünen Prüfungen.
PR #78 / ad150c9 und #79 / 088d24f korrigieren die eigenen Fehlertransaktionen in allen geprüften Adaptern; jeweils zehn aktuelle Prüfungen grün.
PR #80 / 5a4d5aa bestätigt die Speichergrundlage nach Vertrag/ADR-039 mit zehn grünen Prüfungen.
PR #81 / 7813bcb bestätigt die tatsächlichen Bibliotheksprüfungen und den atomaren Zusatzabschluss mit zehn grünen Prüfungen.
PR #82 / 9ad5c23 bestätigt aktuellen Mitgliedschafts-Lesebestand mit zehn grünen Prüfungen vor/nach Neustart.
PR #83 / 00bad74 bestätigt internen Diensttokenbestand und Datenbanknachweise nach ADR-019/042 mit zehn grünen Prüfungen.
PR #84 / 06e28bb bestätigt die MFA-Zuordnung mit allen zehn aktuellen Prüfungen einschließlich beider Neustartphasen.
PR #85 / 644925a bestätigt die tatsächliche mTLS-Grundlage mit zehn grünen Prüfungen.
PR #86 dokumentiert die notwendige Sicherheits-Rückübernahme in 21 ältere eigene PRs.
Alle 21 Tokio-Patches und sieben zusätzliche TLS-Sicherheits-Patches sind veröffentlicht und bestehen jeweils alle zehn aktuellen Prüfungen. Zwei normale Basis-Merges bewahren identische Dateibäume und beheben Manifest-Konflikte. Aktuelle Köpfe und vollständige Laufnachweise stehen in TOKIO_BACKPORT_STATE.json.
Der Nachweiskopf von PR #86 / cb715a3 ist erneut vollständig grün und konfliktfrei bestätigt. Erledigte Rückübernahmen nicht wiederholen.
PR #87 auf Folgebranch codex/kc-006f-http-verbindungsbindung ergänzt nach Vertrag/ADR-045 in 1624964 den aktuellen Prüfzugang für spätere Handler derselben mTLS-Verbindung. Codekopf ff9ac5e besteht native PR-CI mit sechs tatsächlichen mTLS-Tests, zwei getrennten Zeitfehlerprüfungen und allen sechzehn SQLx-Gruppen vor/nach Neustart. Der abschließende Dokumentationskopf erhält eigene CI. Es gibt noch keinen HTTP-Router; vollständiger Endpunktvertrag mit OpenAPI, Registrierung, Formular-/Fehlerzuordnung und Grenzen folgt vor Routercode.
Die begrenzte OAuth-Fehlerformatausnahme ist ausdrücklich bestätigt und vor HTTP-Code dokumentiert. Nicht erneut fragen.
PR #88 / 267aef4 ergänzt Vertrag/OpenAPI/ADR-046 in 7e072be und die ausdrückliche DNS-/Fingerabdruckbestätigung in 10f8565 vor Code. Alle zehn aktuellen Prüfungen sind grün; sieben Protokolltests und sechzehn SQLx-Gruppen vor/nach Neustart bestehen. DNS-Name plus aktiver genauer Fingerabdruck ist verbindlich; nicht erneut fragen.
Aktiver Folgebranch codex/kc-006f-dienstanmeldung / PR #89 ergänzt nach Vertrag/ADR-047 in 24324ca die tatsächliche SAN-Prüfung und aktuellen lesenden Registrierungsadapter. Codekopf c4dcfd6 besteht native Push-CI mit allen siebzehn SQLx-Gruppen vor/nach Neustart. Der abschließende Kopf ergänzt Common-Name-Negativfall und Fortsetzungsnachweis; aktuelle CI im PR prüfen. Keine neue Migration, produktive Namensvorgabe oder erlaubende RLS-Policy. Danach Ausgaberestlaufzeit, grobe Ablehnungs-/Ausfallfehler und tatsächliche Axum-Endpunkte ergänzen.
Aktiver Folgebranch ist nun codex/kc-006f-ausgabemetadaten auf #89 / 21f97bd. Vertrag/ADR-048 in 7e29e5e steht vor Code. Die erweiterte bestätigte Ausgabe, aktuelle Restlaufzeit und grobe Ablehnungs-/Ausfallfehler sind vorbereitet; native Prüfung folgt. Keine Migration oder Paketänderung. Danach tatsächliche Axum-Endpunkte mit getrennten Router-/TLS-/Datenbanknachweisen anschließen.
Die Aktion stammt vom zuständigen Dienst; unbekannte Einstufung verweigert Zugriff. Mitgliedschaft und MFA allein gewähren keine Fachrechte.
Die früheren SQL-Tests mit synthetischen Faktor-Markierungen bleiben als Speicherprüfungen getrennt ausgewiesen.
Danach folgen HTTP-/Introspektionsvertrag und der echte Organisations-/Gateway-Vertrag. MFA-Einstufung und zusätzliche Faktorprüfung sind mit PRs #81/#84 bereits technisch geprüft.
Die 17 ursprünglichen PRs sind technisch durchgesehen. Korrektur-PR #75 ist vollständig grün.
Die gesammelte persönliche und sprachliche Durchsicht ist noch nicht erfolgt. Daraus keinen Merge-Auftrag oder erfülltes Review ableiten.
#47/#48 behalten die begründete Ablehnungsempfehlung. Die automatisierte Arbeit an anderen freigegebenen Schritten läuft weiter.

## Offene Issues

| GitHub | Kennung | Phase | Bearbeitungsstand | Versionierte Voraussetzungen |
|---|---|---|---|---|
| [#4](https://github.com/itsh-neumeier/kitaconnect/issues/4) | KC-004 | P1 | Voraussetzungen oder spätere Umsetzung offen | KC-001, KC-002, KC-003, KC-037, KC-038 |
| [#5](https://github.com/itsh-neumeier/kitaconnect/issues/5) | KC-005 | P1 | Voraussetzungen oder spätere Umsetzung offen | KC-004A, KC-004B |
| [#6](https://github.com/itsh-neumeier/kitaconnect/issues/6) | KC-006 | P1 | Voraussetzungen oder spätere Umsetzung offen | KC-004A, KC-004B |
| [#7](https://github.com/itsh-neumeier/kitaconnect/issues/7) | KC-007 | P1 | Voraussetzungen oder spätere Umsetzung offen | KC-006 |
| [#8](https://github.com/itsh-neumeier/kitaconnect/issues/8) | KC-008 | P1 | Voraussetzungen oder spätere Umsetzung offen | KC-005, KC-006 |
| [#9](https://github.com/itsh-neumeier/kitaconnect/issues/9) | KC-009 | P1 | Voraussetzungen oder spätere Umsetzung offen | KC-008 |
| [#10](https://github.com/itsh-neumeier/kitaconnect/issues/10) | KC-010 | P1 | Voraussetzungen oder spätere Umsetzung offen | KC-004, KC-008 |
| [#11](https://github.com/itsh-neumeier/kitaconnect/issues/11) | KC-011 | P2 | Voraussetzungen oder spätere Umsetzung offen | KC-008, KC-009 |
| [#12](https://github.com/itsh-neumeier/kitaconnect/issues/12) | KC-012 | P2 | Voraussetzungen oder spätere Umsetzung offen | KC-011 |
| [#13](https://github.com/itsh-neumeier/kitaconnect/issues/13) | KC-013 | P2 | Voraussetzungen oder spätere Umsetzung offen | KC-011 |
| [#14](https://github.com/itsh-neumeier/kitaconnect/issues/14) | KC-014 | P2 | Voraussetzungen oder spätere Umsetzung offen | KC-011, KC-012 |
| [#15](https://github.com/itsh-neumeier/kitaconnect/issues/15) | KC-015 | P3 | Voraussetzungen oder spätere Umsetzung offen | KC-008, KC-009, KC-011 |
| [#16](https://github.com/itsh-neumeier/kitaconnect/issues/16) | KC-016 | P3 | Voraussetzungen oder spätere Umsetzung offen | KC-015 |
| [#17](https://github.com/itsh-neumeier/kitaconnect/issues/17) | KC-017 | P3 | Voraussetzungen oder spätere Umsetzung offen | KC-015 |
| [#18](https://github.com/itsh-neumeier/kitaconnect/issues/18) | KC-018 | P3 | Voraussetzungen oder spätere Umsetzung offen | KC-013 |
| [#19](https://github.com/itsh-neumeier/kitaconnect/issues/19) | KC-019 | P4 | Voraussetzungen oder spätere Umsetzung offen | KC-011 |
| [#20](https://github.com/itsh-neumeier/kitaconnect/issues/20) | KC-020 | P4 | Voraussetzungen oder spätere Umsetzung offen | KC-019 |
| [#21](https://github.com/itsh-neumeier/kitaconnect/issues/21) | KC-021 | P4 | Voraussetzungen oder spätere Umsetzung offen | KC-019 |
| [#22](https://github.com/itsh-neumeier/kitaconnect/issues/22) | KC-022 | P5 | Voraussetzungen oder spätere Umsetzung offen | KC-008, KC-009, KC-011 |
| [#23](https://github.com/itsh-neumeier/kitaconnect/issues/23) | KC-023 | P5 | Voraussetzungen oder spätere Umsetzung offen | KC-022 |
| [#24](https://github.com/itsh-neumeier/kitaconnect/issues/24) | KC-024 | P5 | Voraussetzungen oder spätere Umsetzung offen | KC-023 |
| [#25](https://github.com/itsh-neumeier/kitaconnect/issues/25) | KC-025 | P5 | Fachprüfung vor Implementierung erforderlich | KC-008, KC-009, KC-011 |
| [#26](https://github.com/itsh-neumeier/kitaconnect/issues/26) | KC-026 | P5 | Voraussetzungen oder spätere Umsetzung offen | KC-022 |
| [#27](https://github.com/itsh-neumeier/kitaconnect/issues/27) | KC-027 | P6 | Voraussetzungen oder spätere Umsetzung offen | KC-011, KC-018 |
| [#28](https://github.com/itsh-neumeier/kitaconnect/issues/28) | KC-028 | P6 | Voraussetzungen oder spätere Umsetzung offen | KC-008 |
| [#29](https://github.com/itsh-neumeier/kitaconnect/issues/29) | KC-029 | P7 | Voraussetzungen oder spätere Umsetzung offen | KC-010 |
| [#30](https://github.com/itsh-neumeier/kitaconnect/issues/30) | KC-030 | P7 | Voraussetzungen oder spätere Umsetzung offen | KC-009, KC-020 |
| [#31](https://github.com/itsh-neumeier/kitaconnect/issues/31) | KC-031 | P7 | Voraussetzungen oder spätere Umsetzung offen | KC-005, KC-030 |
| [#32](https://github.com/itsh-neumeier/kitaconnect/issues/32) | KC-032 | P8 | Voraussetzungen oder spätere Umsetzung offen | KC-001 |
| [#33](https://github.com/itsh-neumeier/kitaconnect/issues/33) | KC-033 | P8 | Voraussetzungen oder spätere Umsetzung offen | KC-031, KC-032 |
| [#34](https://github.com/itsh-neumeier/kitaconnect/issues/34) | KC-034 | P8 | Voraussetzungen oder spätere Umsetzung offen | KC-001, KC-032 |
| [#35](https://github.com/itsh-neumeier/kitaconnect/issues/35) | KC-035 | P9 | Voraussetzungen oder spätere Umsetzung offen | KC-002, KC-034, KC-033, KC-030, KC-031 |
| [#50](https://github.com/itsh-neumeier/kitaconnect/issues/50) | KC-004A | P1 | PR vorbereitet; Integration und zugehörige Abnahme offen | KC-001, KC-002, KC-003, KC-037, KC-038 |
| [#51](https://github.com/itsh-neumeier/kitaconnect/issues/51) | KC-004B | P1 | PR vorbereitet; Integration und zugehörige Abnahme offen | KC-004A |
| [#52](https://github.com/itsh-neumeier/kitaconnect/issues/52) | KC-004C | P1 | Voraussetzungen oder spätere Umsetzung offen | KC-004B, KC-006, KC-008 |
| [#56](https://github.com/itsh-neumeier/kitaconnect/issues/56) | KC-005A | P1 | PR vorbereitet; Integration und zugehörige Abnahme offen | KC-004B |
| [#57](https://github.com/itsh-neumeier/kitaconnect/issues/57) | KC-005B | P1 | PR vorbereitet; Integration und zugehörige Abnahme offen | KC-005A |
| [#58](https://github.com/itsh-neumeier/kitaconnect/issues/58) | KC-005C | P1 | PR vorbereitet; Integration und zugehörige Abnahme offen | KC-005B |
| [#62](https://github.com/itsh-neumeier/kitaconnect/issues/62) | KC-006A | P1 | PR vorbereitet; Integration und zugehörige Abnahme offen | KC-004A, KC-004B |
| [#63](https://github.com/itsh-neumeier/kitaconnect/issues/63) | KC-006B | P1 | PR vorbereitet; Integration und zugehörige Abnahme offen | KC-006A |
| [#64](https://github.com/itsh-neumeier/kitaconnect/issues/64) | KC-006C | P1 | PR vorbereitet; Integration und zugehörige Abnahme offen | KC-006B |
| [#65](https://github.com/itsh-neumeier/kitaconnect/issues/65) | KC-006D | P1 | PR vorbereitet; Integration und zugehörige Abnahme offen | KC-006B |
| [#66](https://github.com/itsh-neumeier/kitaconnect/issues/66) | KC-006E | P1 | PR vorbereitet; Integration und zugehörige Abnahme offen | KC-006B, KC-006C, KC-006D |
| [#67](https://github.com/itsh-neumeier/kitaconnect/issues/67) | KC-006F | P1 | In Arbeit | KC-006C, KC-006D, KC-006E |

## Fortsetzungsregeln

1. Prüfen Sie zuerst den aktuellen GitHub- und Checkpoint-Stand.
2. Wählen Sie den nächsten Schritt nach den versionierten Voraussetzungen. Schließen Sie kein Gesamtissue nur aufgrund eines Teilnachweises.
3. Versionieren Sie den Dienstvertrag vor Code. Prüfen Sie lokale Tests und die passende native CI.
4. Fordern Sie offene fachliche oder sicherheitsrelevante Entscheidungen vor abhängiger Implementierung an. Bestätigte Antworten bleiben verbindlich.
5. Aktualisieren Sie Issue, PR und CURRENT_STATE nach jedem begrenzten Paket.
6. Beginnen Sie keine Wahlprotokollimplementierung ohne die vorgeschriebene unabhängige Fachprüfung.
   Veröffentlichungen, produktive Zugänge und rechtliche Freigaben werden aus diesem Arbeitsauftrag nicht abgeleitet.
