# Eigene lokale Anmeldekennung v1

Stand: 26.10.2026. KC-006F / #67. Die Projektleitung bestätigt die eigene eindeutige Kennung je Installation unabhängig von E-Mail.

## Zweck und Grenze

Der Identitätsdienst erhält einen eigenen lokalen Zuordnungsbestand. Eine Kennung bezeichnet höchstens eine unveränderliche interne Identität.
Die Zuordnung ist kein Passwortnachweis, keine Sitzung und keine Mitgliedschaft. Gateway und Organisationsdienst erhalten keinen Tabellenzugriff.
E-Mail-Adressen, externe Identitäten und Einrichtungskennungen werden nicht automatisch als Anmeldekennung ausgewertet.
Eine unbekannte oder widerrufene Kennung ergibt intern keine Identität. Datenbankfehler bleiben getrennte allgemeine Ausfälle.
Die spätere öffentliche Anmeldung muss unbekannte Konten verdecken und die vorhandene Dummy-Passwortprüfung verwenden.
Dieser interne Leser veröffentlicht keine Kontoabfrage.

## Darstellung

Der erste interne Transporttyp enthält 1 bis 256 UTF-8-Bytes. Er vergleicht die unveränderten Bytes.
Er führt weder Trimmen, Groß-/Kleinschreibung noch Unicode-Normalisierung oder E-Mail-Auswertung aus.
Damit entstehen keine stillen Aliasnamen. Diese technische Transportgrenze legt keine spätere Eingabemaske oder Vergaberegel fest.
Der Datenbankbestand verwendet einen eindeutigen Bytewert, unabhängig von Textkollation.
Debug-Ausgaben verdecken die Kennung. Der gehaltene Eingabepuffer wird beim Freigeben gelöscht.

## Bestand und Zugriff

Migration 0012 verlangt genau die vorhandenen Versionen 1 bis 11 und verwendet die vorhandene Migrationssperre.
Sie ergänzt einen eigenen Bestand mit Kennung, internem Identitätsverweis und Zustand active/revoked.
Kennung und Identitätsbezug bleiben in diesem ersten Bestand unveränderlich. Löschen und Wiederaktivierung sind nicht implementiert.
Umbenennung, Aliasvergabe, Wiederverwendung, Registrierung und öffentliche Schreibzugriffe benötigen einen eigenen Vertrag.
Ein Widerruf verhindert neue Auflösung. Der bestehende aktuelle Sitzungs-/Mitgliedschaftsprüfer bleibt für Sitzungssperren zuständig.
Ein Registrywiderruf allein darf nicht als belegter Widerruf bestehender Sitzungen dargestellt werden.

Erzwungene Zeilensicherheit bleibt ohne erlaubende Laufzeitpolicy geschlossen, auch für den Tabelleneigentümer.
Die vorhandene Identitäts-Laufzeitrolle erhält nur SELECT; keine Schreib-, Truncate- oder Funktionsfreigabe.
Privilegierte positive Vorbereitung und vorübergehende Lesepolicies bestehen ausschließlich im isolierten synthetischen Test.

## Aktuelles Lesen und Grenzen

Der Leser besitzt je Anfrage eine neue eigene Verbindung. Er übernimmt keine fremde Transaktion und hält keinen positiven Cache.
Die Abfrage liest nur aktive genaue Zuordnung. Sie verwendet gebundene Parameter und einen einzigen aktuellen Anweisungsstand.
Parallelität, Verbindungsfrist, Datenbankfrist und Gesamtfrist werden ausdrücklich positiv übergeben.
Ein belegtes Parallelbudget verweigert weitere Arbeit ohne Warteschlange. Ablauf und Abbruch geben Verbindung und Budget frei.
Datenbankseitige Anweisungs-/Transaktionsfristen ergänzen die äußere Frist. Diagnosen enthalten weder Kennung noch SQL-Details.

## Akzeptanznachweise

Reine Tests prüfen Bytegrenzen, fehlende Normalisierung, verdeckte Diagnose und ungültige Arbeitsgrenzen.
Die tatsächliche SQLx-Gruppe prüft genaue Auflösung, unbekannte Kennung, Widerruf, Eindeutigkeit und unveränderliche Zuordnung.
Sie prüft Rollback, tatsächlich wartende Schreibkonkurrenz, erzwungene Zeilensicherheit, Rechteentzug und Ausfall.
Ein unabhängiger Datenbankzeuge prüft die Freigabe wartender Verbindungen bei Zeitablauf und Abbruch.
Nach Datenbankneustart werden erhaltene Zuordnungen und Widerruf vor neuer Vorbereitung geprüft.
Migration, Versionseintrag und neue Objekte müssen bei Fehler gemeinsam zurückgenommen werden.
Dies ist kein positiver Browser-, CSRF-, Faktor- oder Gesamtabnahmenachweis.
