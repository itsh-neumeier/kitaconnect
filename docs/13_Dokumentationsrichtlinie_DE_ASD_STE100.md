# Dokumentationsrichtlinie – Kontrollierte Technische Sprache Deutsch

**KitaConnect v5.9 · verbindliche Projektrichtlinie · 17.10.2026**

## 1. Geltungsbereich und Einordnung
Die gesamte **deutschsprachige** KitaConnect-Projektdokumentation folgt einer kontrollierten technischen Sprache **in Anlehnung an die Prinzipien von ASD-STE100 (Simplified Technical English)**. ASD-STE100 regelt vereinfachtes technisches **Englisch**. Diese Projektrichtlinie ist eine **eigene deutsche Adaption**, keine amtliche Übersetzung und kein Nachweis einer ASD-STE100-Konformität. Englische Quelltexte, Bezeichner, API-Namen und Programmiersyntax bleiben unverändert, wenn ihre Änderung Schnittstellen beschädigen würde.

Gültig für Fachkonzept, Architektur, Datenmodell, API-Dokumentation, UX-Texte, Benutzer- und Betriebshandbücher, Sicherheits- und Datenschutzanweisungen, Fehlermeldungen, GitHub-Issues, Pull Requests sowie Dokumentation durch Codex und Claude. Die Produktlokalisierung Deutsch/Englisch bleibt davon unberührt.

## 2. Verbindliche Schreibregeln
1. Schreibe kurze, eindeutige Sätze. Bevorzuge einen Sachverhalt je Satz.
2. Verwende die aktive Form. Nenne die handelnde Rolle, wenn sie wichtig ist.
3. Verwende für denselben Sachverhalt immer denselben Fachbegriff. Prüfe das Glossar.
4. Formuliere Handlungen als klare Anweisungen im Imperativ: „Öffnen Sie …“, „Prüfen Sie …“.
5. Nummeriere Handlungsfolgen. Trenne Voraussetzungen, Schritte, Ergebnis und Fehlerbehandlung.
6. Definiere Abkürzungen bei der ersten Verwendung. Vermeide nicht definierte Synonyme.
7. Schreibe Anforderungen überprüfbar. Verwende „MUSS“, „DARF NICHT“, „SOLL“ und „KANN“ einheitlich.
8. Gib Werte mit Einheit und Geltungsbereich an. Vermeide Wörter wie „schnell“, „einfach“ oder „bald“ ohne Kriterium.
9. Kennzeichne Warnungen nach Risiko. Beschreibe Ursache, mögliche Folge und Schutzmaßnahme.
10. Schreibe Fehlermeldungen so, dass die betroffene Person den nächsten zulässigen Schritt erkennt. Gib keine Geheimnisse preis.
11. Trenne Fakten, Annahmen, Entscheidungen und offene Prüfungen. Behaupte keine implementierte Funktion ohne Nachweis.
12. Prüfe neue und geänderte Dokumente vor dem Merge auf Verständlichkeit, Konsistenz und Fachbegriffe.

## 3. Vorlagen
### Anforderung
- **ID:** REQ-…
- **Anforderung:** „Das System MUSS …“
- **Geltungsbereich:** Rolle, Mandant, Modul und Datenklasse
- **Akzeptanzkriterium:** Beobachtbares Verhalten mit Testnachweis

### Betriebsanweisung
- **Ziel:** Was wird erreicht?
- **Voraussetzungen:** Welche Rechte, Systeme und Sicherungen werden benötigt?
- **Schritte:** Nummerierte Einzelaktionen
- **Ergebnisprüfung:** Woran ist Erfolg erkennbar?
- **Fehlerbehandlung:** Was ist bei einem Fehler zu tun?

### Warnhinweis
- **WARNUNG:** Risiko und mögliche Auswirkung.
- **Schutzmaßnahme:** Erforderliche Aktion vor dem Fortfahren.

## 4. Beispiele
| Vermeiden | Bevorzugen |
|---|---|
| „Es sollte dann eventuell eine Sicherung durchgeführt werden.“ | „Erstellen Sie vor dem Update eine Sicherung.“ |
| „Die entsprechenden Nutzer können darauf zugreifen.“ | „Nur berechtigte Mitarbeitende der Einrichtung dürfen die Kinderakte lesen.“ |
| „Das System verarbeitet die Daten zeitnah.“ | „Der Hintergrunddienst beginnt die Verarbeitung innerhalb der festgelegten Frist.“ |
| „Klicken Sie auf OK, um fortzufahren.“ | „Wählen Sie **Speichern**. Prüfen Sie anschließend die Bestätigung.“ |

## 5. Qualitätssicherung
- Jeder Dokumentations-PR MUSS eine manuelle sprachliche Prüfung erhalten.
- CI SOLL Glossarbegriffe, defekte Links, Formatierung und definierte Pflichtabschnitte automatisiert prüfen. Ein deutscher Text darf nicht fälschlich als formell ASD-STE100-zertifiziert gekennzeichnet werden.
- Die Redaktion pflegt `docs/14_Fachwortverzeichnis_DE.md` als verbindliche Terminologiequelle.
- Für rechtlich verbindliche Hinweise gelten zusätzlich juristische Prüfung und gesetzliche Formvorgaben. Vereinfachung darf keine fachliche oder rechtliche Aussage verfälschen.

## 6. Status
Diese Richtlinie ist eine **Anforderung und redaktionelle Vorgabe**. Die vorhandenen Dokumente werden ab v5.9 damit verknüpft. Eine vollständige redaktionelle Umschreibung jedes Bestandsabschnitts ist **noch nicht abgeschlossen** und wird im Backlog nachverfolgt.
