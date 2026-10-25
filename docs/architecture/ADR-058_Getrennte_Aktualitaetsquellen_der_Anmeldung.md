# ADR-058: Getrennte Aktualitätsquellen der Anmeldung

- Datum: 25.10.2026
- Status: Technische Integrationsentscheidung vor Code
- Umfang: KC-006F, vorbereitete Verbindung zu KC-008A

## Entscheidung

Die [zusammengesetzte Anmeldegrundlage](../../contracts/identity-access/current-login-source-v1.md) verbindet vorhandene geprüfte Quellen.
Jeder Aufruf fordert einen neuen zweckgebundenen Diensttoken über tatsächliches mTLS an.
Die aktuelle Mitgliedschaft stammt ausschließlich aus dem Organisationsdienst. Der eigene Rollenbestand liefert die MFA-Anforderung.
Der neue interne Ergebnisdatentyp erhält drei getrennte Versionen statt einer künstlichen Gesamtversion.
Eine konservative monotone Tokenfrist und ein äußeres Mengen-/Zeitbudget begrenzen die gesamte Zusammensetzung.

## Begründung

Mitgliedschaft, Einstellungen und Rollen ändern sich unabhängig voneinander.
Addition kann unterschiedliche Versionsstände verwechseln. Ein Hash erhält keine prüfbare monotone Ordnung.
Eine automatische Umwandlung in den bisherigen einversionigen Anmeldekontext würde Aktualitätsnachweise verlieren.
Die Verbindung erhält deshalb zuerst sämtliche Quellen und Versionen. Dauerhafte Anmeldebindung folgt als eigener prüfbarer Schritt.

## Folgen und Grenzen

Der neue Leser hat keine schreibende Rückkopplung und übernimmt keine fremden Transaktionen.
Es entsteht keine dienstübergreifende Transaktion. Jede geschützte Folgeaktion muss erneut aktuell prüfen.
Die bereits bestätigten MFA-Regeln, Rechte und Fristen bleiben unverändert.
KC-006F bleibt bis zu getrennter Versionsbindung, Gateway-, Browser- und ursprünglichen Gesamtnachweisen offen.
Die gemeinsame reine OAuth-Antwortprüfung wird zur normalen Abhängigkeit des Identitätsdienstes.
Es wird keine externe Paketversion geändert.
