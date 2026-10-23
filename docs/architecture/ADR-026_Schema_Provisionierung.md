# ADR-026 – Atomare Einrichtungsschema-Registrierung

Datum: 20.10.2026. Status: technische Konkretisierung des P1-Auftrags für KC-005A.
Grundlage: ADR-014; keine Änderung fachlicher Berechtigungen.

## Entscheidung

Der [Wartungsvertrag](../../contracts/organization-config/schema-provisioning-v1.md) wird vor der Implementierung versioniert.
Die Schema-Registry bleibt beim zuständigen Dienst `organization-config`.
Eine weitere Migration ergänzt den Bestand. Neue Wartungsrollen werden getrennt provisioniert und bleiben ohne Login.
Die privilegierte CREATE-Fähigkeit ist ausschließlich für geprüfte Betriebszugänge bestimmt.
Es gibt keine automatische Anwendungsstartmigration oder öffentliche Provisionierungsschnittstelle.

Einrichtungsschemas verwenden eine feste Darstellung der bereits vorhandenen u128-Kennung.
Schema und Registry werden gemeinsam bestätigt oder zurückgenommen.
Eine globale dienstbezogene Transaktionssperre hält diese erste Stufe einfach prüfbar.
Spätere parallele Migrationswellen benötigen eigene Tests und eine begründete Verfeinerung.

Die bestehenden Runtime-Rollen erhalten keine zusätzlichen Rechte.
Erlaubende Registry- und Kennungslese-Policies gelten ausschließlich für die ausdrücklich privilegierte Wartungsrolle.
Fachliche Benutzerrechte, Personalbereiche und Wahldatenbanken bleiben außerhalb dieses Schritts.

## Alternativen und Folgen

Client-Schemanamen und `CREATE SCHEMA IF NOT EXISTS` würden eine fremde vorhandene Struktur verdecken.
Beide Varianten werden verworfen. Die Auflösung prüft auch den physischen Eigentümer und PUBLIC-/Runtime-Rechte.
Eine gespeicherte Registrierung allein beweist keinen sicheren aktuellen Zustand.
Unsichere Runtime-Attribute, Mitgliedschaften oder Eigentum führen vor Wartungsoperationen zur Verweigerung.

## Planung ohne Abhängigkeitskreis

KC-004 bleibt als Gesamtissue bis KC-004C offen. KC-004C benötigt KC-008, dieses wiederum KC-005.
KC-005 und KC-006 benötigen daher das konkrete Grundmodell und die Persistenz aus KC-004A/B, nicht den Abschluss von KC-004C.
Der Backlog bildet Teilpakete ausdrücklich ab. Die Prüfung berücksichtigt Teilpakete bei der Gesamtabnahme und erkennt dadurch auch verdeckte Zyklen.
Dies präzisiert die technische Reihenfolge; keine ursprüngliche Akzeptanzbedingung entfällt.

## Quellen

- [PostgreSQL: CREATE SCHEMA und Eigentümerrechte](https://www.postgresql.org/docs/18/sql-createschema.html)
- [PostgreSQL: Rollenbasierte RLS](https://www.postgresql.org/docs/18/ddl-rowsecurity.html)
- [PostgreSQL: Transaktionsgebundene Sperren](https://www.postgresql.org/docs/18/explicit-locking.html)
