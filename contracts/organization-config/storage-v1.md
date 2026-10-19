# Organisationsspeicherung v1 – interner Vertrag

Stand: 19.10.2026. Issue: [KC-004B / #51](https://github.com/itsh-neumeier/kitaconnect/issues/51).
Grundlage: [Modellvertrag](model-v1.md), ADR-014 und ADR-023.

## Daten und Zuständigkeit

Der Dienst besitzt das restriktive Plattform-Schema `organization_config`.
`carriers` enthält Trägerkennungen. `facilities` enthält Einrichtungskennungen und einen optionalen Trägerverweis.
Eine gemeinsame Organisationshierarchie, personenbezogene Daten und fremde Diensttabellen werden nicht eingeführt.
Die ID-Domäne speichert ganze Werte von 1 bis 340282366920938463463374607431768211455.
Das entspricht den positiven 128-Bit-Werten des Rust-Modells. Bruchwerte werden abgewiesen und nicht gerundet.
Ein Primärschlüssel verhindert doppelte Kennungen. Der Fremdschlüssel ist sofort wirksam.
Ein Nullverweis bezeichnet eine unabhängige Einrichtung.

## Rollen und Zugriff

| Rolle | Aufgabe | Begrenzung |
|---|---|---|
| `kc_org_owner` | Schema und Dienstobjekte besitzen | Kein Login; keine Laufzeitidentität |
| `kc_org_migrator` | Geprüfte Migration mit ausdrücklichem Rollenwechsel ausführen | Kein Login; keine Vererbung oder Vergabe der Eigentümerrolle |
| `kc_org_runtime` | Spätere geprüfte Dienstzugriffe | Kein Login; keine Eigentümer-/Migrationsmitgliedschaft, kein CREATE, BYPASSRLS oder Superuser |

Alle drei Rollen sind Rechtebündel. Loginrollen, Passwörter und Zertifikate entstehen hier nicht.
Die Provisionierung benötigt eine gesonderte privilegierte Betriebsidentität.
Die Eigentümerrolle wird ausschließlich der Migrationsrolle mit `SET TRUE`, `INHERIT FALSE`, `ADMIN FALSE` zugeordnet.
Die Migration vergibt der Laufzeitrolle nur Schema-USAGE und SELECT/INSERT auf den beiden Datentabellen.
UPDATE, DELETE, TRUNCATE, REFERENCES, TRIGGER und Wartungsrechte werden nicht vergeben.
RLS ist auf beiden Tabellen aktiviert und erzwungen. Es gibt noch keine erlaubende Policy.
Damit liefert die Laufzeitrolle keine Zeilen und kann keine Zeile anlegen.
Eine selbst gesetzte Kontextvariable ändert dies nicht.
Die Eigentümerrolle ist ebenfalls RLS unterworfen; privilegierte Rollen können ihre eigenen Objekte technisch ändern.
Diese Migrationsmacht ist keine Laufzeitberechtigung.

## Migration und Fehler

Provisionierung und Migration sind getrennte versionierte SQL-Dateien.
Die Erstinstallation verlangt neue Rollennamen und ein neues Schema. Vorhandene Objekte werden nicht stillschweigend übernommen.
Die Migration läuft mit `psql --single-transaction` und `ON_ERROR_STOP=1`.
Eine transaktionsgebundene Sperre verhindert konkurrierende Migrationen dieses Dienstbestands.
Ein Versionssatz wird nur zusammen mit den Tabellen und Rechten bestätigt.
Ein erneutes Anwenden derselben Erstinstallation schlägt fehl und ändert keinen Bestand.
Ein künstlich ausgelöster Fehler vor der Bestätigung muss sämtliche Änderungen dieser Migration zurücknehmen.

## Test- und Abnahmegrenze

Eine wegwerfbare PostgreSQL-18.6-Instanz prüft positive Datenintegrität und negative Rechtefälle.
Nur die Test-Betriebsidentität darf synthetische Ausgangsdaten trotz RLS einfügen und vollständig lesen.
Diese Testidentität ist kein Anwendungsdienst und kein Entwurf einer produktiven Rechteprüfung.
Die Tests starten den Server mit demselben Datenbestand neu und prüfen den unveränderten Bestand.
Sie prüfen außerdem einen abgebrochenen Schreibvorgang und eine abgebrochene Erstmigration.

Dieser Schritt enthält SQL-Speicherung und Datenbanktests. Die Rust-/SQLx-Anbindung folgt gesondert.
Vor ihrer Aufnahme müssen Versionen und Sicherheitsmeldungen erneut geprüft werden.
Es gibt keinen HTTP-Endpunkt, keine automatische Startmigration und keine Freigabe personenbezogener Echtdaten.
KC-004C ergänzt die aktuelle Benutzerrechteprüfung vor jeder öffentlichen Dienstoperation.
Die vollständigen KC-005-Isolationstests für Einrichtungsschemas werden hier nicht als erledigt ausgewiesen.
