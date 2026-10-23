# Dienstvertrag: Interne Mitgliedschaftsabfrage über HTTP v1

- Stand: 23.10.2026
- Umfang: KC-006F / #67; ausschließlich identity-access an organization-config
- Grundlage: authenticated-membership-v1, ADR-019/044–053, Spezifikation v5.9

## Aufruf und Datenhoheit

`POST /internal/v1/membership/read` liest einen Datenstand. Der Aufruf verändert keine Daten.
Nur eine tatsächliche gegenseitig geprüfte TLS-Verbindung mit identity-access darf die Abfrage ausführen.
Genauer DNS-SAN, aktueller Zertifikatsfingerabdruck, Ziel und Umfang `membership:read`
werden bei jeder Anfrage durch den bestehenden authentisierten Mitgliedschaftsadapter geprüft.
Der Router erzeugt dessen Prüfzugang ausschließlich aus dem eigenen tatsächlichen Datenstrom.
HTTP-Header, JSON-Felder und weitergeleitete Zertifikatsbehauptungen erzeugen keinen Prüfzugang.

Der Diensttoken steht ausschließlich in genau einem `Authorization: Bearer <token>`-Header.
Das Schema ist unabhängig von Groß-/Kleinschreibung. Genau ein Leerzeichen trennt Schema und Token.
Der Token besitzt die bestehende kanonische Base64url-Darstellung ohne Padding: 43 Zeichen für 32 Bytes.
Query-Parameter, Cookies, Content-Encoding, Proxy-Authorization sowie Forwarded-/X-Forwarded-Header
und verbreitete weitergeleitete Clientzertifikatsheader werden abgewiesen.
Andere Authentisierungsmittel und Browserzugriffe gehören nicht zu diesem Vertrag.
Der Token bleibt zusätzlich an das tatsächlich verwendete Aufruferzertifikat gebunden.
Grundlage für den Headertransport: [RFC 6750, Abschnitt 2.1](https://www.rfc-editor.org/rfc/rfc6750#section-2.1).

## Anfrage und Antwort

Der Medientyp ist `application/json`, optional mit genau einem `charset=utf-8`.
Der Body enthält genau die Felder `identity_ref` und `facility_id` als Zeichenketten.
Beide Kennungen sind positive u128-Werte in kanonischer Dezimaldarstellung.
Vorzeichen, führende Nullen, Leerzeichen, Exponenten, Null und Überlauf werden abgewiesen.
Die Prüfung verwirft doppelte Felder einschließlich gleichwertiger JSON-Escapes,
unbekannte Felder, falsche Typen, Nullwerte und Daten hinter dem einzelnen JSON-Objekt.
Die Identitätsreferenz muss beim aufrufenden Identitätsdienst aus dessen geprüftem Ablauf stammen.
Der Router beweist selbst keine Benutzeranmeldung.

Eine erfolgreiche Antwort besitzt Status 200 und `application/json`.
Sie enthält genau `membership`. Fehlende, verborgene oder inaktive Mitgliedschaft ergibt `null`.
Eine vorhandene Mitgliedschaft enthält `identity_ref`, `facility_id`, `membership_version`,
`policy_version`, `idle_seconds`, `absolute_seconds` und `step_up_seconds`.
Kennungen und Versionen sind kanonische Dezimalzeichenketten. Fristen sind positive ganze Sekunden.
Die bestehenden Grenzen 900/28.800/300 Sekunden gelten unverändert.
Einrichtungszuordnung, Mitgliedschaft und Fristen stammen aus derselben bestehenden lokalen Einzelabfrage.
Der Router ergänzt keine Standardwerte bei fehlendem Bestand.

Die Antwort ist nur ein Datenstand. Sie erteilt keine Benutzer-, MFA-, Objekt- oder Feldrechte.
Insbesondere enthält sie keinen erfundenen Wert `mfa_required`.
Jede nächste Abfrage führt die vollständige aktuelle Dienst- und Mitgliedschaftsprüfung erneut aus.
Ein bestätigter Entzug vor der nächsten Prüfung muss wirken. Es gibt keinen positiven Zwischenspeicher.
Zwischen den Diensten besteht keine gemeinsame Transaktion. Ein Entzug nach dem jeweiligen Lesen
kann den bereits gelesenen Datenstand nicht rückwirkend ändern.

## Fehler und Geheimnisschutz

Routerfehler verwenden `application/problem+json` nach
[RFC 9457](https://www.rfc-editor.org/rfc/rfc9457).
Der Body enthält ausschließlich `type: "about:blank"`, einen festen Titel und den tatsächlichen numerischen Status.
Keine Kennung, kein Token, keine Zertifikats- oder SQL-Diagnose wird ausgegeben oder protokolliert.
Die begrenzte OAuth-Fehlerformatausnahme gilt hier nicht.

| Status | Auslöser |
|---|---|
| 400 | Ungültige Form, doppelte Anmeldemittel oder unzulässige alternative Eingaben |
| 401 | Fehlender Diensttoken oder abgewiesene aktuelle Dienstprüfung |
| 404 | Unbekannter Pfad |
| 405 | Andere Methode am bekannten Pfad; `Allow: POST` |
| 413 | Überschrittene Bodygrenze |
| 415 | Fehlender oder unzulässiger Medientyp |
| 429 | Ausgeschöpftes gemeinsames Anfragebudget |
| 503 | Ausgefallene Prüfung oder abgelaufene Anfragefrist |

401 enthält `WWW-Authenticate: Bearer realm="service"`.
Bei abgewiesenem vorhandenen Token wird ausschließlich `error="invalid_token"` ergänzt.
Routerantworten enthalten `Cache-Control: no-store` und `Pragma: no-cache`.
Fehler schließen die HTTP-Verbindung; ungelesene Bodies werden nicht für weitere Anfragen verwendet.
Fehler des vorgelagerten TLS-/HTTP-Parsers können die Verbindung schließen oder generische
400/408/431-Antworten ohne Routerbody liefern. Sie enthalten keine Anwendungsdaten.

## Ressourcen, Abbruch und Grenzen

Der Router übernimmt genau einen eigenen TCP-Datenstrom. Er startet keinen Listener.
Verbindungen, parallele Anfragen, Bodybytes, Headeranzahl, Headerbytes,
Handshake-, Header-, Anfrage- und Verbindungsfristen müssen ausdrücklich positiv konfiguriert sein.
Ein gemeinsamer Router besitzt gemeinsame Budgets. Weitere Instanzen besitzen getrennte Budgets.
Überzählige Verbindungen schließen vor TLS ohne HTTP-Klartextantwort.
Der bestehende Leser begrenzt zusätzlich Introspektion und eigene Datenbankoperationen.
Abbruch oder Fristende verwirft seine Ressourcen. Es entsteht kein losgelöster Hintergrundauftrag.
Der Router treibt den eingehenden TLS-Datenstrom während der Prüfung weiter an.
Beobachtetes Eingabeende, TLS-Fehler oder Zertifikatsablauf sperrt den geteilten Prüfzugang.
Unmittelbar vor Erzeugung einer erfolgreichen Antwort prüft der Router diesen Zugang erneut.
Die Aussage betrifft beobachtete Zustände; eine nicht beobachtete Netzwerkunterbrechung ist kein Zustandsnachweis.

## Nachweise und ausgeschlossener Umfang

OpenAPI beschreibt diese interne Route vor Code. Gezielte Parsertests prüfen Mehrdeutigkeit und Grenzen.
Die vorhandene tatsächliche HTTP-Datenbankgruppe prüft den Router mit frischen RAM-Zertifikaten,
tatsächlicher OAuth-Introspektion und getrennter Organisationsdatenbank vor/nach Neustart.
Positive Antwort, fehlende/fremde Mitgliedschaft, Entzug auf derselben Verbindung, falsches Zertifikat,
abgewiesener Token, Ausfall, Formatfehler, Grenzen und Freigabe der Budgets erhalten getrennte Nachweise.
Reine Parser-/Konfigurationstests sind keine mTLS- oder Datenbanknachweise.

Keine Migration, erlaubende Laufzeitpolicy, produktive Rolle oder produktive Konfiguration entsteht.
Neue direkte Pakete verwenden ausschließlich bereits gesperrte Versionen.
Produktiver Zertifikatsbetrieb, Rollen-/Poolbindung und der vollständige OrganizationAuthority-Adapter bleiben gesondert.
Gateway, Browser, MFA-/Fachrechte und ursprüngliche Gesamtabnahme bleiben offen.
