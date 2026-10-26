# Dauerhafte Bindung aktueller Anmeldequellen v1

Stand: 26.10.2026. KC-006F / #67. Ergänzt die aktuelle Quellenkette nach ADR-058.

## Kontext und Übergang

Der tatsächliche Quellenleser implementiert den vorhandenen vertrauenswürdigen Anmeldeport.
Der Kontext enthält neben Einrichtungseinstellungen alle drei getrennten Versionen sowie die rohe rollenbezogene MFA-Anforderung.
Die bisherige einversionige Konstruktion bleibt nur für frühere interne Testbestände ohne Migration 0011 verwendbar.
Nach Migration 0011 verweigert jede Anmelde- und Sitzungsprüfung einen unvollständigen Kontext oder fehlenden Versionsbestand.
Es gibt keine Ersatzversion, keinen Rückfall auf alte Ergebnisse und keine Rekonstruktion unbekannter alter Versionen.
Vor der Migration angelegte Vorgänge und Sitzungen ohne Bindung bleiben dadurch unbrauchbar. Eine Neuanmeldung ist erforderlich.

## Getrennte Versionsregeln

Die Mitgliedschaftsversion bleibt für den Vorgang und die gesamte daraus entstandene Sitzungsfamilie unverändert.
Jede Abweichung verlangt einen neuen Anmeldevorgang. Damit kann eine zwischenzeitlich aufgehobene und erneut angelegte Zuordnung keine alte Sitzung wieder erlauben.
Dies konkretisiert den bestehenden Mitgliedschaftsentzug. Es erteilt keine neue organisatorische Berechtigung.
Einstellungs- und Rollenversion dürfen nur steigen. Eine rückläufige Version verweigert die Prüfung.
Bei gleicher Einstellungsversion müssen die drei rohen Fristen gleich bleiben.
Bei gleicher Rollenversion muss die rohe rollenbezogene MFA-Anforderung gleich bleiben.
Eine bereits beobachtete kürzere Frist oder MFA-Pflicht wird weiterhin durch die vorhandene Sitzungsbeobachtung erhalten.
Die rohe rollenbezogene Anforderung bleibt davon getrennt. Eine Zusatzprüfung darf eine stärkere Sitzungsanforderung übernehmen,
ohne damit eine Änderung derselben Rollenversion zu behaupten.

## Speicher und Transaktionen

Migration 0011 ergänzt eigene Versionsbindungen für Anmeldevorgänge und aktuelle Quellenbeobachtungen für Sitzungen.
Die Vorgangsbindung referenziert die bestehende Nachweisableitung. Sie ist unveränderlich und verschwindet bei zulässiger Vorgangslöschung mit dem Vorgang.
Die Sitzungsbeobachtung referenziert die vorhandene Sitzungsbindung. Sie enthält die drei Versionen, rohe Fristen und rohe Rollen-MFA.
Nachträgliches Abschwächen oder Widersprüche derselben Version sind auch durch Datenbanktrigger verboten.
Aufgeschobene Prüfschritte verlangen bei neuen Vorgängen und Sitzungsbindungen den zugehörigen Versionsbestand spätestens beim Commit.
Es gibt keine erlaubende Laufzeitpolicy. Neue Tabellen erhalten ENABLE und FORCE RLS sowie ausschließlich notwendige Tabellenrechte.
Fehlende Sichtbarkeit, fehlende Migration oder ein Datenbankfehler verweigern die Prüfung.

Vorgang und Versionsbindung werden in derselben bestehenden Identitätstransaktion angelegt.
Abschluss vergleicht ursprüngliche und neue Quellen und schreibt Sitzung und Quellenbeobachtung vor Ausgabe des Nachweises atomar.
Die laufende Sitzungsprüfung vergleicht aktuellen Bestand und dauerhaft beobachtete Versionen vor einem positiven Ergebnis.
Rotation kopiert die Quellenbeobachtung innerhalb desselben privaten Savepoints wie die übrige Sitzungsbindung.
Zusatzprüfungen erhalten die vollständigen Quellen und vergleichen sie erneut vor Abschluss.
Abgewarteter Rollback, Ablaufbehandlung, gemeinsame Identitätssperre und bestehende Faktorregeln bleiben erhalten.
Die Quellen nehmen keine fremde Transaktion an und rufen keine schreibende Identitätsoperation zurück.
Eine fremde Mitgliedschaft bleibt ein eigener aktueller Datenstand; es gibt keine dienstübergreifende Atomaritätszusage.

## Nachweise und Grenzen

Reine Tests prüfen Versionsregeln und vollständige Kontexte. Tatsächliche SQLx-Tests prüfen atomare Bindung, Entzug,
Versionsrückschritt, Widerspruch, Rotation, fehlenden Bestand, Rollback, Konkurrenz, geschlossene RLS und Neustart.
Synthetische Testdoppel des vertrauenswürdigen Ports werden ausdrücklich von tatsächlichen mTLS-Quellenprüfungen getrennt.
Die bestehende tatsächliche Quellenkette prüft außerdem die vollständige Übergabe in den Anmeldeport.
Gateway, Browser, produktive Vergabe und ursprüngliche Gesamtabnahmen bleiben offen.
