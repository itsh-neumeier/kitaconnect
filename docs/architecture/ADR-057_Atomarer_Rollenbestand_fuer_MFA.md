# ADR-057: Atomarer Rollenbestand für MFA

- Datum: 25.10.2026
- Status: Technische Konkretisierung vor Code
- Umfang: KC-008A; keine neue Fachregel

## Entscheidung

Der [Speichervertrag](../../contracts/identity-access/rights-assurance-v1.md) führt einen eigenen atomaren Rollenbestand je Identität und Einrichtung ein.
Das gesamte eingestufte Dokument steht mit Version und Lebenszyklus in einer Zeile.
Die MFA-Anforderung wird aus den tatsächlichen Klassen aller enthaltenen Rollen abgeleitet.
Ein gesonderter manueller MFA-Schalter ist ausgeschlossen.
Die bestehende Zuordnung aus ADR-043 wird gemeinsam verwendet.
Die Laufzeit bleibt durch RLS geschlossen. Wartung und synthetische Testdaten sind getrennt.

## Begründung

Ein verteilter Rollenleser könnte bei teilweise sichtbaren Zeilen eine sensible Rolle übersehen.
Ein atomarer Bestand verhindert diese Abschwächung durch teilweise gefilterte Unterzeilen.
Eine einzelne aktuelle SQL-Anweisung verhindert gemischte Versionsstände während einer Änderung.
Unbekannte oder unvollständige Zuordnungen erzeugen keine schwächere Anmeldeanforderung.
Versionierte Ersetzung und endgültiger Widerruf vermeiden eine Wiederverwendung alter Bestände.

## Grenzen und Folgen

Der Bestand ist die dienstlokale technische Einstufung zugeordneter Rollen; kein synchronisierter Cache und keine zweite erlaubende Quelle.
Es entstehen weder Standardrollen noch Objekt-/Feldrechte oder eine Vergabe-API.
Trägerweite Zuordnungen müssen vor einem aktiven Einrichtungsbestand vollständig aufgelöst sein.
Diese Entscheidung erfindet dafür keine Fachregel. Unbekannte Zuordnungen bleiben gesperrt.
Eine spätere Rechteverwaltung muss Bestand und Änderungsversion gemeinsam pflegen oder diesen Vertrag durch eine geprüfte Migration ersetzen.
Die spätere atomare Einbindung in Anmeldung und Sitzungsprüfung bleibt KC-006F vorbehalten.
Der Leser behauptet nur Aktualität seines Anweisungssnapshots.

## Nachgewiesene Transaktionsgrenze vor Korrektur

Die tatsächliche SQLx-Probe am Kopf 8914758 zeigt: `is_in_transaction()` erkennt ein rohes SQL-`BEGIN` nicht.
Eine bloße Prüfung des SQLx-Zählers kann die behauptete Autocommit-Grenze deshalb nicht sichern.
Der öffentliche Leser erstellt künftig nur eigene frische begrenzte Verbindungen.
Er nimmt keine vorhandene Verbindung an. Der unveränderte aktuelle SQL-Snapshot bleibt erhalten.
Eine tatsächliche Gegenprobe hält eine alte rohe Transaktionssicht offen und verlangt vom eigenen Leser trotzdem die neue bestätigte Version.
Zeit- und Mengenbudgets sowie Datenbankgrenzen werden vor dem Korrekturcode im Vertrag festgelegt.
Die erlaubten Datenklassen und fachlichen Rechte bleiben unverändert.
