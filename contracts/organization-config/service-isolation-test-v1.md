# Vertrag: Technischer Nachweis der Dienstisolation v1

Stand: 20.10.2026. Geltungsbereich: KC-005B / #57, ausschließlich Testcontainer.
Voraussetzung: registrierte Einrichtungsschemas aus KC-005A.

## Bestand und Identitäten

Die Testeinrichtungen 1 und 2 verwenden ihre registrierten Schemas.
Die vorhandenen Dienstgrenzen `attendance-care` und `meals` erhalten je eine technische Probe.
Diese Proben bilden keine Fachobjekte ab. Alle Werte sind synthetische Ganzzahlen.

| Dienst | Testobjekte je Schema | Eigentümer | Migration | Testzugänge |
|---|---|---|---|---|
| attendance-care | care_probe, care_probe_count() | kc_test_care_owner | kc_test_care_migrator | kc_test_care_1, kc_test_care_2 |
| meals | meals_probe, meals_probe_count() | kc_test_meals_owner | kc_test_meals_migrator | kc_test_meals_1, kc_test_meals_2 |

Die Eigentümer haben keinen Login. Nur der zugeordnete Test-Migrationszugang darf seine Eigentümerrolle annehmen.
Test-Laufzeitrollen haben keine Rollenmitgliedschaft, kein Eigentum, kein CREATE und keine besonderen Rollenattribute.
Die Testdatenbank erlaubt ihnen auch keine temporären Objekte.
Loginrollen entstehen nur im isolierten, anschließend entfernten Testcontainer. Es gibt keine Passwörter.

## Freigegebenes Testartefakt

Das einzige Artefakt ist `services/organization-config/database/tests/service-isolation.sql`.
Der Rust-Test bindet diese versionierte Datei beim Kompilieren ein. Er akzeptiert keinen Dateipfad und kein Client-SQL.
Vor Anlage prüft das Artefakt beide Registryeinträge und die erwarteten Schemaeigentümer.
Objekte, Rechte, Policies und Funktionsdefinitionen entstehen in einer Transaktion.
CREATE im Einrichtungsschema wird nur während der privilegierten Anlage gewährt und vor Bestätigung entzogen.
Die Objektliste wird nach Anlage aus dem PostgreSQL-Katalog geprüft.
Eine eingespielte Störung muss auch Rollen, Rechte und Objekte zurücknehmen. Danach muss ein sauberer Versuch gelingen.
Dies ist noch keine allgemeine Migrationssteuerung oder kryptografische Artefaktfreigabe.

## Rechte und RLS

Jeder Testzugang erhält USAGE nur für sein Einrichtungsschema und SELECT/INSERT/UPDATE/DELETE nur auf seiner Dienstprobe.
Beide Tabellen verwenden ENABLE und FORCE ROW LEVEL SECURITY.
Die Testpolicy bindet Einrichtung und Dienst an den tatsächlich angemeldeten Datenbankbenutzer.
Ein frei gesetztes `kc.facility_id`, ein anderer search_path oder ein RESET ROLE ändern diese Bindung nicht.
Fremde Testzugänge, Eigentümer- und Migrationsrollen erhalten keine erlaubende RLS-Policy.
Die Testadministration legt absichtlich je eine fremde Zeile an. RLS muss diese trotz Tabellenrecht ausblenden.
Schreibversuche auf fremde Einrichtungskennungen scheitern oder betreffen keine Zeile.

Zählfunktionen verwenden SECURITY INVOKER, einen festen search_path und vollständig qualifizierte Tabellen.
PUBLIC erhält kein EXECUTE. Nur der zugeordnete Testzugang darf die Funktion aufrufen.
Ein privilegiert angelegtes Schattenschema muss zeigen, dass qualifizierte Aufrufe keine fremden Objekte verwenden.
Negative Tests prüfen Rollenwechsel, Eigentümerwechsel, deaktivierte RLS, TRUNCATE und fremde Dienstobjekte.

## Grenzen und Fortsetzung

Die Produktionsprovisionierung und die Runtime-Rolle `kc_org_runtime` bleiben unverändert gesperrt.
Die Loginbindung ist ausschließlich ein kontrollierter Testaufbau, keine Entscheidung über produktive Poolidentitäten.
Vor produktiver erlaubender RLS muss die Projektleitung einen überprüfbaren Identitätsnachweis freigeben.
Die Anbindung an aktuelle Benutzerrechte folgt in KC-006/KC-008; Pool- und Konkurrenztests folgen in KC-005C.
Dieser Vertrag gewährt keine Benutzerrechte und implementiert keine Fachfunktionen.
