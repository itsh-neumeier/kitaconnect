# ADR-037 – Aktuelle Sitzungsprüfung mit dauerhaften Obergrenzen

Datum: 21.10.2026. Status: technische Konkretisierung vor Code für KC-006F.
Grundlage: bestätigte Sitzungsfristen und MFA-Regeln, ADR-019/034/035/036.

## Entscheidung

Der [interne Prüfvertrag](../../contracts/identity-access/session-check-v1.md) ergänzt die Prüfung gebundener Sitzungen.
Die bestehende Sitzung speichert unveränderliche Ausgangsgrenzen. Ein eigener Beobachtungsbestand speichert strengere Grenzen dauerhaft.
Er trennt gekürzte Sitzungsgrenzen von den Rohwerten einer versionierten Einrichtungspolicy.
Diese Trennung verhindert falsche Versionskonflikte nach einer bereits bei Anmeldung gekürzten Frist.

Die gemeinsame Identitätssperre serialisiert Prüfung, Faktorpflege und Abmeldung.
Jede Prüfung fragt die aktuelle Organisation ab; ein lokaler positiver Zustand ersetzt diesen Aufruf nicht.
Rückläufige Versionen, fehlerhafte Antworten und fehlende Bindungen verweigern Zugriff.
Ein beobachteter Ablauf bleibt endgültig. Aktivität verlängert weder absolute Laufzeit noch Authentisierungsalter.

## Alternativen und Folgen

Eine ausschließlich flüchtige Minimum-Berechnung wurde verworfen: Nach späterer Lockerung könnten alte Grenzen erneut gelten.
Eine Änderung der ursprünglichen Sitzungsfelder wurde verworfen: Deren Unveränderlichkeit schützt den bestehenden Speichervertrag.
Migration 0006 ergänzt deshalb einen eigenen Bestand mit unveränderlicher Zuordnung, monotoner Version und nur kürzbaren Grenzen.
Abmeldung bleibt eine lokale Widerrufsoperation und benötigt keinen verfügbaren Organisationsdienst.

## Abnahmegrenzen

Dies ist keine Browser-, Dienstidentitäts- oder Fachberechtigungsfreigabe.
Rotation und zweckgebundene erneute Faktorprüfung folgen in eigenen kleinen Schritten.
Die Tests verwenden ausschließlich synthetische Organisationsantworten und Zustände.
Produktive Domain, Schlüsselbetrieb, Rollen-/Poolbindung und alle bestehenden Fachsperren bleiben gesondert.
