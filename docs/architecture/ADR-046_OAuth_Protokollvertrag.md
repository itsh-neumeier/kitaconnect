# ADR-046: OAuth-Protokollmodell vor Dienstanmeldung

- Datum: 22.10.2026
- Status: Technische Konkretisierung von ADR-019/042/045 vor Code; Namenszuordnung ausdrücklich bestätigt
- Umfang: KC-006F, Formular- und Darstellungsmodell ohne Laufzeitfreigabe

## Entscheidung

Der [OAuth-Dienstvertrag](../../contracts/identity-access/service-oauth-v1.md) und seine OpenAPI-Beschreibung werden vor Code versioniert.
Die Implementierung trennt Formularprüfung, kanonische Tokendarstellung und tatsächliche Clientauthentisierung.
Bekannte Formulare oder Scopes erteilen keine Berechtigung.
Unbekannte Erweiterungsparameter werden gemäß RFC 6749 ignoriert; doppelte Namen werden nach Dekodierung abgewiesen.
Body- und Parametergrenzen sind verpflichtende positive Konfiguration ohne produktive Vorgaben.

Für die Tokendarstellung wird vorhandenes Base64url ohne Füllzeichen verwendet.
Ein nicht darstellbarer abgefragter Token führt erst nach berechtigter Introspektion zur minimalen inaktiven Antwort.
Die positive Tokenantwort wartet auf belastbare verbleibende Ablaufmetadaten des Ausgabeadapters.
Die bestätigte OAuth-Fehlerformatausnahme bleibt auf Protokollendpunkte begrenzt.

## Bestätigte Sicherheitsentscheidung

RFC 8705 verlangt beim Verfahren mit Zertifizierungsstelle einen registrierten Zertifikatsnamen.
Die vorhandene genaue Fingerabdruckbindung ersetzt diese Festlegung nicht.
Die Projektleitung bestätigt am 22.10.2026 ausdrücklich genau einen registrierten DNS-Namen je Dienst.
Der aktive genaue Zertifikatsfingerabdruck bleibt zusätzlich verpflichtend.
Wildcards und automatische Registrierung sind ausgeschlossen. Produktive Namen bleiben offen.
Diese Antwort wird vor Namensadapter und abhängiger Dienstanmeldung versioniert.
Das Protokollmodell bleibt ein getrennt prüfbarer Baustein.

## Begründung und Grenzen

Diese Trennung ermöglicht kleine nachweisbare Schritte ohne eine Sicherheitsentscheidung vorwegzunehmen.
Ein großzügiger Decoder mit stiller UTF-8-Ersetzung oder mehreren akzeptierten Tokenformen wurde verworfen.
Eine ungeprüfte Übernahme der angeforderten Tokenlaufzeit wurde verworfen.
Die zwei direkten Hilfsbibliotheken sind bereits mit denselben Versionen im Abhängigkeitsgraphen vorhanden.
Keine neue Kryptografie, Migration, Rolle oder fachliche Zugriffsregel entsteht.
Router, aktuelle Registrierung, Datenbank-Laufzeitrechte, Organisation und Browser bleiben getrennte Nachweise.
