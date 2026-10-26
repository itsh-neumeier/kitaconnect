# Browsergrenze des Gateways v1

Stand: 26.10.2026. KC-006F / #67. Bestätigte Regeln aus ADR-015/034/049.

## Umfang und Vertrauensgrenze

Das eigenständige Rust-Gateway erhält zunächst eine reine Bibliothek für die Browser-Anfragegrenze.
Sie prüft Transportdarstellung, genaue Origin und benötigte Nachweisfelder. Sie besitzt keine Datenbankrolle.
Das Ergebnis enthält ausschließlich ungeprüft vorgelegte Geheimnisse. Es ist kein positiver Benutzer-, CSRF- oder Sitzungsnachweis.
Der Identitätsdienst muss den CSRF-Nachweis noch gegen seinen aktuellen Vorgangs- oder Sitzungsbestand prüfen.
Die Prüfung muss auch vor Anmeldung und Abmeldung erfolgen. SameSite ersetzt diesen Nachweis nicht.
Öffentliche Kontenauflösung bleibt bis zur Entscheidung über die Anmeldekennung offen.

## Feste Anfrageprofile

Alle folgenden Aufrufe verwenden POST und einen leeren Queryteil. Eine absolute Anfrage-URI ist verboten.
Die Route wählt das Profil serverseitig; kein Clientfeld bestimmt den Prüfmodus.

| Operation | Pfad | Benötigter Cookie | CSRF-Header |
|---|---|---|---|
| Anonymen Browserkontext vorbereiten | `/api/v1/auth/context` | keiner | noch keiner; nur Vorbereitung ohne Benutzerrechte |
| Anmeldevorgang beginnen | `/api/v1/auth/login/start` | `__Host-kc_context` | `X-KC-CSRF` |
| Sitzung aktuell prüfen | `/api/v1/auth/session/check` | `__Host-kc_session` | `X-KC-CSRF` |
| Abmelden | `/api/v1/auth/session/logout` | `__Host-kc_session` | `X-KC-CSRF` |
| Sitzungsnachweis wechseln | `/api/v1/auth/session/rotate` | `__Host-kc_session` | `X-KC-CSRF` |

Die Kontextvorbereitung verlangt bereits die genaue Origin. Sie erteilt keine Anmeldung und ersetzt keine spätere CSRF-Prüfung.
Anmeldekennung und Nutzlast des Beginns folgen erst nach Entscheidung. Weitere Faktorendpunkte erhalten eigene Verträge.
Die Sitzungsprüfung ist hier POST, weil der bestehende Prüfer Zeiten, Fristen und Widerruf dauerhaft beobachten kann.

## Origin und Header

Der erwartete Ursprung stammt ausschließlich aus vertrauenswürdiger Konfiguration.
Er muss eine kanonische HTTPS-Origin ohne Pfad, Benutzerinformation, Query oder Fragment sein.
Das freigegebene isolierte Browserprofil ist `https://localhost:3443`. Produktive Namen bleiben offen.
Genau ein Origin-Header muss dessen exakter kanonischer Darstellung entsprechen.
Genau ein Host-Header muss der konfigurierten Authority entsprechen. Mehrdeutige, fehlende und fremde Werte werden abgewiesen.
Forwarded-, Proxy-, Identitäts- und Rollenheader bestimmen weder Ursprung noch Benutzerkontext.
Die Bibliothek gibt solche Header nicht weiter. Authorization ist auf diesen Browserprofilen nicht zulässig.
Dienstauthentisierung entsteht später ausschließlich durch das konfigurierte Gateway-Verfahren.
Headeranzahl, gesamte Headergröße und Cookiegröße werden ausdrücklich positiv begrenzt. Es gibt keine produktiven Vorgabewerte.

## Cookies und Geheimnisse

Ein Cookie-Header wird als eindeutige Folge von Name/Wert-Paaren gelesen. Mehrere Cookie-Header oder doppelte Namen werden abgewiesen.
Der benötigte Name wird exakt und ohne Ersatz durch einen anderen Cookie ausgewählt.
Andere syntaktisch gültige Cookie-Paare erteilen keine Rechte und werden nicht weitergegeben.
Das Profil verwendet ungequotete sichtbare Cookie-Werte ohne Komma, Semikolon, Backslash oder Steuerzeichen.
Benötigter Cookie und CSRF-Header enthalten jeweils genau 32 Bytes in kanonischer ungepaddeter Base64url-Darstellung.
Geheimnisse bleiben in löschenden Puffern; Debug-Ausgaben verdecken ihre Inhalte.
Die spätere Serverbindung muss den vorgelegten CSRF-Wert dem richtigen aktuellen Kontext zuordnen. Die Formprüfung erledigt das nicht.

Die Ausgabe verwendet `__Host-kc_context` beziehungsweise `__Host-kc_session` mit `Path=/; Secure; HttpOnly; SameSite=Strict`.
Eine Domain-Eigenschaft wird nicht gesetzt. Löschen verwendet dieselben Attribute und `Max-Age=0`.
Cookie-Header werden als geheim markiert. Zustands- und Geheimnisantworten benötigen später `Cache-Control: no-store` und `Pragma: no-cache`.
Der Sitzungsnachweis wird niemals aus einem Clientwert erzeugt oder als gültig übernommen.

## Nachweise und Grenzen

Reine Tests prüfen alle Profile, genaue Origin/Authority, Methode/URI, mehrdeutige Header, Größenbegrenzung,
Cookieverwechslung, doppelte Namen, kanonische Geheimnisse, Ausgabeattribute und verdeckte Diagnosen.
Diese Tests beweisen weder serverseitige CSRF-Bindung noch tatsächliches HTTPS-, Cookie- oder Browserverhalten.
Es entsteht in diesem Schritt kein öffentlich lauschender Dienst und kein Datenbankzugriff.
Der vorhandene WebAuthn-Test auf `http://localhost:3000` bleibt getrennt. Secure wird nicht für diesen Test abgeschwächt.
