# Dienstvertrag: Kurzlebige Diensttokens v1

Stand: 21.10.2026. KC-006F / #67. Grundlage: bestätigte ADR-019 und Dienstmatrix.

## Umfang und Quellen

identity-access stellt nicht selbstbeschreibende, installationsgebundene Diensttokens aus.
Der erste überprüfbare Schritt umfasst nur internen Bestand und aktuelle Zustandsprüfung.
Ein ausdrücklich gekennzeichneter vertrauenswürdiger Test-Aufrufer ersetzt in diesem Teil keinen mTLS-Nachweis.
HTTP-Endpunkte, echte Zertifikatsprüfung und empfangender Dienst folgen getrennt.
Es entsteht keine produktive Laufzeitfreigabe und keine neue MFA-/Fachrechtezuordnung.

Die technische Richtung verwendet OAuth 2.0 Client Credentials nach [RFC 6749, Abschnitt 4.4](https://www.rfc-editor.org/rfc/rfc6749#section-4.4).
Die aktuelle Zustandsabfrage folgt [RFC 7662](https://www.rfc-editor.org/rfc/rfc7662).
Die Zertifikatsbindung folgt [RFC 8705, Abschnitt 3.2](https://www.rfc-editor.org/rfc/rfc8705#section-3.2).
Die weitere Transportauswahl berücksichtigt [RFC 9700](https://www.rfc-editor.org/rfc/rfc9700).
Diese Zuordnung ist eine Implementierungsrichtung und noch kein vollständiger Protokoll- oder Konformitätsnachweis.

## Daten und Vertrauensgrenze

Die Installation und ihr Aussteller sind ausdrücklich konfigurierte Werte ohne Projektvorgabe.
Dienstnamen und aufrufender Dienst stammen aus einer geprüften Registrierung und später aus der tatsächlich geprüften TLS-Verbindung.
Ein Client-Header, eine vom Client behauptete Zertifikatsableitung oder eine Benutzerkennung darf diese Herkunft nicht ersetzen.
Ein Zertifikatsnachweis enthält die SHA-256-Ableitung des DER-Zertifikats; kein privater Schlüssel wird gespeichert.
Produktive Domain, Zertifikatsaussteller, Betriebsidentität und Schlüsselbetrieb bleiben gesondert offen.

Eine gesonderte Registrierung verbindet Dienst, Zertifikatsableitung, Gültigkeit und aktiven Zustand.
Eine ausdrücklich begrenzte Freigabe verbindet Aufrufer, genau einen Zieldienst und einen beschriebenen Zweck.
Fehlende oder unbekannte Zwecke verweigern die Ausgabe. Es gibt keinen pauschalen Zugriff auf alle Dienste.
Registrierungen und Freigaben werden im ersten Teil ausschließlich als synthetische Test-Betriebsdaten angelegt.
Die Laufzeit erhält keine Registrierungs- oder Freigabeverwaltung.

## Ausgabe und Lebenszyklus

Der interne Aufrufer muss Installation, Aussteller, aktive Dienstregistrierung und aktuelle Freigabe prüfen.
Ein Token gilt für genau den registrierten Aufrufer, das genaue Zertifikat, den Zieldienst und den freigegebenen Zweck.
Seine ausdrücklich übergebene Laufzeit beträgt höchstens 300 Sekunden und endet spätestens mit der Zertifikatsgültigkeit.
Die bestätigte Zertifikatsobergrenze von 24 Stunden bleibt gemäß ADR-019 bestehen.
Es gibt keine neuen produktiven Versuchslimits, Standardlaufzeiten oder Erneuerungsfenster.

Der Token enthält 32 zufällige Bytes aus der vorhandenen Betriebssystemquelle.
Ein eigener Typ trennt ihn von Browser-Sitzungen und Wiederherstellungsnachweisen.
Der Bestand speichert nur eine Ableitung. Rohe Tokens gelangen nicht in URLs, Protokolle, Ereignisse oder Git.
Ein roher Token wird ausschließlich nach bestätigter Ausgabe zurückgegeben.
Ein fehlgeschlagener Zufallsaufruf oder Commit erzeugt keinen positiven Nachweis.
Eigene Transaktionen folgen dem ausdrücklich abgewarteten Abschluss aus ADR-040.

Widerruf und einmal beobachteter Ablauf sind endgültig.
Ein Zeitrücksprung darf einen so entwerteten Token nicht reaktivieren.
Der Token darf die Gültigkeit von Registrierung, Zertifikat oder Freigabe nicht verlängern.
Eine nachträglich geänderte sicherheitsrelevante Freigabe entwertet frühere Tokens konservativ.

## Aktuelle Zustandsprüfung und Transportfolge

Die interne aktuelle Prüfung liest Installation, Aussteller, Registrierung, Zertifikat, Freigabe und Tokenzustand erneut.
Sie berücksichtigt genaue Zielbindung, Zweck, monotone Zeit, Widerruf und Ablauf.
Fehlende notwendige Daten und Fehler verweigern Zugriff. Positive Ergebnisse werden nicht zwischengespeichert.
Nach einem bestätigten Entzug muss die nächste geschützte Prüfung verweigern, auch auf derselben offenen Verbindung.

Im späteren HTTP-Vertrag authentisiert sich der Zieldienst am Introspektionsendpunkt ebenfalls über mTLS.
Diese Prüfung benötigt kein weiteres Diensttoken und löst keine rekursive Introspektion aus.
Ein berechtigt abgefragter ungültiger oder fremder Token liefert ausschließlich `active: false`.
Eine aktive Antwort trägt nur erforderliche Metadaten einschließlich Aussteller, Empfänger, Zweck, Zeiten und Zertifikatsbindung.
Die Bindung verwendet `cnf` mit `x5t#S256` in der vorgesehenen Base64url-Darstellung ohne Füllzeichen.
Der empfangende Dienst vergleicht sie mit dem Zertifikat seiner tatsächlich geprüften Aufruferverbindung.
Er prüft außerdem Empfänger, Aussteller, Ablauf und Zweck. Fehler oder Abweichungen verweigern die Aktion.
Jede geschützte Aktion benötigt eine aktuelle Abfrage; Tokenlaufzeit ersetzt diesen Entzugsnachweis nicht.

Ein gültiger Diensttoken ist keine Benutzersitzung, Mitgliedschaft oder Fachberechtigung.
Der zuständige Dienst prüft den Benutzer- und Datenumfang zusätzlich aktuell.
Anonyme Wahlen bleiben gemäß ADR-017 vollständig ausgenommen.

## Speicher- und Testgrenzen

Eine neue Identitätsmigration ergänzt nur dienstinterne Registrierung, Freigabe und Tokenbestand.
Alte Migrationen bleiben unverändert. Neue Tabellen erhalten FORCE RLS ohne erlaubende Laufzeitpolicy.
Tests verwenden ausschließlich ausdrücklich synthetische Registrierungen und frische RAM-Tokens.
Sie prüfen richtige/falsche Installation, Aussteller, Aufrufer, Zertifikat, Ziel und Zweck,
Widerruf, Zeitgrenzen, Rücknahme, Commitfehler, tatsächliche Konkurrenz und Neustart.
Tests ohne echte TLS-Verbindung werden nicht als mTLS-Nachweis bezeichnet.
Private Schlüssel und Testzertifikate dürfen nur zur Laufzeit außerhalb des versionierten Bestands entstehen.
