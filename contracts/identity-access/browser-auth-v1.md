# Vertragsentwurf: Browseranmeldung und aktuelle Sitzungsprüfung v1

Stand: 21.10.2026. KC-006F / #67. Grundlage: Spezifikation v5.9 und ADR-015/019/029 bis ADR-034.
Status: Vorbereitung. Keine HTTP-Anmeldung, Migration oder neue Laufzeitfunktion implementiert.
Die drei angefragten Sicherheitsentscheidungen sind noch nicht bestätigt.

## Zuständigkeit und bestehender Stand

identity-access verwaltet Anmeldedaten, Nachweisverbrauch und Sitzungen.
Das eigenständige Rust-Gateway erhält keine Datenbankrolle und keine TOTP-Geheimnisse.
Es verwendet geprüfte Dienstverträge und leitet ausschließlich benötigte Daten weiter.
Ein Client-Header, eine Identitätskennung oder eine Rollenliste erteilt keine Berechtigung.
SvelteKit stellt Seiten dar; der Zieldienst prüft seine Rechte selbst.

Die bestehenden Passwort-, Passkey- und TOTP-Adapter liefern begrenzte positive Ergebnisse.
Diese Ergebnisse enthalten derzeit nur die Identitätszuordnung. Sie sind keine einmaligen Abschlussnachweise.
Ein Aufruf von create_session nach einem solchen Ergebnis erfüllt den Anmeldevertrag noch nicht.
Der Anmeldedatenbestand oder die Mitgliedschaft kann sich zwischen Prüfung und Sitzungserzeugung ändern.
Eine spätere Prüfung muss diesen Zwischenraum schließen.

## Entscheidungsmatrix

| Thema | Vorschlag | Stand und Sperre |
|---|---|---|
| Sitzungsfristen | 15 Minuten Inaktivität, 8 Stunden absolute Laufzeit, höchstens 5 Minuten zusätzliche Authentisierung; Träger dürfen verkürzen. | Ausdrücklich angefragt, noch offen. Keine Vorgabewerte implementieren. |
| Erfüllung der MFA-Pflicht | Passwort plus TOTP oder Passkey mit zwingender Benutzerverifikation. | Ausdrücklich angefragt, noch offen. Die bestätigte Passkey-Benutzerverifikation entscheidet diese Gleichstellung nicht. |
| Cookie und CSRF | Secure, HttpOnly, SameSite=Strict, Path=/, keine Domain-Freigabe; serverseitig gebundener CSRF-Nachweis und genaue Origin-Prüfung für zustandsändernde Anfragen einschließlich Anmeldung/Abmeldung. | Gemäß ADR-015 ausdrücklich angefragt, noch offen. Keine abhängige Gateway-Implementierung. |

Die bestätigte MFA-Pflicht bei Verwaltungsrechten und besonders sensiblen Daten bleibt bestehen.
Die genaue Rechtezuordnung folgt KC-008. Unbekannte notwendige Policy-Daten führen zur Verweigerung.
Wiederherstellung gibt ausschließlich den bestehenden begrenzten Neueinrichtungsnachweis aus.
Sie ist kein vierter Weg zu einer unbeschränkten Sitzung. TOTP allein ersetzt keine vollständige Identitätsprüfung.

## Geplante technische Übergabe

Die folgende technische Richtung schränkt die Umsetzung ein. Sie gibt keine offene Sicherheitsregel frei.

1. Erzeugen Sie einen serverseitigen, zweckgebundenen Anmeldevorgang mit expliziter Frist und Mengenbegrenzung.
2. Reservieren Sie Prüfversuche dauerhaft vor der Nachweisprüfung. Fehlgeschlagene Fachtransaktionen geben diese Versuche nicht zurück.
3. Prüfen Sie Passwort, Passkey beziehungsweise den zusätzlich erforderlichen TOTP-Faktor mit den bestehenden Bibliotheken.
4. Merken Sie den erfolgreichen Nachweis im selben bestätigten Vorgang wie dessen abschließende Bestandsprüfung oder Einmalverbrauch vor.
5. Binden Sie den Nachweis an Identität, Zweck, Anmeldevorgang, geprüfte Bestandsversion und Datenbankzeit.
6. Prüfen Sie beim Sitzungsabschluss den aktuellen Identitätsbestand, die Wiederherstellungssperre und die erforderlichen Faktoren erneut. Prüfen Sie die Mitgliedschaft beim zuständigen Organisationsdienst.
7. Verbrauchen Sie den Abschlussnachweis genau einmal. Bestätigen Sie Verbrauch, neue Sitzung und erforderlichen Widerruf der bisherigen Sitzung gemeinsam.
8. Geben Sie einen neuen Sitzungsnachweis erst nach erfolgreichem Commit aus. Übernehmen Sie keinen vom Client vorgegebenen Sitzungsnachweis.

Eine zufällige interne Vorgangsreferenz enthält mindestens 32 zufällige Bytes aus der Betriebssystemquelle.
Der Bestand speichert nur deren Ableitung. Rohe Referenzen gelangen nicht in URL, Protokoll oder Ereignisnutzlast.
Die Referenz ist ein interner Anmeldevorgang und kein neu erfundenes Diensttokenverfahren.
Dienstauthentisierung bleibt an die Standards aus ADR-019 gebunden.

Die bloße Speicherung eines positiven Ergebnisses nach Rückkehr aus dem bisherigen Adapter reicht nicht.
Auch das nachträgliche Lesen der dann aktuellen Credential-Version schließt den Zwischenraum nicht.
Die vorgemerkte Version muss nachweisbar zum tatsächlich geprüften Bestand gehören.
Eine Passwort-Hashaktualisierung muss dabei die bestätigte neue Version berücksichtigen.
Ein Passkey-Abschluss muss die Version nach seiner erforderlichen Credential-Aktualisierung binden.
Die TOTP-Übergabe muss zu demselben Vorgang gehören wie der dauerhafte Schrittverbrauch.
Aus einem alten positiven Rust-Objekt darf kein neuer Abschlussnachweis erzeugt werden.

## Geplante Serialisierung und Persistenz

Der Identitätsdienst benötigt eine gemeinsame Sperre je Identität für schutzrelevante Änderungen und Sitzungsabschlüsse.
Die geplante lokale Reihenfolge lautet: gemeinsame Identitätssperre, MFA-Bestand, Passwortbestand, Passkey-Bestand,
Anmeldevorgang und Sitzungsbestand. Ereignisse folgen nach den Bestandsänderungen.
Alle beteiligten Schreibpfade müssen dieselbe Reihenfolge einhalten.
Teure Passwort- und Signaturarbeit soll außerhalb dieser Sperre bleiben; deren abschließende Bestandsprüfung benötigt die Sperre.
Eine reine Gateway-Sperre oder ein lokaler Prozessmutex reicht bei mehreren Dienstinstanzen nicht.

Die neue zusammengesetzte Operation benötigt einen eigenen Transaktionsvertrag.
Die bisherigen Adapter mit eigenen Transaktionen dürfen nicht ungeprüft in eine äußere Transaktion verschachtelt werden.
Neue Tabellen und nötige Erweiterungen erhalten neue Migrationen. Migrationen 0001 bis 0004 bleiben unverändert.
Erzwungene RLS bleibt ohne erlaubende produktive Policy bestehen.
Eine konkrete Migration und die Adapteränderungen folgen erst nach versioniertem Implementierungsvertrag.

## Mitgliedschaft und aktuelle Prüfung

organization-config ist laut Dienstmatrix für Mitgliedschaften zuständig. identity-access bleibt für Identitäten und Sitzungen zuständig.
Der aktuelle Organisationsbestand muss Identität und Einrichtung eindeutig verbinden und Widerruf erkennbar machen.
Eine vom Client ausgewählte Einrichtung ist nur ein gewünschtes Ziel und kein Berechtigungsnachweis.
Die Mitgliedschaftsprüfung benötigt einen geprüften Dienstvertrag von organization-config; direkte dienstübergreifende Tabellenzugriffe und Fremdschlüssel bleiben verboten.
KC-006F erfindet keine neue fachliche Regel zum Anlegen oder Übertragen von Mitgliedschaften.
Positive Tests dürfen ausschließlich ausdrücklich gekennzeichnete synthetische Zuordnungen über ihren Test-Betriebszugang anlegen.

Jede geschützte Prüfung liest Sitzung und Wiederherstellungssperre aktuell und fragt die Mitgliedschaft beim zuständigen Organisationsdienst ab.
Sie berücksichtigt Zweck und erforderliche Authentisierungsstärke. Fehlende notwendige Daten und Ausfälle verweigern Zugriff.
Ein Mitgliedschaftsentzug muss bei der nächsten geschützten Prüfung wirken; ein positiver Ergebniszwischenspeicher ist ausgeschlossen.
Die Prüfung liefert höchstens einen begrenzten Identitäts-/Einrichtungskontext. Objekt- und Feldrechte folgen KC-008.
Die spätere Fachaktion braucht weiterhin eine aktuelle Prüfung durch ihren zuständigen Dienst.

Eine lokale Identitätstransaktion kann die fremde Mitgliedschaft nicht atomar einschließen.
Ein früherer Mitgliedschaftstreffer darf deshalb weder als dauerhaft gültiger Sitzungsinhalt noch als Ersatz für die nächste geschützte Prüfung dienen.
Der konkrete Implementierungsvertrag muss die Prüfzeitpunkte und konkurrierenden Widerruf zwischen beiden Diensten ausdrücklich festlegen.
Eine ausgegebene Sitzung gewährt allein noch keinen Einrichtungszugriff. Der aktuelle Mitgliedschaftsaufruf bleibt bei geschützten Aktionen erforderlich.
KC-004C bleibt ein eigenes abhängiges Paket; es darf nicht als vollständige Voraussetzung von KC-006F eingefügt werden und dadurch einen Backlog-Zyklus erzeugen.
Der minimale Prüfvertrag und seine synthetischen Zuordnungen sind von der späteren fachlichen Organisationsanbindung zu trennen.

## Geplante Gateway- und Browsergrenze

Der vorgeschlagene Cookie-Name ist __Host-kc_session. Die Domain-Eigenschaft bleibt ungesetzt.
Der CSRF-Nachweis ist an den aktuellen serverseitigen Vorgang beziehungsweise die Sitzung gebunden.
Er wird über einen eigenen Anfrageheader übertragen und bei Sitzungswechsel entwertet.
Anmeldung benötigt bereits Schutz vor Login-CSRF; sie darf nicht auf eine erst danach existierende Sitzung warten.
Zustandsänderungen über GET und ein Ersatz für fehlende Prüfungen durch SameSite allein sind ausgeschlossen.
Fehlende, mehrdeutige oder fremde Ursprünge müssen nach dem bestätigten Browservertrag abgewiesen werden.
Ungeprüfte Forwarded-Header dürfen den erlaubten Ursprung nicht bestimmen.

Abmeldung widerruft den Serverbestand und entfernt das Cookie mit passenden Attributen.
Sitzungsrotation entwertet den alten Nachweis atomar; parallele Wiederverwendung darf keinen zweiten Erfolg erzeugen.
Wiederherstellungsnachweise dürfen weder als Sitzungscookie noch als fachlicher Zugriffskontext angenommen werden.
Antworten mit Geheimnissen benötigen Cache-Control: no-store und eine begrenzte Fehlerdarstellung.

Das bestehende WebAuthn-Profil gilt ausschließlich für localhost und http://localhost:3000.
Es ist kein Nachweis für Cookie-Verhalten im Browser unter HTTPS oder für eine produktive Domain.
Ein sicherer Browser-/TLS-Testzugang muss vor entsprechenden Tests konkretisiert werden.
Der Vertrag erlaubt keine Abschwächung von Secure als Ersatz für diesen Testaufbau.
Externe Föderations- und SSO-Rückleitungen bleiben KC-007 vorbehalten.

## Testmatrix und Nachweisgrenzen

| Bereich | Erforderlicher Nachweis |
|---|---|
| Nachweisübergabe | Richtige Identität und Zweck, richtige Bestandsversion, Fristen, rückläufige Zeit, keine Umwandlung alter positiver Ergebnisse. |
| Konkurrenz | Zwei tatsächlich sperrwartende Abschlüsse ergeben höchstens eine Sitzung. Credential-Wechsel und MFA-Wiederherstellung konkurrieren über dieselbe lokale Schutzgrenze. Mitgliedschaftsentzug wird beim zuständigen Dienst und bei der nächsten geschützten Prüfung separat nachgewiesen. |
| Vier Wege | Passwort, Passkey und TOTP mit bestätigter Kombination positiv/negativ; Wiederherstellung nur eingeschränkt bis zur Neueinrichtung. |
| Cookie und CSRF | Bestätigte Attribute, falscher/fehlender Ursprung und CSRF-Nachweis, Login-CSRF, Fixierung, Rotation, Abmeldung und Wiederverwendung abgewiesen. |
| Rechte | Fremde Einrichtung, fremde Identität, widerrufene Mitgliedschaft und fehlende Policy-Daten abgewiesen; keine pauschalen Fachrechte. |
| Fehler und Speicherung | Abbruch, unterdrücktes Update, Commitfehler und verlorene Verbindung erzeugen keine Freigabe; Verbrauch und Widerruf vor/nach Neustart nachgewiesen. |
| Dienstgrenze | ADR-019: falscher Dienst, Aussteller, Empfänger, Ablauf, mTLS-Bindung, Widerruf und Ausfall; Testdoppel gelten nicht als Transportnachweis. |

Prozessinterne Routertests ersetzen keine Browser-, TLS- oder verteilten Diensttests.
Ein synthetisches Mitgliedschafts-Testdoppel belegt keine produktive Organisationsbindung.
Ein Server-Cookieheader belegt allein kein Cookie-Verhalten eines Browsers.
Alle ursprünglichen Kriterien von KC-006F und KC-006 bleiben offen, bis die jeweiligen Nachweise vorliegen.

## Quellen und Einordnung

- [OWASP: Sitzungsverwaltung](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html)
- [OWASP: CSRF-Schutz](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html)
- [NIST SP 800-63B-4: Synchronisierbare Authentikatoren](https://pages.nist.gov/800-63-4/sp800-63b/syncable/)

Die Quellen begründen technische Vorschläge. Sie bestätigen weder die offenen Projektentscheidungen noch eine NIST-Konformität von KitaConnect.
