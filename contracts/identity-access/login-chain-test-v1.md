# Tatsächliche Anmeldekette: Prüfvertrag v1

Stand: 26.10.2026. KC-006F / #67. Ausschließlich isolierte synthetische Tests.

## Umfang und Aufbau

Der Test verbindet vorhandene Faktor-Bibliotheken, dauerhaften Anmeldeabschluss und Sitzungsprüfung mit der tatsächlichen Quellenkette.
Der Anmeldeport verwendet den vorhandenen Quellenleser. Es gibt kein positives Mitgliedschafts- oder MFA-Testdoppel.
Jede aktuelle Prüfung verwendet neue Dienstanmeldung über mTLS, tatsächliche OAuth-Ausgabe, Organisationsrouter,
aktuelle Introspektion und getrennte Identitäts-/Organisationsdatenbanken.
Die Identitätsdatenbank des tatsächlichen HTTP-Tests erhält zusätzlich die vorhandene Migration 0011.
Es entstehen keine neue Anwendungsmigration, Abhängigkeit, fachliche Erlaubnis oder erlaubende Laufzeitpolicy.
Positive Daten entstehen nur durch privilegierte synthetische Vorbereitung im eigenen Testcontainer.
Zertifikate, Passkey-Testschlüssel, MFA-Schlüssel und rohe Nachweise bestehen ausschließlich zur Laufzeit im Arbeitsspeicher.

## Prüffälle

1. Passwort: Prüfen Sie falsches Passwort, verweigerten Abschluss ohne Beweis, richtigen Beweis, einmaligen Abschluss,
   aktuelle Sitzung und Rotation. Der alte Nachweis darf nach Rotation keinen Zugriff erhalten.
2. Zusatzprüfung: Prüfen Sie die tatsächliche Passkey-Bibliothek mit Benutzerverifikation und die daran gebundene Rotation.
3. Rollen und Fristen: Erhöhen Sie Rollen- und Einstellungsversion unabhängig. Prüfen Sie MFA-Pflicht,
   gekürzte Fristen, unbekannte Einstufung und die erneute aktuelle Abfrage nach einer gültigen höheren Version.
4. Passkey: Prüfen Sie fehlenden Beweis, gültige Bibliothekssignatur, MFA-Stärke, genaue Einrichtung und dauerhafte Quellenversionen.
5. Mitgliedschaft: Entziehen Sie die Zuordnung im Organisationsbestand. Die nächste Prüfung muss verweigern.
   Eine erneute Zuordnung erhält eine höhere Version. Alte Sitzungen bleiben gesperrt; ein neuer Anmeldevorgang kann erfolgreich sein.
6. Passwort plus TOTP: Prüfen Sie verweigerten Abschluss nach nur einem Faktor, tatsächliche TOTP-Prüfung,
   erfolgreichen MFA-Abschluss und die Verweigerung eines erneut verwendeten Zeitschritts.
7. Wiederherstellung: Verbrauchen Sie einen Offline-Code einmalig. Prüfen Sie Widerruf bestehender Sitzungen,
   die Sperre normaler Anmeldung und die notwendige Faktorneueinrichtung mit dem eingeschränkten Wiederherstellungsnachweis.
   Erst danach darf ein neuer vollständiger Anmeldevorgang mit neuen Faktoren erfolgreich sein.
8. Ausfall: Prüfen Sie fehlenden Rollenbestand, geschlossene Rollen-RLS, falschen Ausstellerfingerabdruck,
   nicht verfügbare Quelle und fehlende Dienstfreigabe. Ein bisher positives Ergebnis darf keinen Zugriff ersetzen.
9. Entzug: Prüfen Sie eine widerrufene Rolle gegen eine zuvor gültige Sitzung.
10. Neustart: Prüfen Sie vor neuer Vorbereitung die erhaltenen Sitzungszustände und Quellenversionen.

Fehler müssen die lokale Transaktion vor Rückgabe beenden. Ein unabhängiger Datenbankzeuge prüft dies bei ausgewählten Fehlern.
Die bestehende tatsächliche HTTP-/SQLx-Gruppe führt diese Fälle vor und nach PostgreSQL-Neustart aus.
Die Gesamtzahl bleibt zwanzig SQLx-Gruppen. Der aktuelle CI-Lauf muss deren tatsächlichen Erfolg belegen.
Während längerer Faktorprüfungen gehaltene reine Testverbindungen werden vor späteren Protokollprüfungen erneuert.
Die vorhandenen produktiven Transportregeln und Zeitgrenzen werden nicht gelockert.

## Speichergrenze der zusammengesetzten Test-Futures

Die erste native Ausführung am Kopf dcee0ab endet mit einem Stacküberlauf des erweiterten asynchronen Tests.
Die zusätzliche Zusammensetzung wird deshalb an ihren Testgrenzen mit `Box::pin` auf dem Heap gehalten.
Die betroffenen Futures bleiben im selben besitzenden Aufruf und werden vollständig abgewartet.
Es entstehen keine abgelösten Aufgaben. Stack-Vorgaben, Prüfkriterien und Laufzeitgrenzen werden nicht erhöht.
Diese Änderung betrifft ausschließlich den Testaufbau.

## Grenzen

Die Bibliotheksprüfungen verwenden den Soft-Passkey der bestehenden Testbibliothek. Sie beweisen keine physische Gerätebiometrie.
Anmeldeaufrufe erfolgen über Rust-Schnittstellen. Der Test beweist weder einen öffentlichen Anmelde-HTTP-Endpunkt
noch Gateway, Browser, Cookies, CSRF oder produktive Rollen-/Poolbindung.
Diese ursprünglichen Kriterien und die persönliche/sprachliche Durchsicht bleiben gesondert.
