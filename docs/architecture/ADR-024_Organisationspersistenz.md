# ADR-024 – Restriktive Organisationspersistenz

Datum: 19.10.2026. Status: technische Festlegung für KC-004B (#51).
Grundlage: P1-Auftrag, ADR-014 und ADR-023.

## Entscheidung

Der [Speichervertrag v1](../../contracts/organization-config/storage-v1.md) wird vor der Migration versioniert.
Organisationsreferenzen liegen im eigenen Plattform-Schema des zuständigen Dienstes.
Kennungen werden als unbeschränkter PostgreSQL-Numeric-Wert mit Bereichs- und Ganzzahlprüfung gespeichert.
Eine feste Skala könnte Bruchwerte vor der Prüfung runden. Sie wird deshalb nicht verwendet.
Fremdschlüssel bleiben innerhalb dieses Dienstbestands.

Eigentümer, Migration und Laufzeit erhalten getrennte Rollen ohne Login.
Die Laufzeit erhält keine Eigentümermitgliedschaft und keine Änderungsrechte an bestehenden Zuordnungen.
RLS wird aktiviert und erzwungen. Ohne die spätere aktuelle Rechteprüfung wird keine erlaubende Policy eingeführt.
Eine frei gesetzte Sitzungsvariable oder ein Client-Header darf keine Zugriffserlaubnis erzeugen.

Die Datenbanktests laufen im bestehenden verpflichtenden Prüfschritt `rust`.
Der Prüfschritt bleibt dadurch blockierend, ohne einen neuen ungeschützten Status einzuführen.
Sie verwenden das offizielle PostgreSQL-18.6-Bookworm-Image mit festgelegtem OCI-Digest.
Der Testcontainer öffnet keinen Host-Port. Befehle laufen über `docker exec` und einen lokalen Socket im Container.
Nur dieser kurzlebige Container verwendet eine vertrauensbasierte lokale Testanmeldung.
Dies ist kein produktives Authentisierungsprofil.

## Alternativen und Folgen

Eine frühe RLS-Policy auf Grundlage selbst gesetzter Kontextwerte wäre kein geprüfter Identitätsnachweis.
Ein allgemeiner Laufzeit-Vollzugriff würde die späteren Dienstgrenzen schwächen.
Beide Varianten werden verworfen. Bis KC-004C bleibt die Datenbank für Laufzeitdatenzugriffe gesperrt.

SQLx wird in diesem Schritt nicht eingebunden. Die Migration und echten PostgreSQL-Tests können ohne neue Rust-Abhängigkeit geprüft werden.
Die Rust-Anbindung bleibt als nächster Teil von KC-004B offen.
Die Tests belegen keine vollständige Mandantenisolation oder Benutzerberechtigung.

## Quellen und Versionsnachweis

Am 19.10.2026 wurden die offiziellen PostgreSQL-Unterlagen und das Manifest des offiziellen Images geprüft.
Image: `postgres:18.6-bookworm@sha256:afc7e2d441324c0388fa80c3d24f733b4194a4eb7f47dd8ee2b08eb1a24a647c`.
Dies ist ein CI-Testartefakt, keine Freigabe des späteren Betriebsprofils.

- [PostgreSQL: RLS und standardmäßige Verweigerung](https://www.postgresql.org/docs/18/ddl-rowsecurity.html)
- [PostgreSQL: Rollenrechte](https://www.postgresql.org/docs/18/sql-grant.html)
- [PostgreSQL: Numeric-Datentypen](https://www.postgresql.org/docs/18/datatype-numeric.html)
- [PostgreSQL: psql und Einzeltransaktionen](https://www.postgresql.org/docs/18/app-psql.html)
