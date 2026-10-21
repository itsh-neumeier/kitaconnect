# ADR-036 – Einmalige Faktorübergabe und lokaler Sitzungsabschluss

Datum: 21.10.2026. Status: technische Konkretisierung vor dem nächsten KC-006F-Code.
Grundlage: Spezifikation v5.9, bestätigte MFA-/Browserregeln und ADR-019/034/035.

## Entscheidung

Der [Speichervertrag](../../contracts/identity-access/login-completion-v1.md) ergänzt vorgangsgebundene Varianten der bestehenden Faktorprüfungen.
Sie merken den positiven Nachweis in derselben Faktortransaktion vor. Alte positive Ergebnisobjekte bleiben ungeeignet für eine neue Sitzungsfreigabe.
Ein einmaliger, zweck- und einrichtungsgebundener Vorgang wird beim lokalen Sitzungsabschluss atomar verbraucht.
Eine Bestandsableitung erkennt zwischenzeitlichen Faktorwechsel. Sie enthält auch den PHC-Bestand und schützt dadurch gegen Änderungen ohne Versionsfortschritt.

Alle beteiligten Faktoradapter verwenden eine gemeinsame transaktionsgebundene PostgreSQL-Advisory-Sperre je Identität.
Der 64-Bit-Sperrschlüssel wird mit SHA-256 aus festem Namensraum und der Identität abgeleitet.
Dies ist eine technische Sperrzuordnung; eine Kollision serialisiert zusätzliche Identitäten, gibt aber keine Freigabe.
Die zweigeteilten Migrations-/Mengenlocks bleiben von diesem PostgreSQL-Schlüsselraum getrennt.
Direkte Credential-Schreibvorgänge ohne den gemeinsamen Sperrvertrag sind nicht zulässig.

Der aktuelle Organisationsvertrag wird über eine interne Adaptergrenze angefordert. Tests verwenden zunächst ein explizites synthetisches Testdoppel.
Die Antwort muss Identität, Einrichtung, Mitgliedschaft, MFA-Anforderung und versionierte Fristen binden.
Fehlende oder fehlerhafte Daten führen zur Verweigerung. Die spätere Netzwerkimplementierung muss ADR-019 erfüllen.
Es gibt keinen direkten Zugriff des Identitätsdienstes auf fremde Organisationstabellen.

## Fristen und Widerruf

Ursprüngliche und aktuelle Einrichtungsfristen werden durch das Minimum verbunden. Eine Lockerung verlängert keine bestehende Vorgangsbindung.
Auch die ursprünglich erforderliche Authentisierungsstärke darf während desselben Vorgangs nicht herabgesetzt werden.
Ein eingerichteter aktiver TOTP-Faktor macht die Passwortanmeldung mehrstufig; der bestätigte Passkey mit Benutzerverifikation bleibt eine zulässige Alternative.
Der Ablauf wird nach einer wartenden Organisationsprüfung erneut geprüft.
Passwortpflege und Passkey-Entzug widerrufen bestehende Sitzungen in derselben Transaktion.

## Abnahmegrenzen

Dieser Schritt enthält lokale Nachweisübergabe, Sitzungserzeugung und gespeicherte Einrichtungsbindung.
Er enthält keine Cookies, Gateway-Anbindung, Sitzungsrotation oder vollständige Prüfung späterer Browseraktionen.
Die Organisations-Testgrenze ist kein mTLS- oder produktiver Mitgliedschaftsnachweis. Der aktuelle Dienstaufruf bleibt für spätere geschützte Aktionen erforderlich.
Produktive Domain, Schlüssel-/Rollenbindung, konkrete Verwaltungsrechte und Betriebsgrenzwerte bleiben gesondert.
Neue Migration 0005 erhält geschlossene RLS; alte Migrationen bleiben unverändert.
Die Gesamtabnahme von KC-006F bleibt bis zur übrigen Integration offen.
