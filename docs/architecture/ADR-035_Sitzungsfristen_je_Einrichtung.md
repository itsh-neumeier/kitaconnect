# ADR-035 – Versionierte Sitzungsfristen je Einrichtung

Datum: 21.10.2026. Status: technische Konkretisierung der bestätigten KC-006F-Antworten vor Code.
Grundlage: Spezifikation v5.9 und ADR-014/015/034.

## Entscheidung

Die Projektleitung verlangt einstellbare Fristen pro Einrichtung und bestätigt anschließend ausdrücklich die vorgeschlagenen Voreinstellungen und Obergrenzen.
Es gelten höchstens 900 Sekunden Inaktivität, 28.800 Sekunden Gesamtlaufzeit und 300 Sekunden für eine zusätzliche Authentisierung.
Eine Einrichtung darf kürzere Fristen wählen. Fehlende Konfiguration darf keine stille Freigabe erzeugen.

Die Einrichtungseinstellungen gehören gemäß Dienstmatrix zu organization-config.
Der [Speichervertrag](../../contracts/organization-config/session-policy-v1.md) definiert einen aktuellen Satz mit atomarem Versionsvergleich.
Konkurrierende Änderungen dürfen einander nicht unbemerkt überschreiben.
Eine neue Migration ergänzt den dienstinternen Bestand mit RLS-Sperre; sie verändert keine alte Migration und keine Sitzung.
Eine Verwaltungs-API und deren Rechte werden in diesem Schritt nicht eingeführt.

## Wirkung und Grenzen

Der bestätigte Voreinstellungssatz ist ausdrücklich wählbar. Der Leseweg legt keinen Bestand an.
Ein neuer Wert muss positiv sein und innerhalb der bestätigten Obergrenze liegen.
Inaktivität und Zusatzprüfung dürfen die Gesamtlaufzeit nicht überschreiten; dies verhindert widersprüchliche Fristkonfiguration.
Der nächste Integrationsschritt muss aktuelle und ursprüngliche Fristen gemeinsam prüfen.
Verschärfung wirkt bei der nächsten geschützten Prüfung. Eine Lockerung verlängert keine bestehenden Nachweise.

Die Versionskennung schützt konkurrierendes Schreiben; sie ersetzt kein vollständiges Änderungsprotokoll und keine Autorisierung.
Positive echte Datenbanktests verwenden ausschließlich den isolierten Betriebszugang. Die produktive Anwendungsrolle bleibt gesperrt.
MFA-Kombination und Cookie-/CSRF-Schutz aus ADR-034 sind bestätigt, werden durch diesen Konfigurationsadapter aber noch nicht implementiert.

## Nachweise

Werteprüfung, konkurrierender Versionsvergleich, Fehler-Rücknahme, Migration, RLS und Neustart werden gesondert getestet.
Ein bestandener Konfigurationstest erfüllt keine Browser-, mTLS- oder vollständige Anmeldeabnahme.
