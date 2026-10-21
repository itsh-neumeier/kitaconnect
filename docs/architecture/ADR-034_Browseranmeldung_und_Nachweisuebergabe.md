# ADR-034 – Browseranmeldung und atomare Nachweisübergabe vorbereiten

Datum: 21.10.2026. Status: technischer Entwurf für KC-006F / #67; Sicherheitsentscheidungen offen.
Grundlage: Spezifikation v5.9, ADR-014/015/019 und ADR-029 bis ADR-033.

## Ausgangszustand

PR #72 ist offen und zur Durchsicht bereit. Abschlusskopf cb3d8f62d1fde50dcb67b96468534cf8876fbe1a besteht alle zehn Prüfungen von Push und PR.
Passwort-, Passkey- und TOTP-Adapter besitzen echte positive und negative Tests, erzeugen aber keine vollständige Anmeldung.
Ihre positiven Ergebnistypen enthalten nur die Identitätszuordnung. Die Sitzungsanlage setzt ausdrücklich einen bereits vollständig geprüften Zweck voraus.
Ein Gateway-Dienst und eine Browseranmeldung sind noch nicht implementiert.
Die Mitgliedschaft besitzt noch keinen aktuellen Speicheradapter.

## Technische Richtung

Der [Vertragsentwurf](../../contracts/identity-access/browser-auth-v1.md) konkretisiert die noch fehlende Übergabe.
Eine atomare, einmalige und versionierte Nachweisübergabe soll den Zwischenraum zwischen Faktorprüfung und Sitzungserzeugung schließen.
Sie muss im Identitätsdienst an den tatsächlich geprüften Bestand gebunden werden.
Das Gateway darf keinen alten positiven Ergebniswert in einen neuen Anmeldenachweis umwandeln.

Alle schutzrelevanten Identitätsschreibvorgänge benötigen vor Sitzungsabschluss eine gemeinsame Serialisierungsgrenze.
Die bestehende Versuchszählung bleibt vor teurer Arbeit dauerhaft bestätigt.
Ein zusammengesetzter Sitzungsabschluss benötigt einen eigenen Transaktionsvertrag; er darf bestehende Eigen-Transaktionen nicht verschachteln.
Neue Migrationen ergänzen den Bestand. Alte Migrationen und produktive RLS-Sperren bleiben erhalten.

Das Gateway erhält keine Datenbankrolle. Identitätsbestand bleibt bei identity-access, Mitgliedschaften bleiben gemäß Dienstmatrix bei organization-config.
Die lokale Identitätstransaktion kann keine fremde Mitgliedschaft atomar einschließen. Der aktuelle Organisationsaufruf bleibt für geschützte Prüfungen erforderlich.
Prüfzeitpunkte und konkurrierender dienstübergreifender Widerruf benötigen vor Code einen konkreten Vertrag; eine alte positive Antwort ist keine dauerhafte Freigabe.
Fachliche Rechteverwaltung, Föderation und produktiver Schlüsselbetrieb werden nicht in diesen Vorbereitungsschritt vorgezogen.
Eine Entscheidung über Fristen, MFA-Kombination und Browser-Schutz wird mit konkreten Vorschlägen angefordert.

## Offene Entscheidungen

| Thema | Vorgelegter Vorschlag | Grenze |
|---|---|---|
| Fristen | 15 Minuten Inaktivität, 8 Stunden absolut, zusätzliche Authentisierung höchstens 5 Minuten; Verkürzung durch Träger erlaubt. | Bisherige Frage bleibt unbeantwortet. Keine abhängige Laufzeitregel. |
| MFA-Kombination | Passwort plus TOTP oder Passkey mit zwingender Benutzerverifikation erfüllt die MFA-Pflicht. | Passkey-Benutzerverifikation ist bestätigt; ihre Gleichstellung mit Passwort plus TOTP ist neu angefragt. |
| Browser-Schutz | Secure, HttpOnly, SameSite=Strict, keine Domain-Freigabe; serverseitig gebundener CSRF-Nachweis und genaue Origin-Prüfung. | ADR-015 verlangt Bestätigung vor Implementierung. Externe SSO-Abläufe folgen KC-007. |

Produktive Domain, Rollen-/Poolbindung, Versuchslimits und Schlüsselbetrieb bleiben gesondert offen.
Die Spezifikation verlangt aktuelle Mitgliedschaftsprüfung, legt aber keine neue öffentliche Vergabe- oder Transferregel in KC-006F fest.
Solche Fachregeln werden nicht aus synthetischen Testzuordnungen abgeleitet.

## Entwicklungsreihenfolge und Nachweise

1. Antworten dokumentieren und Vertrag sowie diese ADR vor abhängigen Codeänderungen versionieren.
2. Den konkreten Speicher- und Sperrvertrag für einmalige Nachweisübergabe und aktuelle Mitgliedschaft ausarbeiten.
3. Den Identitätsdienst mit positiven und negativen echten Datenbanktests einschließlich Konkurrenz und Neustart erweitern.
4. Das Rust-Gateway nach ADR-015 mit dem bestätigten Cookie-/CSRF-Vertrag anbinden.
5. Browser-, TLS- und Dienstidentitätsnachweise getrennt prüfen; ursprüngliche Gesamtkriterien vollständig nachführen.

Reine Routertests erfüllen keinen mTLS-, Browser- oder produktiven RLS-Nachweis.
Diese Vorbereitung ergänzt keine Abhängigkeit, Migration oder Laufzeitfunktion. Alle Kriterien von #67 bleiben offen.

## Quellen

Der Vertragsentwurf verlinkt die geprüften OWASP- und NIST-Primärquellen.
Die Quellen liefern die fachliche Begründung der Vorschläge; sie ersetzen keine Projektentscheidung oder unabhängige Sicherheitsprüfung.
