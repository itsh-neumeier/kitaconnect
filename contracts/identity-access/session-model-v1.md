# Identitäts- und Sitzungsmodell v1

Stand: 20.10.2026. Teilpaket: KC-006A / #62. Grundlage: Spezifikation v5.9 und ADR-019.

## Umfang

`services/identity-access` enthält zunächst eine interne Rust-Bibliothek ohne Netzwerk oder Speicherung.
Das Modell beschreibt nur Kennungen, Zeitgrenzen und den Lebenszyklus eines Sitzungsdatensatzes.
Es prüft keine Passwörter, Passkeys, TOTP oder Wiederherstellungscodes.
Es stellt weder Browsernachweise noch Diensttokens aus. Ein Modellwert erteilt keine Anmeldung oder Fachberechtigung.

`IdentityId` und `SessionRecordId` sind verschiedene Typen mit jeweils einer u128-Kennung ungleich null.
Diese Kennungen sind interne Datensatzschlüssel, keine zufälligen Geheimnisse oder vom Browser akzeptierten Sitzungsnachweise.
E-Mail-Adresse, Personenname und Einrichtungsrechte werden nicht abgebildet.
Es gibt keinen impliziten Zusammenhang zwischen Identitätskennung und Organisationskennung.

## Eingaben und Zeit

Der Aufrufer übergibt Kennungen, Erstellungszeit und ausdrücklich gewählte positive Zeitgrenzen.
Zeitpunkte sind serverseitige ganze Sekunden seit der Unix-Epoche. Zeitgrenzen sind ganze Sekunden.
Das Modell liest keine Systemuhr. Clientzeit und Clientvorgaben sind keine zulässige Autorität für spätere Laufzeitaufrufe.
Es gibt keine Vorgabewerte für Sitzungsdauer, MFA oder Wiederherstellung.
Die offenen Interviewvorschläge werden nicht durch Beispielwerte oder Tests bestätigt.

Die absolute Ablaufzeit entsteht durch geprüfte Addition von Erstellungszeit und absoluter Laufzeit.
Nullwerte für Zeitgrenzen und ein Überlauf bei der Anlage werden abgewiesen.
Die Inaktivitätsfrist beginnt bei der letzten ausdrücklich erfassten berechtigten Aktivität.
Die wirksame Ablaufzeit ist stets das frühere Ende aus Inaktivitätsfrist und absoluter Ablaufzeit.
Eine Inaktivitätsfrist über das absolute Ende hinaus erweitert die Sitzung nicht.

## Zustände und Operationen

`SessionState` enthält `Active`, `Expired` und `Revoked`.
Diese Zustände sind lokale Modellzustände und keine aktuelle Authentisierungsentscheidung.

- `SessionRecord::new` erzeugt einen internen Datensatz mit den übergebenen Grenzen.
- `observe(now)` prüft Zeit und Ablauf. Am exakten Fristende gilt die Sitzung bereits als abgelaufen.
  Reines Beobachten verlängert die Inaktivitätsfrist nicht.
- `record_activity(now)` prüft zuerst Zeit und Zustand. Nur ein aktiver Datensatz kann eine neue Aktivitätszeit erhalten.
  Der spätere Aufrufer darf diese Methode nur nach erfolgreicher aktueller Prüfung einer berechtigten Aktivität verwenden.
  Sie verändert weder Erstellungszeit noch absolute Ablaufzeit.
- `revoke()` sperrt den Datensatz dauerhaft. Wiederholung ist zulässig.
  Keine Methode aktiviert einen widerrufenen oder beobachtet abgelaufenen Datensatz erneut.

Zeitpunkte vor der zuletzt beobachteten Zeit werden mit `ClockRegression` abgewiesen.
Sie verändern den Datensatz nicht. Der Aufrufer muss den Fehler als verweigerte Prüfung behandeln.
Aktivität nach Ablauf oder Widerruf wird mit `InactiveSession` abgewiesen und verlängert keine Frist.
Das Modell hat keine Funktion zum Zurücksetzen des Zustands oder Verlängern der absoluten Frist.

## Fehler und spätere Integration

`SessionError` unterscheidet ungültige Grenzen, Überlauf bei Anlage, rückläufige Zeit und inaktive Sitzung.
Fehler enthalten keine Zugangsnachweise oder personenbezogenen Inhalte.
Ein zwischengespeicherter Datensatz ist kein Nachweis des aktuellen serverseitigen Zustands.
KC-006B muss Speicherung, aktuelle Abfrage und atomare Änderungen gegen konkurrierende Vorgänge absichern.
KC-006F integriert aktuelle Identitäts-/Sitzungsprüfung, Mitgliedschaft und Gateway.
KC-008 prüft danach konkrete Berechtigungen. Dieses Modell ersetzt keine dieser Prüfungen.

## Nachweise

Rust-Tests prüfen exakte Fristgrenzen, Beobachtung ohne Verlängerung, begrenzte Verlängerung durch Aktivität,
dauerhaften Widerruf, dauerhaft beobachteten Ablauf, rückläufige Zeit und den u64-Grenzbereich.
Ein negativer Dokumentationstest verhindert die Verwechslung beider Kennungstypen.
Diese Tests verwenden synthetische Kennungen und freie Testzeiten, keine vorgeschlagenen Projektvorgabewerte.
