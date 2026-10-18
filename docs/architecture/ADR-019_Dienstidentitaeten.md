# ADR-019 – Kurzlebige Dienstidentitäten und aktueller Rechtekontext

Datum: 18.10.2026. Status: technische Festlegung im delegierten P0-Umfang von KC-037.
Grundlage: v5.9, ADR-015 und Auftrag, nach den Rückfragen autonom weiterzuarbeiten.
O-003 wird auf Architekturebene konkretisiert. Keine Implementierung in P0.

## Entscheidung

Der Dienst `identity-access` stellt installationsgebundene, nicht selbstbeschreibende Diensttokens aus.
Ein Token gilt höchstens 5 Minuten, für genau einen aufrufenden Dienst und einen Zieldienst.
Die Anwendung verwendet etablierte OAuth-2.0-Verfahren. Sie erfindet kein eigenes Kryptoverfahren.
Für geschützte Dienstaufrufe wird der aktuelle Tokenzustand serverseitig beim Aussteller geprüft.
Die Tokenprüfung folgt dem Introspektionsverfahren aus RFC 7662.
Sie liefert keinen pauschalen Benutzerzugriff.

TLS bezeichnet die verschlüsselte Transportverbindung.
mTLS bezeichnet TLS mit beidseitiger Zertifikatsprüfung.
Die Dienstkommunikation verwendet mTLS mit je Dienst eigener Identität.
Ein Anwendungsdienst darf sich weder als ein anderer Dienst ausgeben noch selbst Zertifikate ausstellen.
Tokenausstellung und Tokenprüfung authentisieren den Dienst ebenfalls über mTLS.
Die Tokenprüfung darf keine weitere Tokenprüfung auslösen und dadurch eine Endlosschleife erzeugen.
Die Bindung an den aufrufenden Dienst wird nach einem geprüften Verfahren gemäß RFC 8705 umgesetzt.

## Benutzerrechte und Widerruf

Der empfangende Dienst prüft zusätzlich aktuelle Sitzung, Mitgliedschaft, Zweck sowie Objekt- und Feldrechte.
Ein Benutzerkontext stammt ausschließlich aus einer geprüften Sitzung oder einem ausdrücklich berechtigten Auftrag.
Ein Client-Header, eine Rollenliste oder eine Mandantenkennung ist allein keine Autorität.
Ein Diensttoken enthält keine vollständigen Kinder-, Gesundheits-, Personal- oder Wahldaten.

Tokenlaufzeit ersetzt keinen sofortigen Rechteentzug.
Widerrufene Dienstidentitäten, Sitzungen oder Mitgliedschaften werden bei der nächsten geschützten Aktion abgewiesen.
Eine alte positive Rechteentscheidung darf deshalb nicht als Freigabe weiterverwendet werden.
Fällt die erforderliche Prüfung aus, wird die geschützte Aktion verweigert.
Dieses Verhalten muss im Last- und Ausfalltest einschließlich des zentralen Identitätsdienstes geprüft werden.

## Schlüssel, Zertifikate und Installation

Jede Installation erhält einen eigenen Vertrauensbereich. Es gibt keine gemeinsamen Projektschlüssel.
Eine gesonderte Betriebsrolle verwaltet Zertifikatsausstellung, Rotation und Widerruf.
Anwendungsdienste erhalten keine Schlüssel der ausstellenden Stelle.
Automatisierte Dienstzertifikate gelten zunächst höchstens 24 Stunden und werden rechtzeitig erneuert.
Ein abgelaufenes Zertifikat darf nicht durch abgeschaltete Prüfung umgangen werden.
Das spätere Betriebsissue MUSS Erneuerung, Zeitabweichung und Wiederanlauf nach längerer Abschaltung prüfen.

Compose stellt Laufzeitschlüssel über geschützte Secret-Dateien bereit.
Kubernetes stellt sie über Secret Stores mit begrenzten Rechten und geprüfter Speicherung bereit.
Das konkrete Ausstellungswerkzeug wird vor Implementierung auf Wartung, Rechte und Lizenz geprüft.
Keine Schlüssel, Tokens, Beispielzertifikate oder produktiven Zugangsdaten werden im Repository erzeugt.

## Abgrenzung und Alternative

Selbstbeschreibende, signierte Tokens mit ausschließlich lokaler Prüfung wurden nicht als Standard gewählt.
Sie würden zusätzlich eine sofort wirksame Widerrufs- und Rechteprüfung benötigen.
Die zentrale Prüfung erleichtert den nachvollziehbaren Rechteentzug, erhöht aber Latenz und Ausfallabhängigkeit.
Diese Folgen gehören zum bestätigten Lastprofil und zu den späteren Verfügbarkeitstests.

Browseranmeldung und Föderation erhalten eigene Dienstverträge in KC-006/KC-007.
Diese Festlegung definiert dafür keine neuen fachlichen Rechte.
Die anonyme Stimmabgabe ist ausdrücklich ausgenommen. Ihre Identitäts- und Transportgrenzen folgen ADR-017.
Eine allgemeine Introspektion darf dort keine Personenspur erzeugen.

## Nachweise vor Implementierungsabnahme

KC-006/KC-008 und KC-032 MÜSSEN falschen Aussteller, falschen Empfänger, fremde Dienstidentität,
abgelaufenes Token, Zertifikatswechsel, Widerruf und Ausfall der Prüfung negativ testen.
Ein Test mit aktiver Sitzung MUSS den sofortigen Entzug einer Mitgliedschaft nachweisen.
Es sind noch keine solchen Diensttests ausgeführt.

Quellen: [RFC 7662](https://www.rfc-editor.org/rfc/rfc7662),
[RFC 8705](https://www.rfc-editor.org/rfc/rfc8705).
Die Laufzeiten sind technische Projektfestlegungen und keine von diesen Quellen vorgeschriebenen Werte.
