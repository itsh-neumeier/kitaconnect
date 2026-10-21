# ADR-040: Abschluss eigener Transaktionen

- Datum: 21.10.2026
- Status: Technische Fehlerkorrektur im beauftragten Umfang von KC-006F
- Grundlage: PR #77, SQLx 0.9, [Dienstvertrag](../../contracts/identity-access/transaction-completion-v1.md)

## Problem

Ein nur vorgemerkter Rollback kann bei offen gehaltener Verbindung Sperren behalten.
Eine unabhängige Operation wartet dann trotz bereits ausgegebenem Fehler.
Der Rotations-Datenbanktest hat dieses Verhalten nachgewiesen.

## Entscheidung

Eigene Transaktionen erhalten einen ausdrücklich abgewarteten Abschluss.
Der Fehlerpfad wartet Rollback ab. Erfolg und absichtlich dauerhafte negative Zustände warten Commit ab.
Die Implementierung trennt diese Ergebnisse, damit ein erkannter Ablauf nicht zurückgenommen wird.
Bereits bestätigte Versuchslimits werden nicht zurückgesetzt.
Tests verwenden eine getrennte Zeugenverbindung, bevor die Fehlerverbindung erneut verwendet wird.

## Folgen und Grenzen

Die Korrektur erfolgt in kleinen Paketen. Der erste Schritt umfasst Sitzungsbestand, Passwortadapter und Einrichtungseinstellungen.
Die weiteren Faktoradapter und der Anmeldeabschluss bleiben bis ihrer Prüfung ausdrücklich offen.
Äußere Transaktionen, abgebrochene Futures und Verbindungsverlust bleiben eigene Fälle.
Ohne bestätigte Bereinigung darf eine Verbindung nicht wiederverwendet werden.
Diese technische Entscheidung verändert keine fachlichen oder sicherheitsbezogenen Projektvorgaben.
