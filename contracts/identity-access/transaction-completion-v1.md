# Dienstvertrag: Abschluss eigener Datenbanktransaktionen v1

Stand: 21.10.2026. Teil von KC-006F / #67. Nur interne Speicheradapter.

## Ausgangslage

SQLx 0.9 merkt beim Verwerfen einer Transaktion einen Rollback vor.
Eine offen gehaltene Verbindung kann bis zur nächsten Verwendung serverseitige Sperren behalten.
PR #77 bestätigt diesen Fehler und die Korrektur für Sitzungsprüfung, Rotation und Abmeldung.

## Anforderungen

1. Ein Adapter MUSS seine eigene Transaktion vor einer regulären Rückgabe abschließen.
2. Bei Erfolg MUSS er Commit abwarten. Bei einem Fehler vor Commit MUSS er Rollback abwarten.
3. Ein Fehler beim Rollback MUSS Zugriff verweigern. Die Verbindung DARF ohne bestätigte Bereinigung nicht wiederverwendet werden.
4. Bereits bestätigte Versuchszähler und verbrauchte Herausforderungen bleiben dauerhaft verbraucht.
5. Ein absichtlich gespeicherter Ablaufzustand bleibt bestätigt, auch wenn das Ergebnis die Anmeldung verweigert.
6. Ein Adapter DARF eine äußere Transaktion nicht eigenmächtig beenden. Bestehende Ablehnung äußerer Transaktionen bleibt erhalten.
7. Bei abgebrochenen Rust-Futures kann kein asynchroner Abschluss zugesagt werden.
   Der Aufrufer MUSS die Verbindung verwerfen oder vor Wiederverwendung deren Rücknahme bestätigen.
8. Ein fehlgeschlagener oder unklarer Commit DARF kein positives Ergebnis erzeugen.

## Nachweise und Umfang

Eine getrennte Zeugenverbindung prüft nach einem Fehler den Datenbankprozess der offen gehaltenen Fehlerverbindung.
Der Prozess MUSS ohne laufende Transaktion im Zustand `idle` sein.
Eine Folgeoperation auf der Fehlerverbindung ist vor dieser Prüfung unzulässig, weil sie den Fehler verdecken kann.
Tests prüfen zusätzlich Rücknahme, erhaltene Versuchszähler und bleibende Ablaufzustände vor und nach Neustart.

Der erste Korrekturschritt umfasst Sitzungsbestand, Passwortadapter und Einrichtungseinstellungen.
TOTP, WebAuthn und zusammengesetzte Anmeldung folgen nach derselben Prüfung getrennt.
Migrationen, Abhängigkeiten, fachliche Rechte und produktive Grenzwerte bleiben unverändert.

## Folgeschritt: Faktoradapter und Anmeldeabschluss

Der Folgebranch nach PR #78 korrigiert TOTP, WebAuthn und den zusammengesetzten Anmeldeabschluss.
Interne Hilfsfunktionen, die eine laufende eigene Transaktion an ihren Aufrufer übergeben, beenden nur ihren Fehlerpfad.
Der Aufrufer bleibt für den erfolgreichen Abschluss verantwortlich.
Der WebAuthn-Verbrauch bestätigt die Herausforderung vor der anschließenden Bibliotheksprüfung weiterhin dauerhaft.
Der dauerhafte Verbrauch von Verschlüsselungs-Nonces bleibt außerhalb der späteren Faktortransaktion bestehen.
Eine bestätigte Ablaufmarkierung im Anmeldevorgang wird als absichtlich negatives Ergebnis gemeinsam mit geprüften Faktorzuständen bestätigt.
Die getrennten Zeugen prüfen jeweils die offen gehaltene Fehlerverbindung vor jedem weiteren Zugriff auf diese Verbindung.
