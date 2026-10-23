# KC-005: Technischer Prüfstand und Abnahmegrenzen

Stand: 20.10.2026. Grundlage: Spezifikation v5.9, ADR-014 und P1-Auftrag.
Diese Unterlage dokumentiert Teilnachweise auf vorbereiteten Branches. Sie erklärt keine produktive Gesamtabnahme.

## Teilpakete und Nachweise

| Teilpaket | Vorbereiteter PR | Nachweis und Grenze |
|---|---|---|
| KC-005A / #56 | [PR #59](https://github.com/itsh-neumeier/kitaconnect/pull/59) | Atomare leere Schemata und Registry; aktuelle Rollen-/Eigentümerprüfung. Keine Fachobjekte. |
| KC-005B / #57 | [PR #60](https://github.com/itsh-neumeier/kitaconnect/pull/60) | Dienstrechte und erzwungene RLS für zwei technische Dienste und Einrichtungen. Test-Loginbindung, keine produktive Identitätsentscheidung. |
| KC-005C / #58 | [PR #61](https://github.com/itsh-neumeier/kitaconnect/pull/61) | Poolrückgabe nach Erfolg/Fehler/Abbruch, Rechteentzug, beobachtete Wartungskonkurrenz und fehlerhafte Testwelle. |

Die [Verträge](../contracts/README.md) wurden jeweils vor der Implementierung versioniert.
Die verbindlichen Sicherheitsgrenzen stehen in ADR-026, ADR-027 und ADR-028.
Alle Testdaten sind synthetisch. Keine produktiven Zugänge oder neue Laufzeitfreigaben wurden angelegt.

## Ausgeführte Prüfungen

Implementierung e4c1e000d8e2b5ddf554e6eb52e99c87b27dde94: erster vollständiger CI-Lauf mit fünf erfolgreichen Pflichtprüfungen.
[Build, Clippy und echte PostgreSQL-/SQLx-Tests](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39406137461),
[Secret- und Abhängigkeitsprüfungen](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39406137330).
Beide neuen Datenbanktests sind im Protokoll ausdrücklich vor und nach Neustart erfolgreich.
Der folgende Dokumentationscheckpoint erhält eigene CI. Den Prüfstand des aktuellen Kopfes führt PR #61 nach.
Lokal erfolgreich: 44 Node-Tests, Struktur-/Verweisprüfung, rustfmt, JavaScript-Syntax und Gitleaks.
Docker und Windows-Linker fehlen lokal. Ausführbare Nachweise stammen aus Linux-CI.

## Prüfanforderungen aus KC-005C

1. Prüfen Sie die PostgreSQL-Prozesskennung vor und nach jeder Poolrückgabe.
   Eine neue Verbindung darf die verlangte Wiederverwendung nicht ersetzen.
2. Prüfen Sie Erfolg, SQL-Fehler, Abbruch einer Rust-Operation und PostgreSQL-Anweisungszeitlimit.
   Der nächste Zugriff darf keinen alten Einrichtungskontext oder abgebrochene Schreiboperation vorfinden.
3. Prüfen Sie Rechte- und Policyentzug auf einer geöffneten Verbindung.
   Eine unbekannte oder gelöschte Registryzuordnung darf keinen Wartungstreffer liefern.
4. Beobachten Sie zwei wartende PostgreSQL-Sperranforderungen vor Freigabe der Wartungssperre.
   Prüfen Sie doppelte Provisionierung, doppelte Testmigration und deren gemischten Ablauf.
5. Prüfen Sie die fehlgeschlagene Testwelle und den gezielten erneuten Versuch.
   Frühere bestätigte Änderungen bleiben erhalten. Die nächste Einrichtung bleibt nach dem Fehler unbearbeitet.
6. Prüfen Sie die bestätigten Registrierungen und Versionsstände nach einem Server-Neustart erneut.

Die Testmigrationshilfe verwendet nur zwei festgelegte technische Versionsschritte.
Sie prüft das Transaktions- und Sperrverfahren. Sie ist keine produktive Migrationssteuerung oder Wiederherstellung.

## Offene Gesamtabnahme

KC-005 / #5 bleibt offen. Die folgenden Grenzen werden durch technische Testrollen nicht aufgehoben:

- Die produktive Bindung von Dienst, Einrichtung und Verbindungspool ist noch nicht entschieden.
  Der Vorschlag eigener Identitäten und Pools je Dienst und Einrichtung liegt der Projektleitung vor.
  Ohne Antwort gilt keine Freigabe. Frei gesetzte Sitzungswerte bleiben als alleinige Identität ausgeschlossen.
- Ein erlaubter produktiver Einrichtungswechsel ist noch nicht implementiert oder nachgewiesen.
  Die Testzugänge aus KC-005B/C verweigern unerlaubte Rollen- und Einrichtungswechsel.
- Aktuelle Sitzungs-, Mitgliedschafts-, Objekt- und Feldrechte folgen in KC-006/KC-008.
  Technischer Datenbankrechteentzug ersetzt keinen vollständigen Benutzerrechtewiderruf.
- Eine produktive Migrationssteuerung benötigt Artefaktfreigabe, Zustandsprotokoll und geprüfte Fehlerbehandlung.
  Zwei fest eingebundene Testschritte decken diesen Betriebsumfang nicht vollständig ab.
- Datensicherung und Wiederherstellung benötigen gesonderte Nachweise.
  Die Rücknahme einer offenen Transaktion ist keine Wiederherstellung verlorener bestätigter Daten.
- Die vorbereiteten PRs benötigen persönliche/sprachliche Durchsicht und den gesonderten Integrationsauftrag.
  Nach jeder Basisintegration sind Zielbranch und CI erneut zu prüfen.

Die nächste unabhängige P1-Vorbereitung kann den Vertrag und die kleinen Arbeitspakete von KC-006 / #6 behandeln.
Produktive RLS-Freigaben bleiben bis zur ausdrücklichen Identitätsentscheidung gesperrt.
