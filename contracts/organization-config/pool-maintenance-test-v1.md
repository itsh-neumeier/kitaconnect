# Vertrag: Pool- und Wartungsnachweise v1

Stand: 20.10.2026. Issue: KC-005C / #58. Voraussetzung: KC-005A/B.
Geltungsbereich: isolierter PostgreSQL-Testcontainer mit synthetischen Daten.

## Wiederverwendete Verbindungen

Ein ausschließlich im Test angelegter SQLx-Pool verwendet genau eine Verbindung und die Testidentität `kc_test_care_1`.
Nach Rückgabe setzt ein Pool-Hook Sitzungsparameter und Rolle zurück. Ein fehlgeschlagener Hook verwirft die Verbindung.
Die nächste Ausleihe prüft die ursprüngliche Datenbankidentität. Die Einrichtung stammt nicht aus einer frei gesetzten Variable.
Die PostgreSQL-Prozesskennung muss nach Erfolg, SQL-Fehler, Datenbankabbruch und verworfenener Rust-Operation gleich bleiben.
So kann eine neue Verbindung den Nachweis der Wiederverwendung nicht verdecken.

Die Tests verändern search_path und den synthetischen Sitzungswert `kc.facility_id`.
Nach Rückgabe darf kein vorheriger Einrichtungswert gelten. Eigene qualifizierte Probezugriffe müssen weiterhin funktionieren.
Rollenwechsel und Zugriff auf die andere Einrichtung müssen scheitern.
Ein verworfener Vorgang hält zuvor eine offene Transaktion mit lokalem Kontext und einer synthetischen Schreiboperation.
Nach Abbruch müssen Kontext und Schreiboperation zurückgenommen sein.
Ein PostgreSQL-Anweisungszeitlimit prüft zusätzlich den Abbruch einer laufenden Datenbankanweisung.

Der Entzug von Schema-USAGE muss auch auf einer bereits geöffneten Verbindung den nächsten Zugriff verweigern.
Eine unbekannte oder entfernte Registryzuordnung darf die Wartungsauflösung nicht erfolgreich zurückgeben.
Ein früher aufgelöster Name ist keine dauerhafte Berechtigung. Der Test behauptet keine Benutzerrechteprüfung.

## Konkurrierende Wartung

Die Einrichtungen 10, 11 und 12 sind ausschließlich synthetische Wartungsproben.
Zwei echte Verbindungen rufen die vorhandene Provisionierungs-API gleichzeitig für Einrichtung 10 auf.
Eine dritte Transaktion hält zunächst deren Sperre. Der Test muss zwei wartende PostgreSQL-Sperranforderungen beobachten.
Nach Freigabe darf genau ein Aufruf erfolgreich sein. Der andere muss die vorhandene Registrierung melden.
Schema und Registry dürfen nur einmal vorhanden sein.

Ein fest definierter Testmigrationsschritt erzeugt ausschließlich `care_wave_versions` und `care_wave_probe`.
Er akzeptiert eine typisierte Einrichtungskennung und die festgelegten Versionen 1 oder 2, keine Artefaktpfade oder Client-SQL.
Die Schemaauflösung erfolgt unter derselben transaktionsgebundenen Wartungssperre wie die Provisionierung.
Version 1 erzeugt die beiden Probetabellen. Version 2 ergänzt eine technische Spalte und die Versionsmarke gemeinsam.
Die Testadministration führt diesen privilegierten Schritt aus. Keine Laufzeitrolle erhält DDL-Rechte.
Auch konkurrierende Aufrufe desselben Versionsschritts müssen tatsächlich an der Sperre warten und genau einen neuen Stand erzeugen.

Eine Testwelle führt Einrichtung 10, dann 11 und erst danach 12 fort.
Ein eingespielter Fehler in Einrichtung 11 nimmt nur deren laufenden Versionsschritt zurück und stoppt die Welle.
Der bestätigte Stand von Einrichtung 10 bleibt erhalten. Einrichtung 12 bleibt unbearbeitet.
Ein gezielter erneuter Versuch für Einrichtung 11 muss gelingen. Ein Server-Neustart muss die bestätigten Zustände erhalten.

## Abnahmegrenzen

Dieser Vertrag definiert Testhilfen, keinen produktiven Pool, keine allgemeine Migrationssteuerung und keine neue ausgelieferte Migration.
Die Frage nach produktiver Identitäts-/Poolbindung bleibt offen. Eine ausbleibende Antwort erteilt keine Freigabe.
KC-005 kann seine technischen Teilnachweise auf dieser Grundlage dokumentieren.
Die produktive Gesamtabnahme bleibt bis zur festgelegten Identitätsbindung und deren Integration gesperrt.
Aktuelle Benutzerrechte folgen in KC-006/KC-008. Datensicherung und Wiederherstellung benötigen eigene spätere Nachweise.
Keine gesamte Mehrmandanten-Datenbank wird zur Fehlerbehandlung zurückgesetzt.
