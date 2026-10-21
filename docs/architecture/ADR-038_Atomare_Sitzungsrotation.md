# ADR-038 – Atomare Rotation innerhalb derselben Anmeldung

Datum: 21.10.2026. Status: technische Konkretisierung vor Code für KC-006F.
Grundlage: bestätigter Browservertrag und ADR-030/036/037.

## Entscheidung

Der [Rotationsvertrag](../../contracts/identity-access/session-rotation-v1.md) verwendet einen neuen Datensatz und einen neuen zufälligen Nachweis.
Der alte Datensatz bleibt endgültig widerrufen. Die unveränderlichen Felder des bestehenden Sitzungsvertrags bleiben dadurch geschützt.
Beide Datensätze gehören zur selben einmal geprüften Anmeldung. Die Authentisierungszeit wird nicht verjüngt.
Ein gesonderter unveränderlicher Nachfolgerbestand begrenzt jeden Vorgänger auf einen Nachfolger.
Die letzte aktuelle Sitzungsprüfung wird innerhalb derselben Transaktion und Identitätssperre ausgeführt.

Abmeldung widerruft die Nachfolger derselben Anmeldung. Sie lässt andere Anmeldungen der Identität unverändert.
Damit bleibt Abmeldung auch wirksam, wenn eine Rotation unmittelbar zuvor gewinnt.
Ein interner Savepoint erlaubt, bei Ablauf während einer Schreibwartezeit nur den Kandidaten zu entfernen und den beobachteten Ablauf dauerhaft zu bestätigen.

## Alternativen und Folgen

Die Änderung des Nachweisfelds im ursprünglichen Datensatz wurde verworfen: Der bestehende Schutztrigger macht dieses Feld unveränderlich.
Eine vollständig neue Anmeldung wurde verworfen: Rotation ist keine neue Faktorprüfung und darf keine Fristen oder Stärke erneuern.
Migration 0007 erlaubt ausschließlich die benötigte Mehrfachbindung des ursprünglichen Vorgangs für Rotationsnachfolger.
Der einmalige Verbrauch bleibt durch den bestehenden Sperrvertrag geschützt.

## Grenzen

Keine fachlichen Rechte, produktiven Rollen, Schlüssel oder neuen Kryptoverfahren.
Keine Browser- oder Dienstidentitätsfreigabe. Der aktuelle Organisationsvertrag bleibt verpflichtend.
Die spätere Aufbewahrung der Sitzungs- und Nachfolgerbestände benötigt weiterhin einen eigenen Betriebsvertrag.
