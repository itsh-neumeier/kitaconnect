# ADR-020 – Redis Streams mit dauerhafter Auftragsquelle in PostgreSQL

Datum: 18.10.2026. Status: technische Festlegung im delegierten P0-Umfang von KC-037.
Grundlage: v5.9 fordert PostgreSQL als dauerhafte Quelle, Redis, Outbox und idempotente Verarbeitung.
Keine Implementierung in P0. O-004 wird auf Architekturebene konkretisiert.

## Entscheidung

Redis Streams transportieren Aufträge über Verbrauchergruppen.
Der zuständige Dienst schreibt Fachänderung und Outbox-Eintrag in derselben PostgreSQL-Transaktion.
Eine Outbox ist der dauerhafte Bestand noch zuzustellender Ereignisse.
Der Hintergrunddienst verarbeitet eine stabile Auftragskennung ohne doppelte fachliche Wirkung.
Die Zustellung kann mehrfach erfolgen. Es gibt keine pauschale Genau-einmal-Zusage.

Redis enthält nur die nötigen Referenzen und minimierte Metadaten.
PostgreSQL enthält Auftragszustand, erwartete Empfänger, Versuchszähler, nächste Ausführung und Ergebnis.
Die technische Kennung wird pro Auftrag und Empfänger eindeutig verwendet.
Zustellzustände und Schutzklassen bleiben beim jeweils zuständigen Dienst.
Redis-Persistenz und eine erfolgreiche Übergabe an Redis gelten allein nicht als fachliche Bestätigung.

## Verarbeitung und Wiederanlauf

1. Der Zustelldienst überträgt fällige Aufträge aus der Outbox an den passenden Stream.
2. Der Verbraucher übernimmt den Auftrag und prüft dessen aktuellen Berechtigungsumfang sowie Frist und Sperrstatus.
3. Er prüft in seinem dauerhaften Bestand, ob derselbe Auftrag für denselben Empfänger bereits wirksam war.
4. Er speichert lokale Fachwirkung und Verarbeitungsnachweis in derselben Datenbanktransaktion.
5. Erst nach dauerhaftem Erfolg bestätigt er die Stream-Nachricht.
6. Fehlende Bestätigungen können nach einem Abbruch erneut übernommen werden.

Ein Abbruch zwischen Datenbankabschluss und Stream-Bestätigung darf keine doppelte Fachwirkung erzeugen.
Nach Redis-Verlust werden nicht dauerhaft abgeschlossene Zustellungen aus PostgreSQL erneut aufgebaut.
Eine Markierung „an Redis gesendet“ darf den Auftrag deshalb nicht endgültig entfernen.
Aufbewahrung und Löschung der Zustellnachweise folgen der bestätigten Kategorie und Rechtsgrundlage.
Ein erneut zugestellter Auftrag erhält durch den Wiederanlauf keine neuen Rechte.

## Technische Startwerte und Fehlerablage

Normale Kurzaufträge erhalten zunächst 60 Sekunden Bearbeitungszeit und 120 Sekunden Übernahmefrist bei Untätigkeit.
Längere Medienaufträge benötigen ein eigenes geprüftes Laufzeit- und Lebenszeichenprofil.
Eine Zeitüberschreitung allein darf kein zweites ungeschütztes Schreiben auslösen.
Verbraucher prüfen bei konkurrierender Übernahme weiterhin den dauerhaften Verarbeitungsnachweis.

Es gibt höchstens 10 automatische Versuche einschließlich des ersten Versuchs.
Der Abstand steigt von 1 Sekunde exponentiell bis höchstens 300 Sekunden, mit zufälliger Streuung.
Fachliche Fristen, Widerrufe und dauerhafte Fehler können die Wiederholung vorher beenden.
Nach dem letzten Fehlversuch verbleibt der Auftrag als Fehlerfall in PostgreSQL.
Er verschwindet nicht durch eine Redis-Bereinigung. Eine erneute Ausführung benötigt eine berechtigte Aktion.
Ausgabe und Fehlerablage enthalten keine unnötigen Fachinhalte oder Zugangsdaten.

Externe Wirkungen wie SMTP-Versand sind nicht Teil einer PostgreSQL-Transaktion.
Ein Absturz nach externem Erfolg kann dort trotz lokaler Absicherung zu Wiederholung führen.
KC-028 MUSS je Integration Idempotenzmöglichkeiten und das verbleibende Risiko dokumentieren.

## Rechte und Ausnahmen

Dienstbezogene Redis-Rechte begrenzen Stream-Namen und erlaubte Befehle.
Die Anwendungsprüfung erzwingt zusätzlich Einrichtung, Rolle, Zweck, Objekt und Datenfelder.
Ein Stream-Präfix oder eine allgemeine Verbrauchergruppe ist kein Nachweis von Mandantenisolation.
Stimmabgabe und geheime Wahlabläufe dürfen diese allgemeine Infrastruktur nicht ungeprüft verwenden.
Ihre getrennte Verarbeitung bleibt ADR-017 und der unabhängigen Protokollprüfung vorbehalten.

## Alternative und Nachweise

Redis Pub/Sub wurde nicht gewählt, weil die Verarbeitung explizite Bestätigungen und Wiederübernahme benötigt.
Ein zusätzlicher Nachrichtenbroker wird in P0 nicht eingeführt.
KC-028/KC-029 und KC-032 MÜSSEN Abbruch, Doppelzustellung, Redis-Verlust,
konkurrierende Übernahme, Rechteentzug und erschöpfte Versuche prüfen.
Die Startwerte werden vor Einsatz mit Laufzeit- und Lastnachweisen geprüft.

Quellen: [XREADGROUP](https://redis.io/docs/latest/commands/xreadgroup/),
[XAUTOCLAIM](https://redis.io/docs/latest/commands/xautoclaim/).
Die Zeit- und Versuchswerte sind technische Projektfestlegungen und keine Vorgaben dieser Quellen.
