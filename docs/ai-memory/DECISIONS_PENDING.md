# Ausstehende Entscheidungen

Stand: 18.10.2026. Grundlage: Spezifikation v5.9.

Die vollständige Liste O-001 bis O-008 steht im [P0-Plan](../15_P0_Initialisierung.md).
Die Projektleitung MUSS Entscheidungen ausdrücklich bestätigen.
Eine ausbleibende Antwort ist keine Zustimmung.

| Frage aus dieser Sitzung | Stand |
|---|---|
| Persönliche/sprachliche Durchsicht und Integration | Die Projektleitung hat mit „Ja“ Durchsicht und Integration #39 bis #44 bestätigt. Alle sechs PRs sind integriert; Main-CI grün. Nicht erneut nach dieser Freigabe fragen. |
| P0-Gesamtabnahme und P1 | „P0 abnehmen, und P1 freigeben“ am 18.10.2026 ausdrücklich bestätigt. Nicht erneut abfragen. Abschlussnachweis #49 integrieren; mit KC-004 beginnen. |
| Review-Ausnahme | „Einfache Konto-Ausnahme; Nutzung auf eigene PRs beschränken“ ausdrücklich gewählt. Konto-Ausnahme für `@itsh-neumeier` im gesonderten Review-Regelsatz aktiv. „Nur eigene PRs“ ist eine bestätigte Nutzungsregel; technisch auch bei fremden PRs möglich. [ADR-022](../architecture/ADR-022_Reviewausnahme_eigene_PRs.md). Nicht erneut abfragen. |
| Pflichtprüfungen | Basisregelsatz `25341109` bleibt ohne Ausnahme. Review-Ausnahme steht nur in Regelsatz `25346711`. Beide nach Einrichtung per API nachgewiesen. Keine zusätzliche App oder neue Secrets erforderlich. |
| Referenzprofil für die Lastprüfung | Eine Einrichtung. 50 Sitzungen, 10 Anfragen pro Sekunde, 80/20 Lesen/Schreiben, p95 500/1.000 ms, unter 1 % Fehler, 4 vCPU, 8 GB RAM, 100 GB SSD, 5 Minuten Aufwärmen und 30 Minuten Messung ausdrücklich bestätigt. |
| CODEOWNER für das Repository | Antwort `@itsh-neumeier Timo Neumeier`; vorläufige Zuordnung in CODEOWNERS eingetragen. Kein Nachweis unabhängiger Prüfung. |
| PR-Prüfung durch Projektleitung und KI | Persönliche/sprachliche Prüfung durch Timo Neumeier und technische Vorprüfung durch den KI-Assistenten für #39 bis #44 bestätigt und dokumentiert. Die bestätigte Konto-Ausnahme wurde nur für diese eigenen PRs genutzt. |
| Sicherheits- und Datenschutzprüfung | Antwort „Ich auch als Prüfer“ benennt Timo Neumeier, `@itsh-neumeier`. Keine durchgeführte oder unabhängige Prüfung behauptet. Für eigene Änderungen bleibt unabhängige Gegenprüfung offen. |
| KC-036 / F-001: Tagesereignis | Antwort „Ja“ bestätigt Prüfung vor Speichern. Ablauf in Unterlage 05 klargestellt; F-001 entschieden. [Prüfbericht](../19_Redaktionspruefung_KC-036.md). |
| Rechteinhaber, juristische Lizenzprüfung und unabhängige Wahlprüfung | Antwort `offen lassen`. Projektleitung bleibt für Besetzung vor den gesperrten Aktionen zuständig. Keine automatische Zuordnung zum CODEOWNER. |
| npm audit und externe Paketmetadaten | Die Projektleitung hat die Übermittlung von Paketnamen und Versionen an registry.npmjs.org ausdrücklich erlaubt. Gilt lokal und in GitHub Actions. |

Die ausdrückliche Freigabe für npm audit liegt nach einer automatischen Ablehnung vor.
Die ursprüngliche Ablehnung betraf die Übermittlung von Abhängigkeitsmetadaten.
Sie betraf weder Quelltext noch produktive Zugangsdaten.

## Bestätigte Entscheidungen für KC-002

Die [Dienstmatrix](../architecture/KC-002_Dienstgrenzen.md) liegt mit Datenhoheit,
Kommunikationszwecken und Testanforderungen vor.
[Pull Request #40](https://github.com/itsh-neumeier/kitaconnect/pull/40)
ist ebenso wie der Grundgerüst-PR #39 integriert.

Die Projektleitung hat am 18.10.2026 im Projektchat ausdrücklich `1A, 2A, 3A, 4A` geantwortet.
Diese Antwort bestätigt die vier zuvor einzeln erläuterten Varianten.

| Entscheidung | Bestätigter Stand |
|---|---|
| O-001 / ADR-014 | `1A`: Ein PostgreSQL-Schema je Einrichtung mit getrennten Dienstrechten und Migrationsrollen. |
| O-002 / ADR-015 | `2A`: Eigenständiges Rust-Gateway. |
| O-001-HR / ADR-016 | `3A`: Gesonderter Personalbereich je Träger; Einrichtungen erhalten nur erlaubte Einsatzdaten. Trägerlose Einrichtungen behalten eigene Bereiche. |
| O-001-WAHL / ADR-017 | `4A`: Getrennte Datenbanken und Dienstidentitäten für Berechtigung und Stimme. Unabhängige Prüfung des Wahlprotokolls vor Implementierung bleibt verbindlich. |

Die Varianten sind damit entschieden. Die Implementierungsnachweise folgen in den zugeordneten Issues.
O-003 bis O-008 sind inzwischen im [KC-037-Register](../17_P0_Entscheidungsregister.md) konkretisiert.
Technische Festlegungen und bestätigte Ziele sind von den offenen Fachfreigaben getrennt.
KC-038 hat den [Main-Regelsatz](../18_Branchschutz_und_Reviews.md) eingerichtet und per API nachgewiesen.
ADR-022 dokumentiert die spätere, ausdrücklich bestätigte Konto-Ausnahme nur vom gesonderten Review-Regelsatz.
Fachprüfungen und persönliche Durchsicht bleiben erforderlich. Die Pflichtprüfungen erhalten keine Ausnahme.
P0 implementiert keine Dienste, Datenbankrollen oder Migrationen.

## Bestätigte Zurückstellungen und technische Fortsetzung

Die Projektleitung hat nach den Rückfragen autonome Weiterarbeit beauftragt.
ADR-019 und ADR-020 konkretisieren deshalb Dienstidentitäten und Hintergrundaufträge im bestehenden Sicherheitsumfang.
Das Register hält die recherchierten Infrastrukturstände mit Prüfpflicht vor tatsächlicher Verwendung fest.
Die Versionsmatrix behauptet keine getestete Installation oder rechtliche Freigabe.

Offen bleiben Rechteinhaber, juristische Prüfung und die unabhängigen Fachpersonen.
Die Zurückstellung ist ausdrücklich gewünscht. Sie hebt die Sperren aus dem Register nicht auf.
Vor Implementierung des Wahlprotokolls ist die unabhängige Prüfung weiterhin zwingend.

## Produktive RLS-Bindung nach KC-005B

Stand: 20.10.2026. [ADR-027](../architecture/ADR-027_Dienstisolation_Testaufbau.md) verwendet eigene Loginrollen ausschließlich im Testcontainer.
Vor einer produktiven erlaubenden Policy sind Identitätsnachweis, aktuelle Widerrufsprüfung und Bindung an den Verbindungspool ausdrücklich festzulegen.
Eine frei gesetzte Einrichtungsvariable reicht dafür nicht. Bis zur Entscheidung bleiben produktive Laufzeitfreigaben gesperrt.
KC-005B prüft technische Dienstrechte; KC-006/KC-008 integrieren den Benutzerkontext. Diese offene Entscheidung blockiert keine synthetischen Negativtests.

Der Projektleitung wurde vorgeschlagen: eigene Datenbankidentität und eigener Verbindungspool je Dienst und Einrichtung.
Die Rückfrage wurde im Projektchat gestellt. Eine Antwort steht aus. Ohne Antwort gilt keine Freigabe dieser Produktionsarchitektur.

## KC-006: Angefragte Sicherheitsregeln

Ursprüngliche Fragen vom 20.10.2026. Aktualisierter Antwortstand: 21.10.2026.

| Thema | Angefragter Vorschlag | Stand und Sperre |
|---|---|---|
| MFA-Zuordnung | Pflicht für Konten mit Verwaltungsrechten oder Zugriff auf besonders sensible Daten; sonst zunächst optional. Sensible Aktionen benötigen zusätzliche Prüfung. | Ausdrücklich bestätigt. Konkrete Zuordnung folgt KC-008. |
| Browser-Sitzungen | 15 Minuten Inaktivität, 8 Stunden absolute Laufzeit, zusätzliche Prüfung höchstens 5 Minuten gültig; Träger dürfen verkürzen. | Offen. Projektvorschlag, keine gesetzliche Vorgabe oder bestätigte Mindestregel. Keine eingebauten Vorgabewerte. |
| Wiederherstellung | Einmalige offline aufbewahrte Codes; keine MFA-Rücksetzung allein durch E-Mail. Bestehende Sitzungen nach Wiederherstellung widerrufen und nötige Faktoren neu einrichten. Fälle ohne Nachweis bleiben bis zum geprüften Ersatzverfahren gesperrt. | Ausdrücklich bestätigt. KC-006E setzt den begrenzten internen Ablauf um. |

KC-006A / #62 kann unabhängig davon nur Kennungstypen und lokale Zeit-/Zustandsregeln mit expliziten Eingaben implementieren.
Die [KC-006-Planung](../24_KC-006_Authentisierung.md) beschreibt Teilpakete und Abnahmegrenzen.

## Laufzeitgrenzen nach KC-006C

[ADR-031](../architecture/ADR-031_Passwortpruefung.md) enthält keine produktiven Versuchslimits.
Vor KC-006F sind Zeitfenster, Versuche je Prüfschlüssel, Gesamtversuche und maximale aktive Prüfschlüssel festzulegen.
Die Gateway-Anbindung muss außerdem die vertrauenswürdige Bildung dieser Schlüssel festlegen.
Diese Punkte wurden noch nicht als Interviewentscheidung bestätigt. Synthetische Testwerte sind keine Vorgabewerte.
Vor produktiver Nutzung muss das Argon2id-Profil auf der bestätigten Referenzhardware gemessen werden.
Keine Antwort zu diesen Punkten aus der allgemeinen Aufforderung zur Weiterarbeit ableiten.

## KC-006D: Bestätigtes WebAuthn-Testprofil

Stand: 20.10.2026. Die Projektleitung hat „Alle drei Vorschläge bestätigen“ geantwortet.
Bestätigt sind zunächst ausschließlich localhost im isolierten Test, erforderliche Benutzerverifikation und erlaubte synchronisierte Passkeys.
Die bisherigen Sperren für dieses Testprofil sind durch die ausdrückliche Antwort aufgehoben.

| Thema | Bestätigter Stand | Verbleibende Grenze |
|---|---|---|
| Anmeldedomain | Zunächst nur localhost im isolierten Test. | Produktive RP-ID und HTTPS-Ursprung offen. |
| Benutzerverifikation | Bei jedem Passkey-Vorgang erforderlich. | Allgemeine MFA-Regeln anderer Anmeldewege bleiben offen. |
| Synchronisierte Passkeys | Mit Benutzerverifikation erlaubt. | Kompatibilität konkreter Geräte und Passwortmanager bleibt gesondert zu prüfen. |

[ADR-032](../architecture/ADR-032_WebAuthn_Vorbereitung.md) und der [Vertrag](../../contracts/identity-access/webauthn-v1.md) wurden nach der Antwort in ae85a5f vor Code aktualisiert und versioniert.
Die produktive Domain und die übrigen MFA-/Wiederherstellungs-/Rollenentscheidungen bleiben offen.

## KC-006E: Erneut vorgelegte Regeln und Benachrichtigung

Stand: 21.10.2026. MFA-Zuordnung und Wiederherstellung wurden für #66 erneut gemeinsam zur ausdrücklichen Entscheidung vorgelegt.
Die Vorschläge aus der bisherigen Tabelle bleiben unverändert. Es liegt noch keine Antwort vor.
Die WebAuthn-Bestätigung beantwortet diese Fragen nicht.

Zusätzlich ist angefragt: Sicherheitsprotokoll und atomar vorgemerkte Nachricht an einen bereits bestätigten Kontakt
bei Einrichtung, Wechsel oder Widerruf eines Faktors sowie bei Wiederherstellung. Die Nachricht enthält keine Geheimnisse.
Der spätere Nachrichtendienst würde den Versand übernehmen. Auch diese Antwort steht aus.

[ADR-033](../architecture/ADR-033_TOTP_und_Wiederherstellung.md) und der [Vertragsentwurf](../../contracts/identity-access/mfa-recovery-v1.md) dokumentieren die unabhängige Vorbereitung.
Vor abhängiger Implementierung zuerst Antworten in Vertrag und ADR versionieren.
Produktiver Schlüsselanbieter, Nonce-Eindeutigkeit, Wiederanlauf und Rotation benötigen vor produktiver Nutzung einen geprüften Vertrag.
Der separate Bibliotheks-Prüfgraph ersetzt weder diese Betriebsentscheidungen noch die vollständige Projektprüfung.

## KC-006E: Ausdrückliche Bestätigung

Am 21.10.2026 bestätigt die Projektleitung „Ja, alle genannten Regeln bestätigen“.
Damit sind die zuvor vorgelegten MFA-/Wiederherstellungsregeln und Sicherheitsprotokoll plus atomarer Benachrichtigungsauftrag bestätigt.
Diese Regeln nicht erneut abfragen. Die vorherigen Abschnitte dokumentieren den damaligen Fragezustand.
Die genaue Rechtezuordnung folgt KC-008. Browser-Sitzungswerte und produktiver Schlüsselbetrieb bleiben offen.
Vertrag und ADR-033 werden vor Implementierung mit dieser Antwort aktualisiert und versioniert.

## KC-006F: Sitzungsfrage erneut vorgelegt

Am 21.10.2026 wurde ausschließlich die noch offene Sitzungsfrage erneut vorgelegt:
15 Minuten Inaktivität, spätestens 8 Stunden Laufzeit und höchstens 5 Minuten für eine zusätzliche Authentisierung.
Träger dürfen Fristen verkürzen. Eine ausdrückliche Antwort steht aus.
Die MFA-/Wiederherstellungsbestätigung entscheidet diese Werte nicht. KC-006E wird unabhängig davon abgeschlossen.
Produktive Versuchslimits, vertrauenswürdige Prüfschlüsselbindung, Domain, Schlüsselanbieter und Rollenbindung bleiben gesondert offen.

## KC-006F: Zusammengeführtes Interview vor der Browseranbindung

Am 21.10.2026 wurden drei konkrete Vorschläge gemeinsam vorgelegt. Antworten stehen aus.

| Frage | Vorschlag | Sperre |
|---|---|---|
| 1: Sitzungsfristen | 15 Minuten Inaktivität, 8 Stunden absolute Laufzeit, zusätzliche Authentisierung höchstens 5 Minuten; Verkürzung durch Träger erlaubt. | Keine bestätigten Vorgabewerte. Die frühere offene Frage ist darin enthalten. |
| 2: MFA-Kombination | Passwort plus TOTP oder Passkey mit zwingender Geräte-PIN/Biometrie erfüllt die MFA-Pflicht. Alternative: zusätzliches TOTP auch beim Passkey. | Die Gleichstellung ist noch nicht entschieden. Die bestehende Passkey-Benutzerverifikation und erlaubte Synchronisierung bleiben bestätigt. |
| 3: Browser-Schutz | Secure, HttpOnly, SameSite=Strict, keine Domain-Freigabe; serverseitig gebundener CSRF-Nachweis und genaue Origin-Prüfung für zustandsändernde Anfragen einschließlich Anmeldung/Abmeldung. | ADR-015 verlangt Bestätigung vor Implementierung. Externe SSO-Rückleitungen folgen KC-007. |

Unabhängig davon werden [ADR-034](../architecture/ADR-034_Browseranmeldung_und_Nachweisuebergabe.md)
und der [Vertragsentwurf](../../contracts/identity-access/browser-auth-v1.md) vorbereitet.
Eine allgemeine Aufforderung zur Weiterarbeit beantwortet die drei Fragen nicht.
Die konkrete Rechtezuordnung, produktive Domain, Versuchslimits und Rollen-/Schlüsselbindung bleiben gesondert.

## KC-006F: Bestätigte Interviewantworten

Am 21.10.2026 antwortet die Projektleitung auf Frage 1 „Einstellbar pro Einrichtung“, auf Frage 2 „Beide Varianten bestätigen“ und auf Frage 3 „Vorschlag bestätigen“.
Die anschließende Frage zu Voreinstellungen und Obergrenzen beantwortet sie ausdrücklich mit „Diese Voreinstellungen und Obergrenzen bestätigen“.

- Pro Einrichtung einstellbare Fristen: höchstens 900 Sekunden Inaktivität, 28.800 Sekunden absolut und 300 Sekunden zusätzliche Authentisierung. Diese Werte sind auch die Voreinstellungen; Einrichtungen dürfen kürzere Werte wählen.
- Die MFA-Pflicht kann durch Passwort plus TOTP oder durch einen Passkey mit zwingender Geräte-PIN/Biometrie erfüllt werden.
- Secure, HttpOnly, SameSite=Strict, keine Domain-Freigabe, serverseitig gebundener CSRF-Nachweis und genaue Origin-Prüfung für zustandsändernde Browseranfragen einschließlich Anmeldung/Abmeldung sind bestätigt. Externe SSO-Regeln folgen KC-007.

Diese vier Antworten ersetzen die zuvor dokumentierten offenen Fragezustände. Nicht erneut fragen.
Vertrag, ADR-034 und [ADR-035](../architecture/ADR-035_Sitzungsfristen_je_Einrichtung.md) werden vor Code versioniert.
Produktive Domain, Versuchslimits, konkrete Verwaltungsrechte und Rollen-/Schlüsselbindung bleiben gesondert.

## KC-006F: Zusätzliches HTTPS-Testprofil angefragt

Am 21.10.2026 wurde für kommende isolierte Gateway-/Browserprüfungen zusätzlich https://localhost:3443 vorgeschlagen.
Das bestehende HTTP-WebAuthn-Testprofil soll getrennt erhalten bleiben. Testzertifikate würden ausschließlich zur Laufzeit erzeugt und nicht versioniert.
Die produktive Domain bleibt ausdrücklich offen. Eine Antwort steht aus; das zusätzliche Profil darf vorher nicht implementiert werden.
Interne Sitzungs-, Rotations- und Vertragsarbeiten bleiben davon unabhängig.

## KC-008 / KC-006F: Konkrete MFA-Zuordnung angefragt

Am 21.10.2026 wurde folgende Zuordnung zur Entscheidung vorgelegt:
Aktuelle zusätzliche MFA für Rollen-/Rechteverwaltung, sicherheitsrelevante Einstellungen,
Gesundheitsdaten, Sorgerechtsbeschränkungen, Personalakten und interne Entwicklungsakten.
Normale Anmeldung erteilt dafür keinen Zugriff. Unbekannte Einstufungen verweigern Zugriff.
Veröffentlichte Portfolios und einfache Kontaktdaten behalten ihre gesonderten Objekt-/Feldrechte.
Alternativen waren MFA für alle personenbezogenen Fachzugriffe oder ausdrückliches Offenlassen.
Die Antwort steht aus. Keine dieser Varianten wird ohne Antwort implementiert.
Die interne Mitgliedschaftsabfrage und bereits bestätigte Authentisierungsprotokolle bleiben davon unabhängig.

## KC-008 / KC-006F: Konkrete MFA-Zuordnung bestätigt

Am 21.10.2026 bestätigt die Projektleitung ausdrücklich: „Diese Zuordnung bestätigen“.
Die oben vorgelegte Zuordnung ist damit verbindlich. Nicht erneut abfragen.
Rollen-/Rechteverwaltung, sicherheitsrelevante Einstellungen und Zugriff auf Gesundheitsdaten,
Sorgerechtsbeschränkungen, Personalakten sowie interne Entwicklungsakten verlangen aktuelle zusätzliche MFA.
Normale Anmeldung allein gewährt dafür keinen Zugriff. Unbekannte Einstufungen werden abgewiesen.
Veröffentlichte Portfolios und einfache Kontaktdaten behalten ihre gesonderten Objekt-/Feldrechte.
Die Grundpflicht für Verwaltungsrechte und besonders sensible Daten bleibt bestehen.
Die Umsetzung wird vor Code in einem eigenen Rechte-/MFA-Vertrag konkretisiert.
Das zusätzliche HTTPS-Testprofil bleibt weiterhin ausdrücklich offen.

## OAuth-Fehlerformate: begrenzte Ausnahme bestätigt

Am 22.10.2026 bestätigt die Projektleitung ausdrücklich die vorgeschlagene OAuth-Ausnahme.
Nur OAuth-Protokollendpunkte verwenden ihre standardisierten protokollspezifischen Fehlerformate.
Fach-APIs behalten Problem-Details nach RFC 9457. Die Klarstellung wird in Dokument 04 und im Entscheidungsregister vor Code dokumentiert.
Keine neue Fachberechtigung und keine Freigabe des offenen HTTPS-Browserprofils ableiten. Nicht erneut fragen.

## KC-006F: Zertifikatsname für OAuth-Clientauthentisierung angefragt

Am 22.10.2026 ist vor dem HTTP-Registrierungsadapter folgende Entscheidung angefragt:
Genau ein ausdrücklich registrierter DNS-Name aus dem Zertifikat je Dienst, zusätzlich zum aktiven genauen Zertifikatsfingerabdruck.
Keine Wildcards, automatische Registrierung oder produktiven Namensvorgaben.
Grundlage ist [RFC 8705, Abschnitt 2.1](https://www.rfc-editor.org/rfc/rfc8705#section-2.1).
Der vorhandene interne Fingerabdruckbestand allein ist noch kein vollständiger Nachweis des dort beschriebenen Verfahrens.
Bis zur ausdrücklichen Antwort bleiben Namensadapter und abhängige Dienstanmeldung gesperrt.
Formularprüfung, OpenAPI- und Darstellungsmodell werden unabhängig weiterbearbeitet.

## KC-006F: Zertifikatsnamenszuordnung bestätigt

Am 22.10.2026 bestätigt die Projektleitung ausdrücklich „DNS-Name plus aktiven Fingerabdruck bestätigen“.
Je Dienst gilt genau ein ausdrücklich registrierter DNS-Name aus dem Zertifikat.
Zusätzlich muss der genaue Zertifikatsfingerabdruck aktiv registriert sein.
Keine Wildcards oder automatische Registrierung; produktive Namen bleiben offen.
Diese Antwort ersetzt den vorherigen offenen Fragezustand. Nicht erneut fragen.
ADR-046 und Dienstvertrag werden vor dem abhängigen Code aktualisiert.
