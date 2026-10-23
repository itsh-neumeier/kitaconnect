# ADR-053: Authentisierte Mitgliedschaftsabfrage

- Datum: 23.10.2026
- Status: Technische Konkretisierung vor Code
- Umfang: KC-006F, interner lesender Organisationsadapter

## Entscheidung

Der [Dienstvertrag](../../contracts/organization-config/authenticated-membership-v1.md) verbindet den tatsächlichen mTLS-Prüfzugang,
frische OAuth-Introspektion und den eigenen Mitgliedschaftsbestand.
Die Datenbank des Organisationsdienstes liefert die frische Zeit. Fremde Tabellen werden nicht gelesen.
Der vorhandene reine Antwortparser prüft die genauen Bindungen vor dem lokalen Lesen und die Zeit erneut nach Wartezeiten.
Ein ausdrücklich konfigurierter Serverfingerabdruck ergänzt den bestehenden TLS-Namensvergleich.
Produktive Verteilung und Rotation dieser Konfiguration bleiben offen.

## Begründung und Grenzen

Der lokale Lesebestand und die geprüften Transportbausteine bestehen bereits.
Ihre Verbindung benötigt keine zusätzliche fachliche Zugriffsregel oder Migration.
Der Adapter gibt ausschließlich einen Datenstand zurück. Er erfindet keine fehlende Rechte- oder MFA-Entscheidung.
Es gibt keine dienstübergreifende Atomarität und keinen positiven Berechtigungscache.
Ein öffentlicher Router und der vollständige Anmeldeadapter folgen getrennt.
Der Vertrag steht vor abhängigem Code. Die Laufzeit-RLS bleibt geschlossen.
