# Durchsicht der offenen Pull Requests in P1

Stand: 21.10.2026. Grundlage: Spezifikation v5.9 und Auftrag zur Durchsicht aller offenen PRs.
P0 ist abgenommen. P1 ist bereits beauftragt. Dieser Bericht bereitet die gesammelte persönliche und sprachliche Durchsicht vor.
Die Projektleitung wünscht diese Durchsicht erst nach Bearbeitung aller offenen PRs. Es erfolgt kein Merge.

## Umfang und Verfahren

Zu Beginn sind 17 PRs offen: vier Aktualisierungen von Abhängigkeiten und 13 aufeinander aufbauende P1-PRs.
Geprüft wurden Änderungen, betroffene interne Verträge und Migrationen, Testaufrufe, Commit-Reihenfolge, CI sowie vorhandene Reviews und Kommentare.
Zum Erfassungszeitpunkt gibt es keine GitHub-Reviews oder zeilenbezogenen Review-Kommentare in diesen PRs.
Die [Momentaufnahme](ai-memory/PR_REVIEW_STATE.json) bindet jede Empfehlung an einen Kopf, seine Basis und die gelesenen Prüfergebnisse.
Ein grüner Lauf auf einer früheren Basis ersetzt keine Prüfung nach einer späteren Integration.
Dies ist eine technische KI-Vorprüfung. Sie ersetzt keine persönliche Prüfung, juristische Prüfung oder unabhängige Wahlprüfung.

## Ergebnis je PR

| PR | Geprüfter Kopf | CI bei Erfassung | Empfehlung und Grenze |
|---|---|---|---|
| [#45](https://github.com/itsh-neumeier/kitaconnect/pull/45) | 87c8a17 | 5/5 grün | Nach aktualisierter Basis und grüner CI übernehmen. Feste Commit-ID bleibt erhalten; persist-credentials bleibt false. Nach P1-Integration alle Checkout-Verwendungen prüfen. |
| [#46](https://github.com/itsh-neumeier/kitaconnect/pull/46) | 356edaa | 5/5 grün | Nach aktualisierter Basis und grüner CI übernehmen. Feste Commit-ID und .node-version bleiben erhalten. PR #54 ergänzt eine weitere Setup-Node-Verwendung; beim späteren Zusammenführen ebenfalls aktualisieren. |
| [#47](https://github.com/itsh-neumeier/kitaconnect/pull/47) | 8e1a6cb | 4/5 grün | Nicht integrieren; TypeScript 7 gesondert migrieren. Web-CI scheitert in SvelteKit load_tsconfig an readFile. Kein Überspringen der Prüfung und keine Abschwächung des Tests. |
| [#48](https://github.com/itsh-neumeier/kitaconnect/pull/48) | f9ec1a0 | 5/5 grün | Nicht integrieren; Node-24-Typen beibehalten. Node.js ist durch .node-version und engines auf 24 festgelegt. Node-26-Typen können APIs zulassen, die diese Laufzeit nicht bereitstellt. |
| [#53](https://github.com/itsh-neumeier/kitaconnect/pull/53) | 5d42ef8 | 10/10 grün | Als begrenztes Modell übernehmen. Typtrennung, Nullgrenze, Duplikate und fehlgeschlagene Anlage geprüft. Keine Organisationsrechte oder vollständige KC-004-Abnahme. |
| [#54](https://github.com/itsh-neumeier/kitaconnect/pull/54) | 16cd295 | 10/10 grün | Als geschlossene Persistenzgrundlage übernehmen. Rollen, numerische Grenzen, Migration und erzwungene RLS geprüft. Keine erlaubende Laufzeit-Policy. |
| [#55](https://github.com/itsh-neumeier/kitaconnect/pull/55) | a8db9ca | 10/10 grün | Mit Korrektur der Schreibbestätigung übernehmen. Unterdrückte INSERT-Anweisungen dürfen keinen Erfolg melden. Korrektur und Testnachweis liegen auf dem gesonderten Durchsichtsbranch. |
| [#59](https://github.com/itsh-neumeier/kitaconnect/pull/59) | 87cebbb | 10/10 grün | Als leere Schema-Provisionierung übernehmen. Kanonische Namen, Wartungsrechte, Konkurrenz und Rücknahme geprüft. Keine Fachtabellen oder produktive Laufzeitidentität. |
| [#60](https://github.com/itsh-neumeier/kitaconnect/pull/60) | 3dcf78f | 10/10 grün | Als Testnachweis übernehmen. Getrennte Dienst-/Einrichtungsproben, RLS und Namensverdeckung geprüft. Synthetische Loginrollen sind keine produktive Rollenentscheidung. |
| [#61](https://github.com/itsh-neumeier/kitaconnect/pull/61) | 4fa55b0 | 10/10 grün | Als Testnachweis übernehmen. Gleicher Datenbankprozess nach Poolrückgabe, Widerruf und reale Wartungskonkurrenz geprüft. Keine allgemeine produktive Migrationssteuerung. |
| [#68](https://github.com/itsh-neumeier/kitaconnect/pull/68) | c7711e8 | 10/10 grün | Als internes Sitzungsmodell übernehmen. Typgrenzen, Zeitüberlauf, rückläufige Zeit und endgültige Zustände geprüft. Modellzustand erteilt keine Berechtigung. |
| [#69](https://github.com/itsh-neumeier/kitaconnect/pull/69) | e0e1214 | 10/10 grün | Mit Korrektur der Schreibbestätigung als internen Sitzungsspeicher übernehmen. Identitäts- und Sitzungsanlage müssen genau eine Zeile bestätigen. create_session bleibt interner Speicherzugang; spätere Browseranmeldung muss login::finish verwenden. |
| [#70](https://github.com/itsh-neumeier/kitaconnect/pull/70) | e003eb0 | 10/10 grün | Als begrenzten Passwortfaktor übernehmen. Argon2id-Profile, blockierende Arbeit, dauerhaft reservierte Versuche und Bestandswechsel geprüft. Produktive Limits und Referenzmessungen fehlen. |
| [#71](https://github.com/itsh-neumeier/kitaconnect/pull/71) | 74c5e17 | 10/10 grün | Nur im bestätigten localhost-Testumfang übernehmen. Bibliotheksprüfung, Benutzerverifikation, Einmalverbrauch und Bestandsbindung geprüft. Kein produktiver Ursprung, Browser-/TLS- oder vollständiger Anmeldenachweis. |
| [#72](https://github.com/itsh-neumeier/kitaconnect/pull/72) | cb3d8f6 | 10/10 grün | Nur im dokumentierten lokalen MFA-Umfang übernehmen. TOTP, Nonce-Zähler, einmalige Wiederherstellung, Sitzungswiderruf und atomare Ereignisaufträge geprüft. RAM-Schlüssel und Testkontakte sind keine produktive Schlüssel-/Kontaktverwaltung. |
| [#73](https://github.com/itsh-neumeier/kitaconnect/pull/73) | f5f0586 | 10/10 grün | Als Einrichtungseinstellungsspeicher übernehmen. Wertebereiche, erwartete Version, reale Schreibkonkurrenz, Commitfehler und RLS geprüft. Keine Fristdurchsetzung für Browseranfragen. |
| [#74](https://github.com/itsh-neumeier/kitaconnect/pull/74) | 58ea681 | 10/10 grün | Als lokalen Anmeldeabschluss übernehmen. Echte Faktoren, Einmalverbrauch, Identitätssperre, Wiederherstellungs-Konkurrenz und dauerhafter Ablauf geprüft. SyntheticOrganization ersetzt keinen echten Mitgliedschafts-/mTLS-Adapter. |

## Konkrete Korrektur und offene Aktualisierungen

Der Organisationsadapter aus #55 meldet bislang auch bei null betroffenen INSERT-Zeilen Erfolg.
Ein synthetischer BEFORE-INSERT-Trigger mit RETURN NULL kann diesen Zustand erzeugen.
Der Folgebranch präzisiert den Vertrag vor Code. Beide Anlageoperationen verlangen anschließend genau eine betroffene Zeile.
Ein echter PostgreSQL-Test prüft unterdrückte Träger- und Einrichtungsanlage vor und nach Neustart.
Die Korrektur liegt getrennt über #74. Sie verändert keine bestehende Migration, Rollenregel oder fachliche Anforderung.
Die neue native CI muss den Korrekturstand bestätigen; Abschlussnachweise stehen im zugehörigen PR.

Die gleiche fehlende Bestätigung betrifft Identitäts- und Sitzungsanlage in #69.
Der Sitzungsvertrag wurde in 8816a18 vor der zweiten Korrektur präzisiert.
Auch diese beiden Anlagen verlangen nun genau eine betroffene Zeile. Die Sitzungsanlage gibt bei einer unterdrückten Anlage keinen Nachweis aus.
Der echte Sitzungs-Datenbanktest prüft beide Fehler vor und nach Neustart. Neue oder geänderte Migrationen sind dafür nicht erforderlich.

#47 bleibt wegen des belegten Webfehlers von der Integration ausgeschlossen.
Eine Umstellung auf TypeScript 7 braucht eine gemeinsam kompatible SvelteKit-/Prüfwerkzeugkette und eigene positive Nachweise.
#48 soll nicht mit Node 24 integriert werden. Für spätere Aktualisierungen sind Typen innerhalb der Laufzeit-Hauptversion zu wählen.
Beide PRs bleiben bis zur gesammelten Durchsicht offen; eine Ablehnung ist eine begründete Empfehlung, kein verschwiegener Abschluss.
Die Aktualisierungen #45/#46 benötigen eine aktuelle Main-Basis. Beim späteren Zusammenführen sind alle fünf Checkout- und vier Setup-Node-Verwendungen des P1-Stands abzugleichen.

Diese Main-Aktualisierung ist inzwischen ohne Force-Push erfolgt. #45 steht auf 1712c7a und #46 auf 8ccef99.
Alle jeweils fünf Prüfungen sind am neuen Kopf erfolgreich. Die Diffs enthalten weiterhin ausschließlich die vorgesehenen Action-Pins.
[CI #45](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39600343646),
[Sicherheitsprüfungen #45](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39600343744),
[CI #46](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39600356339),
[Sicherheitsprüfungen #46](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39600356345).
Die Momentaufnahme bewahrt den ursprünglichen Stand und ergänzt diese Folgeprüfung getrennt.
Alle 17 ursprünglichen PR-Beschreibungen enthalten jetzt die jeweilige technische Empfehlung und den Verweis auf [Korrektur-PR #75](https://github.com/itsh-neumeier/kitaconnect/pull/75).

## Empfohlene Integrationsfolge nach gesondertem Auftrag

1. Persönliche und sprachliche Durchsicht gesammelt auf Grundlage dieses Berichts und des Korrektur-PRs durchführen.
2. P1-Kette in der Reihenfolge #53 → #54 → #55 → #59 → #60 → #61 → #68 → #69 → #70 → #71 → #72 → #73 → #74 übernehmen.
   Den Korrektur-PR unmittelbar danach übernehmen. #55 und #69 bleiben bis zur Korrektur ohne produktive Freigabe.
3. Nach jeder Integration den nächsten PR auf main ausrichten. Aktuelle Basis, unveränderten Umfang und alle fünf Pflichtprüfungen erneut prüfen.
   Bei Konflikten oder fehlgeschlagenen Prüfungen anhalten und den Fehler beheben. Keine Prüfungen umgehen.
4. #45 und #46 auf dem vollständigen P1-Stand abgleichen und mit frischer CI gesondert übernehmen. Neue Workflow-Verwendungen dürfen keine alte Action-Version behalten.
5. #47 und #48 nach Durchsicht begründet schließen oder mit gesondertem kompatiblem Migrationsauftrag fortsetzen.

Diese Folge ist eine Vorbereitung und keine Merge-Erlaubnis. Die Review-Ausnahme gilt bei Dependabot-PRs nach ADR-022 nicht als Ausnahme für eigene PRs.
Ein tatsächlicher fremder PR benötigt das dafür geltende Review.

## Fortsetzung von P1

KC-004, KC-005 und KC-006 bleiben im jeweiligen Gesamtumfang offen.
Die geprüften Bausteine implementieren noch keine produktive Browseranmeldung oder Fachberechtigung.
Nächstes Entwicklungspaket bleibt #67: aktuelle Sitzungsprüfung, Rotation und Abmeldung; Vertrag vor Code versionieren.
Danach folgen der echte Organisations-/Mitgliedschaftsadapter, Gateway, Cookies/CSRF und getrennte Browser-/TLS-/mTLS-Nachweise.
Produktive Domain, Schlüsselbetrieb, Versuchslimits, konkrete Verwaltungsrechte und Rollen-/Poolbindung behalten ihre dokumentierten Sperren.

## Quellen für Aktualisierungsgrenzen

- [Checkout v7.0.1: Action-Vertrag](https://github.com/actions/checkout/blob/v7.0.1/action.yml).
- [Setup Node v7.1.0: Action-Vertrag](https://github.com/actions/setup-node/blob/v7.1.0/action.yml).
- [DefinitelyTyped: Zuordnung von Paket- und Typversionen](https://github.com/DefinitelyTyped/DefinitelyTyped/blob/master/README.md).
- [Belegter Webfehler von #47](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39142343237/job/117719142359).
