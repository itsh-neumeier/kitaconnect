# Einrichtungsschemas v1 – privilegierter Wartungsvertrag

Stand: 20.10.2026. Issue: KC-005A / #56. Grundlage: ADR-014 und Speichervertrag.

## Umfang und Identität

Die Wartungsoperation erhält eine bestehende PostgreSQL-Verbindung und eine typisierte Einrichtungskennung.
Sie akzeptiert keine Schemanamen, SQL-Fragmente, Rollenlisten oder Client-Header.
Die Verbindung muss die gesonderten Provisionierungsrechte besitzen. Die Operation führt keinen Rollenwechsel aus.
Sie ist kein öffentlicher Endpunkt und keine Benutzerrechteprüfung.
Sie erzeugt nur ein leeres Einrichtungsschema und dessen Registrierung, keine Fachtabellen oder Laufzeitfreigabe.
Personalbereiche und Wahldatenbanken bleiben ausgenommen.

## Namen und Zustand

Der Schemaname lautet `facility_` mit genau 32 kleinen hexadezimalen Ziffern der Einrichtungskennung.
Er ist damit eindeutig und bleibt unter der PostgreSQL-Grenze für Bezeichner.
Schema und Eintrag in `organization_config.facility_schemas` entstehen in einer Transaktion.
Eine dienstbezogene transaktionsgebundene Sperre serialisiert die erste Provisionierungsstufe.
Unbekannte Einrichtungen, bestehende Registrierungen und bestehende physische Schemata werden abgewiesen.
Eine bereits vorhandene Struktur wird nicht stillschweigend übernommen.
Bei einem Fehler werden die gesamte Anlage und ihre Registrierung zurückgenommen.

## Rollen und Registry

- `kc_facility_schema_owner`: gesonderte Schemaeigentümerrolle ohne Login und Sonderattribute.
- `kc_facility_provisioner`: gesonderte Wartungsrolle ohne Login und Sonderattribute.
  Sie erhält CREATE auf der gewählten Datenbank und darf die Schemaeigentümerrolle technisch setzen.
  Dies ist privilegierte Betriebsfähigkeit; sie darf niemals einer Laufzeitidentität zugeordnet werden.
- Die Wartungsrolle darf ausschließlich Einrichtungskennungen lesen sowie Schema-Metadaten lesen und einfügen.
- Sie darf keine Trägerzuordnung ändern, Fachinhalte lesen oder bestehende Registrierungen ändern/löschen.
- Die Registry hat erzwungene RLS. Erlaubende Policies gelten ausschließlich für die gesonderte Wartungsrolle.
  Die vorhandene Runtime-Rolle erhält keinen Registry-Zugriff oder Zugriff auf die neuen leeren Schemata.

Vor Anlage und Auflösung wird die bestehende Runtime-Rolle `kc_org_runtime` aus dem Datenbankkatalog geprüft.
Login, Superuser, BYPASSRLS, CREATEDB, CREATEROLE, Replikation, Datenbank-CREATE,
Objekteigentum oder eine eigene Rollenmitgliedschaft führen zur Verweigerung.
Die Prüfung gilt zunächst für dieses bereits vorhandene dienstbezogene Rechtebündel.
Weitere Dienste benötigen eine eigene geprüfte Rollen-/Objektzuordnung in KC-005B.

Die Auflösung einer Einrichtung prüft Registry, kanonischen Namen, physischen Eigentümer und fehlende PUBLIC-/Runtime-Rechte.
Eine fehlende Registrierung liefert keinen Treffer. Abweichungen erzeugen einen Fehler.
Ein zurückgegebener Name ist keine Benutzerberechtigung oder dauerhafte Zugriffszusage.

## Nachweise und Grenzen

Echte PostgreSQL-/SQLx-Tests prüfen atomare Anlage, Doppelanlage, fremde Namen, unbekannte Einrichtung,
fehlerhafte Registrierung nach Schemaanlage, unsichere Runtime-Rolle, falschen Eigentümer und Registry-Manipulation.
Der Registry- und Schema-Zustand wird nach echtem Server-Neustart erneut geprüft.
Die Registry-Erweiterung ist eine eigene Migration nach Version 0001.
Bestehende Migrationen werden nicht nachträglich umgeschrieben.

KC-005B ergänzt Dienstobjekte und RLS-Isolation. KC-005C ergänzt Poolwechsel und Migrationsabläufe.
Dieser erste Schritt behauptet keine vollständige KC-005-Abnahme oder produktive Mandantenisolation.
