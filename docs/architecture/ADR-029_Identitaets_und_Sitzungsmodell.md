# ADR-029 – Internes Identitäts- und Sitzungsmodell

Datum: 20.10.2026. Status: technische Konkretisierung für KC-006A im P1-Auftrag.
Grundlage: v5.9, ADR-015 und ADR-019. Keine neue fachliche Berechtigung.

## Entscheidung

KC-006 wird in sechs Teilpakete mit ausdrücklichen Abnahmekriterien aufgeteilt.
Der [Plan](../24_KC-006_Authentisierung.md) erhält alle ursprünglichen Anforderungen an die vier Anmelde-/Wiederherstellungswege.
Das erste Paket implementiert nur den [internen Modellvertrag](../../contracts/identity-access/session-model-v1.md).
Die Bibliothek liegt im zuständigen Dienst `identity-access` und erhält zunächst keine externen Abhängigkeiten.

Identitätskennung und Sitzungsdatensatzkennung verwenden verschiedene Rust-Typen.
Eine Datensatzkennung wird ausdrücklich von einem geheimen Sitzungsnachweis getrennt.
Der Aufrufer übergibt Zeit und positive Grenzen. Es gibt keine eingebauten Projektvorgabewerte.
Beobachteter Ablauf und Widerruf bleiben innerhalb des Datensatzes endgültig.
Rückläufige Zeit kann keine Frist verlängern. Aktivität verlängert niemals die absolute Frist.

## Alternativen und Grenzen

Eine vollständige Authentisierung in einem PR wäre zu groß für die geforderte technische Prüfung.
Ein allgemeines gemeinsames Paket außerhalb der Dienstgrenze würde die Zuständigkeit verdecken.
Beide Alternativen werden verworfen.
Das Modell ist keine selbst entwickelte Kryptografie und kein Anmeldedienst.
Es nimmt weder Argon2id-Parameter noch MFA-, Cookie- oder Wiederherstellungsregeln vorweg.
Persistenz, aktuelle serverseitige Prüfung und Konkurrenzkontrolle folgen in KC-006B.
Eine alte Modellkopie darf später keine aktuelle Prüfung ersetzen.
Produktive Identitäts-/Poolbindung aus KC-005 bleibt offen. Sie wird durch dieses Modell nicht entschieden.

## Offene Entscheidungen

MFA-Zuordnung, konkrete Browser-Sitzungsgrenzen und Wiederherstellung wurden mit Vorschlägen angefragt.
Die [Entscheidungsliste](../ai-memory/DECISIONS_PENDING.md) hält ihren tatsächlichen Antwortstand fest.
Die Bibliothek darf unabhängig davon ohne Vorgabewerte implementiert werden.

## Technische Quellen

- [OWASP: Serverseitige Sitzungsprüfung und Fristen](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html)
- [OWASP: MFA und Wiederherstellung](https://cheatsheetseries.owasp.org/cheatsheets/Multifactor_Authentication_Cheat_Sheet.html)

Die Quellen begründen Prüfanforderungen. Sie bestätigen keine vorgeschlagenen KitaConnect-Zeitwerte oder Fachregeln.
