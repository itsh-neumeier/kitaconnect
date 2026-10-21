# Sitzungsbestand v1 – interner Speichervertrag

Stand: 20.10.2026. Teilpaket: KC-006B / #63. Grundlage: ADR-014, ADR-019 und Modellvertrag.

## Datenhoheit und Rechte

`identity-access` besitzt den gesonderten Plattformbereich `identity_access`.
Er enthält `identities`, `sessions` und `schema_versions`. Es gibt keine dienstübergreifenden Fremdschlüssel oder Fachakten.
`identities` speichert zunächst ausschließlich interne Kennungen. Eine Kennung allein belegt keine geprüfte Anmeldung.
Die Rollen `kc_identity_owner`, `kc_identity_migrator` und `kc_identity_runtime` bleiben ohne Login und besondere Rollenattribute.
Nur die Migrationsrolle darf die Eigentümerrolle ausdrücklich annehmen; sie erbt deren Rechte nicht automatisch.
Die Laufzeit erhält kein Eigentum, CREATE, TRUNCATE, DELETE oder Migrationsrecht.
ENABLE und FORCE RLS gelten für Identitäten und Sitzungen. Es gibt zunächst keine erlaubende Policy.
Eine produktive Laufzeitfreigabe ist kein Teil dieses Vertrags.

Migration 0001 legt diesen Bestand atomar an. Sie läuft gesondert mit einer Betriebsverbindung und niemals beim Anwendungsstart.
Fehler müssen Objekte, Versionsstand und neue Rechte der Migration vollständig zurücknehmen.
Vorhandene Strukturen werden nicht stillschweigend übernommen.

## Interne Kennungen, Zeit und Sitzungszustand

Kennungen verwenden ganzzahlige numerische Werte von 1 bis u128::MAX.
Zeitpunkte verwenden ganze Sekunden seit der Unix-Epoche von 0 bis u64::MAX; Grenzen sind positive Sekunden.
Unbegrenzte PostgreSQL-Numerik mit ausdrücklichen Prüfbedingungen verhindert Rundung ungültiger Bruchwerte.
Die Datenbank prüft Reihenfolge von Erstellung, Aktivität, Beobachtung und absolutem Ende.
Zustände sind `active`, `expired` und `revoked`. Sitzungsfristen erhalten keine Vorgabewerte.

Der Adapter übernimmt die serverseitige Datenbankzeit durch `clock_timestamp()` nach Erwerb der betreffenden Zeilensperre.
Er verwendet keine Clientzeit und keine vor einer Sperrwartezeit eingefrorene Transaktionszeit.
Die Umwandlung in das Rust-Modell prüft alle numerischen Grenzen und Zustandsinvarianten.
Ein ungültiger gespeicherter Datensatz oder rückläufige Zeit führt zu einem Fehler, niemals zu einem positiven Treffer.

## Geheimer Nachweis

Ein Sitzungsnachweis besteht intern aus 32 zufälligen Bytes aus der Betriebssystemquelle über `getrandom` 0.4.3.
Fehler der Zufallsquelle brechen die Erzeugung ab. Es gibt keinen Ersatzgenerator.
Die Datenbank speichert ausschließlich SHA-256 aus `sha2` 0.11.1, mit eindeutiger 32-Byte-Spalte.
Diese Ableitung gilt für zufällige Sitzungsnachweise, nicht für Passwörter. Argon2id bleibt für Passwörter verbindlich.
Die rohen Nachweise werden nur im Speicher erzeugt und nicht in SQL, Dateien, Logs oder Fehlermeldungen übernommen.
Die Nachweishülle hat keine automatische Kopier-/Serialisierungsfunktion und zeigt in Debug nur einen Platzhalter.
`zeroize` 1.9.1 überschreibt den verwalteten Bytepuffer beim Verwerfen; dies garantiert nicht die Entfernung beliebiger fremder Kopien.
Die ausdrückliche Byteausgabe dient erst der späteren Transportanbindung. Cookieformat und HTTP-Verwendung bleiben KC-006F.

## Operationen und Konkurrenz

Die interne Rust-API erhält eine bestehende Verbindung. Sie ändert keine Rolle und öffnet keine Datenbankverbindung.
Ihre Aufrufer müssen einen freigegebenen internen Zweck und die passende Dienstidentität prüfen.
Sie ist keine öffentliche Erstellungs- oder Widerrufsschnittstelle.
Die Verbindung muss außerhalb einer aufrufenden Transaktion stehen. Der Adapter weist aktive SQLx-Transaktionen ab.
Manuell per Roh-SQL eröffnete Transaktionen sind ebenfalls kein zulässiger Aufrufkontext.
Eigene mehrteilige Vorgänge verwenden ausdrücklich READ COMMITTED und bestätigen vor erfolgreicher Rückgabe.
Eine nicht bestätigte Zustandsaktualisierung darf keinen positiven Treffer erzeugen.
Ein Datenbanktrigger verhindert zusätzlich Reaktivierung, rückläufige Zeit und Änderungen unveränderlicher Sitzungsfelder.

- Identitätsanlage speichert nur eine interne Kennung; doppelte Kennungen scheitern.
  Erfolg verlangt genau eine betroffene Zeile. Ein Trigger, der die Anlage unterdrückt, muss zu einem Fehler führen.
- Sitzungsanlage erhält interne Kennungen und ausdrücklich gewählte Grenzen.
  Sie erzeugt den Nachweis, bestimmt die Datenbankzeit und speichert nur dessen Ableitung.
  Erfolg bestätigt die gesamte Anlage; ein Fehler gibt keinen nutzbaren Nachweis zurück.
  Auch hier muss genau eine betroffene Zeile bestätigt sein, bevor die Transaktion bestätigt wird.
- Aktuelle Prüfung sucht über die Ableitung und sperrt den Sitzungsdatensatz mit `FOR UPDATE`.
  Fehlende, abgelaufene oder widerrufene Sitzungen liefern keinen positiven Treffer.
  Ein beobachteter Ablauf wird im selben Vorgang gespeichert.
- Aktivitätsaktualisierung prüft denselben aktuellen Bestand unter derselben Sperre.
  Sie darf nur nach aktuell geprüfter berechtigter Aktivität aufgerufen werden und überschreitet nie die absolute Frist.
- Widerruf setzt den Zustand auf `revoked`; er ist idempotent und konkurriert über dieselbe Zeilensperre.
  Weder Prüfung noch Aktivität setzen den Zustand auf `active` zurück.

Jede Operation muss Fehler zurückgeben oder die vollständige eigene Transaktion bestätigen.
Ein positiver Rückgabewert ist eine Momentaufnahme, keine dauerhafte Zugriffsberechtigung.
Die nächste geschützte Prüfung muss den aktuellen Bestand erneut lesen. Es gibt keinen positiven Ergebniszwischenspeicher.
Datenbankfehler oder fehlende Rechte führen zu Fehler oder fehlendem Treffer, niemals zu einem Ersatzwert mit Freigabe.

## Nachweise und Grenzen

Echte SQLx-/PostgreSQL-Tests prüfen Zufallsnachweiserzeugung, Ableitung statt Klartext, Fristen, Widerruf,
konkurrierende Aktualisierung, gesperrte Laufzeitrolle, Fehler und Erhalt nach Server-Neustart.
Die positive Speicherprüfung verwendet ausschließlich die Test-Betriebsidentität im isolierten Container.
Die gesonderte Test-Laufzeitidentität prüft die unveränderte RLS-Sperre.
Testnachweise werden nur im Arbeitsspeicher erzeugt; kein roher Nachweis wird zur Neustartprüfung in einer Datei gespeichert.

Dies implementiert keine Anmeldung, Mitgliedschaft, Benutzerberechtigung, Cookies, MFA oder Wiederherstellung.
Die offenen Sicherheitsentscheidungen bleiben gesperrt. KC-006C/D/E und KC-006F ergänzen ihre jeweiligen Nachweise.
