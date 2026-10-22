# Dienstvertrag: Diensttoken-Ausgabemetadaten v1

- Stand: 22.10.2026
- Umfang: KC-006F / #67, interne Voraussetzung für tatsächliche OAuth-Endpunkte
- Status: Vertrag vor Code; kein HTTP-Listener

## Bestätigte Ausgabe

Der Ausgabeadapter erhält einen zusätzlichen Ergebnistyp mit rohem Token, geprüftem Scope, Erzeugungszeit und tatsächlicher Ablaufzeit.
Nur die bestätigte Ausgabe darf diesen Typ erzeugen. Es gibt keinen öffentlichen Konstruktor aus behaupteten Metadaten.
Die tatsächliche Ablaufzeit stammt aus demselben geschützten Ausgabevorgang und berücksichtigt das Zertifikatsende.
Die bisherige interne `issue`-Schnittstelle bleibt als Aufrufer der erweiterten Ausgabe erhalten.
Kein roher Token wird vor erfolgreichem Commit zurückgegeben. Eigene Fehlerpfade warten weiterhin Rollback ab.

Ein eigener asynchroner Prüfschritt berechnet die Restlaufzeit mit aktueller Datenbankzeit.
Er lehnt bereits offene Aufrufertransaktionen ab und erfindet keine produktive Vorgabezeit.
Die Zeit darf weder vor der Erzeugung noch vor der zuletzt beobachteten Ausgabezeit liegen.
Eine verbleibende Laufzeit muss zwischen 1 und 300 Sekunden liegen und wird aus der tatsächlichen Ablaufzeit berechnet.
Ein beobachteter Ablauf oder Zeitfehler sperrt die Ausgabe-Metadatenprüfung dieses Ergebnisses endgültig.
Eine verspätete Prüfung kann deshalb keinen bereits abgelaufenen Token erneut zur Ausgabe freigeben.
Ein Datenbankfehler liefert keine Restlaufzeit. Er erzeugt keine erfolgreiche Antwort.
Eine berechnete Restlaufzeit ist eine Momentaufnahme; der spätere Handler muss sie unmittelbar vor der Antworterzeugung ermitteln.
Er muss daneben aktuelle Verbindung, Registrierung und Freigabe prüfen. Der Zeitprüfschritt ersetzt diese Prüfungen nicht.

## Grobe Fehlerklassen der Dienstanmeldung

Der interne Dienstanmeldeadapter unterscheidet ausschließlich `Rejected` und `Unavailable`.
`Rejected` bezeichnet falschen registrierten Namen, fehlende Zuordnung oder fehlende aktive passende Registrierung.
`Unavailable` bezeichnet Datenbank-/Transaktionsfehler, ungültigen gespeicherten Zustand oder nicht durchführbare aktuelle Transport-/Zeitprüfung.
Ein späterer HTTP-Adapter kann Ablehnung damit als `invalid_client` und Prüfausfall als leere HTTP-503-Antwort abbilden.
Ein Prüfausfall darf nicht als angeblich erfolgreich geprüftes `active=false` ausgegeben werden.
Beide Fehlerklassen geben nur feste Texte aus. Sie enthalten keine SQL-Diagnose, Fingerabdrücke, Formularwerte oder Roh-Tokens.
Die Klassifizierung ist ein internes Ergebnis und keine neue Berechtigung.

## Nachweise und Grenzen

Die bestehende Diensttoken-Datenbankgruppe prüft die erweiterten Metadaten, Zertifikatskürzung,
verbleibende Laufzeit, verzögerte Antwort bis nach Ablauf und endgültige Ablehnung nach beobachtetem Ablauf.
Vorhandene Ausgabe-, Rücknahme-, Konkurrenz- und Neustarttests bleiben erhalten.
Die tatsächliche Dienstanmeldegruppe unterscheidet falsche Registrierung von Datenbankausfall.
Getrennte Fehlerzeugen und geschlossene Laufzeitrolle bleiben verpflichtend.
Tests verwenden ausschließlich die vorhandenen isolierten Datenbanken und flüchtige Tokens.
Keine Migration, Paketversion, Datenbankrechte oder Laufzeitpolicy wird geändert.
Tatsächliche HTTP-Endpunkte, Begrenzung der Verbindungen, Organisation und Browser erhalten eigene Nachweise.
