# P1: Rechtegrundlage und vollständiger Anmeldeadapter

Stand: 25.10.2026. Grundlage: Spezifikation v5.9 und ausdrücklich beauftragte Phase P1.

## Ausgangspunkt

Der Mitgliedschaftsverbraucher in [PR #97](https://github.com/itsh-neumeier/kitaconnect/pull/97)
liefert aktuelle Mitgliedschaft und Fristen. Er enthält keine MFA-Entscheidung.
Der vollständige Anmeldeadapter benötigt zusätzlich die aktuelle zuständige Rechte-/MFA-Quelle.
Rollen und Policies gehören nach ADR-014 zum Identitätsdienst. Mitgliedschaft bleibt beim Organisationsdienst.

KC-008 hängt bisher vom vollständigen KC-006 ab. Würde KC-006F umgekehrt den Abschluss von KC-008 verlangen,
entstünde ein Kreis. Der technische Vorlauf wird deshalb von der späteren Gesamtabnahme getrennt.
Es wird keine fachliche Anforderung gestrichen oder als bereits erfüllt markiert.

## Prüffähige Reihenfolge

| Schritt | Voraussetzung | Ergebnis |
|---|---|---|
| [KC-008A / #98](https://github.com/itsh-neumeier/kitaconnect/issues/98) | KC-005 und KC-006C/D/E | Aktuelle bekannte Rechte-/MFA-Grundlage im Identitätsdienst; keine Fachfreigabe |
| [KC-006F / #67](https://github.com/itsh-neumeier/kitaconnect/issues/67) | KC-006C/D/E und KC-008A | Vollständiger Anmeldeadapter, Gateway, Browser und ursprüngliche Anmeldenachweise |
| [KC-008B / #99](https://github.com/itsh-neumeier/kitaconnect/issues/99) | KC-005, KC-006 und KC-008A | Vollständige Rollen-, Objekt-, Feld-, Einrichtungs- und Trägernachweise |
| [KC-008 / #8](https://github.com/itsh-neumeier/kitaconnect/issues/8) | Ursprüngliche Voraussetzungen und beide Teilpakete | Ursprüngliche Gesamtabnahme; keine automatische Abnahme aus KC-008A |

Die bereits geprüften Teile von KC-006F bleiben erhalten. Sie werden nicht erneut implementiert.
Die Zerlegung ändert ausschließlich technische Abhängigkeiten und die Reihenfolge der Nachweise.
Der versionierte Backlog prüft auch Teilpakete und erkennt verdeckte Kreise.

## Sicherheitsgrenze des Vorlaufs

KC-008A muss vor Code einen eigenen Dienstvertrag und die konkrete Bestandsableitung beschreiben.
Ein manueller falscher MFA-Wert darf fehlende Rollen- oder Policyinformationen nicht ersetzen.
Die Datenquelle muss bekannte vollständige Einstufung, genaue Bindung und aktuelle Bestandsversion nachweisen.
Fehlender, unbekannter, entzogener oder nicht verfügbarer Bestand verweigert Zugriff.
Die bestätigte MFA-Zuordnung aus ADR-043 bleibt unverändert.
MFA allein erteilt keine Objekt- oder Feldrechte.

Es entstehen in diesem Plan keine Standardrollen, Fachfreigaben, Migrationen oder erlaubenden Policies.
Der Identitätsdienst darf keine fremden Organisationstabellen lesen.
Öffentliche Rollenvergabe, produktiver Betrieb und vollständige Objekt-/Feldrechte sind keine Ergebnisse dieses Planungs-PRs.
Ungeklärte Fachregeln müssen vor abhängigem Code dokumentiert und entschieden werden.
Persönliche und sprachliche Durchsicht erfolgt gesammelt. Es liegt kein Merge-Auftrag vor.

## Technischer Vorlauf KC-008A

Der [Bestandsvertrag](../contracts/identity-access/rights-assurance-v1.md) und ADR-057 konkretisieren den nächsten Schritt.
Ein atomarer eingestufter Rollenbestand bindet Identität, Einrichtung und aktuelle Version.
Migration 0010 ergänzt ausschließlich diesen eigenen Bestand. Es gibt keine vorgegebenen Rollen und keine erlaubende Laufzeitpolicy.
Die aktuelle Ableitung verwendet die bestehende MFA-Zuordnung aus ADR-043.
Anmeldeeinbindung und vollständige Rechteabnahme bleiben die folgenden Schritte.
