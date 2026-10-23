# Interner Vertrag: Passwortprüfung v1

Stand: 20.10.2026. KC-006C / #64. Dieser Vertrag geht der Implementierung voraus.
Grundlage: Spezifikation v5.9, ADR-019, ADR-029 und ADR-030.

## Umfang und Ergebnis

Der Dienst prüft ein lokales Passwort. Ein positives Ergebnis belegt nur diesen Faktor.
Es erzeugt keine Sitzung und gewährt keine Berechtigung. MFA-Zuordnung und Wiederherstellung bleiben offen.
Es gibt keine HTTP-Schnittstelle und keine produktive Kontenregistrierung oder Passwortzurücksetzung.
Die interne Speicherpflege setzt eine bereits geprüfte Zuständigkeit voraus.

Passwörter werden als unveränderte Bytefolge mit 1 bis 1.024 Bytes verarbeitet.
Es gibt keine Kürzung oder Normalisierung. Diese technische Obergrenze legt keine fachliche Passwortregel fest.
Passwortpuffer und Hashdarstellungen erhalten keine ausgebende Debug- oder Display-Implementierung.
Fehler enthalten weder Kontenkennungen noch Datenbankdetails oder Nachweise.

## Kryptografisches Profil

Die Implementierung verwendet RustCrypto argon2 0.6.0, Algorithmus Argon2id, Version 19.
Neue Hashes verwenden 65.536 KiB Speicher, drei Durchläufe und einen Parallelitätsgrad von eins.
Salz: 16 zufällige Bytes aus der Betriebssystemquelle. Ausgabe: 32 Bytes.
Als einziges vorheriges Profil werden 19.456 KiB, zwei Durchläufe und Parallelitätsgrad eins akzeptiert.
Dies ist ein technischer Aktualisierungspfad; vorhandene produktive Altbestände werden nicht unterstellt.

Vor jeder Hashprüfung werden Algorithmus, Version, Parameter, Salz- und Ausgabelänge begrenzt.
Nicht freigegebene Parameter dürfen keine teure Berechnung auslösen.
Jede zugelassene Anfrage führt beide begrenzten Profile aus.
Ein unbekannter oder ungültiger Bestand verwendet stattdessen zufällig erzeugte Vergleichshashes.
Dies vermindert Unterschiede zwischen negativen Fällen. Es ist keine Zusage identischer Netzwerklaufzeiten.
Ein gültiges altes Profil wird nur nach erfolgreicher Prüfung auf das neue Profil aktualisiert.
Ein konkurrierend geänderter Bestand darf kein positives Ergebnis aus einem alten Stand erzeugen.

Die Hasharbeit läuft außerhalb des asynchronen Ausführungsfadens.
Eine prozessweite Begrenzung lässt höchstens eine laufende Hashoperation zu; weitere Aufrufe werden abgewiesen.
Ein abgebrochener Aufruf darf die Begrenzung nicht vor Ende der laufenden Hasharbeit freigeben.
Das Profil muss vor produktiver Nutzung auf der bestätigten Referenzhardware gemessen werden.
Es ersetzt keine bestätigten Lastmessungen und wird bei Überlast nicht automatisch abgeschwächt.

## Dauerhafte Versuchslimits

Der Aufrufer übergibt positive Grenzen für Zeitfenster, Versuche je Prüfschlüssel,
Gesamtversuche und gleichzeitig gespeicherte Prüfschlüssel. Es gibt keine produktiven Vorgabewerte.
Ein Prüfschlüssel ist eine dienstintern gebildete Kennung mit 32 Bytes.
Clientwerte dürfen ihn nicht ungeprüft bestimmen. E-Mail-Adressen und IP-Adressen werden nicht gespeichert.
Die spätere Gateway-Anbindung muss dessen vertrauenswürdige Bildung gesondert festlegen.

Ein PostgreSQL-Bestand reserviert jeden Versuch atomar vor der teuren Passwortprüfung.
Dies gilt auch für unbekannte Konten und ungültige Hashes.
Eine globale Zeilensperre begrenzt konkurrierende Reservierungen über mehrere Verbindungen.
Ein verbrauchter Versuch bleibt auch bei Fehler oder Abbruch verbraucht.
Erfolg setzt keine Zähler zurück. Es gibt keine dauerhafte Kontosperre.
Aktive Schlüssel sind mengenbegrenzt; abgelaufene Einträge werden bei Reservierung entfernt.
Eine abweichende Grenzkonfiguration oder rückläufige Datenbankzeit sperrt die Reservierung.
Die gespeicherte Konfiguration wird nicht durch einen Anmeldeaufruf überschrieben.
Ein Dienstneustart darf Zähler nicht zurücksetzen.

Passwortbestand und Versuchszähler liegen im getrennten Plattform-Schema.
Erzwungene RLS bleibt ohne erlaubende Laufzeit-Policy bestehen.
Positive Integrationstests verwenden nur den isolierten Test-Betriebszugang.
Alle SQL-Werte werden gebunden. Der Adapter verändert keine Rollen oder Rechte.
Ein Aufruf benötigt eine Verbindung ohne offene Transaktion, damit die Reservierung vor Hasharbeit feststeht.
Rohe SQL-Transaktionssteuerung durch den Aufrufer ist wie bei ADR-030 unzulässig.

## Nachweise und Abnahmegrenze

Tests prüfen richtige und falsche Passwörter, unbekannte Konten, ungültige Formate,
überhöhte Parameter, Aktualisierung und konkurrierende Bestandsänderung.
Echte PostgreSQL-Tests prüfen atomare Limits, Ablauf, Konfigurationsabweichung,
Neustart, Mengenbegrenzung, Rücknahme einer Migration und die RLS-Sperre.
Es werden ausschließlich synthetische Konten und zufällige Nachweise im Arbeitsspeicher verwendet.
Die Gesamtabnahme von KC-006 bleibt von KC-006D bis KC-006F und den offenen Entscheidungen abhängig.
