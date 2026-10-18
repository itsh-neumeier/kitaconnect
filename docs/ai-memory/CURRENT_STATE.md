# Projektstatus und Fortsetzung

Aktualisiert: 18.10.2026, Europe/Berlin. Grundlage: Spezifikation v5.9.
Status: KC-037 dokumentiert bestätigte Ziele, technische Festlegungen und ausdrücklich zurückgestellte Freigaben.
P0 ist insgesamt nicht abgenommen. P1 bis P9 besitzen keinen Implementierungsauftrag.

## Arbeitsstand

- `issue_id`: [KC-037 / #37](https://github.com/itsh-neumeier/kitaconnect/issues/37).
- `branch`: `codex/p0-decision-register`.
- `head_commit`: Ausgangscommit `ec907b82323b8beb9c41a22eb777233af378667e`; KC-037 wird als eigener Dokumentationscommit gesichert.
  Aktuellen Branchkopf mit `git rev-parse HEAD` prüfen.
- `objective`: O-001 bis O-008 mit Entscheidungen oder begründeten späteren Prüfterminen dokumentieren.
- `pull_request`: Eigener KC-037-PR auf Basis von `codex/p0-test-security-foundations`; Link im GitHub-Issue #37.
- `migration_state`: Keine Datenbankrollen, Migrationen, Zertifikate, Zugangsdaten oder Laufzeitdienste angelegt.
- `dependencies`: Bisherige PRs #39, #40 und #41 zuerst prüfen und integrieren. Kein Merge durch diese Sitzung.

## Antworten der Projektleitung

Quelle: ausdrückliche Antworten im Projektchat am 18.10.2026.

1. Das vorgeschlagene Referenzprofil wurde mit „Dieses Referenzprofil bestätigen“ angenommen.
2. Rechteinhaber, juristische Lizenzprüfung und unabhängige Wahlprüfung sollen ausdrücklich „offen lassen“ bleiben.
3. Die Antwort „@itsh-neumeier Timo Neumeier“ bestätigt die vorläufige CODEOWNER-Zuordnung.
4. Der Auftrag „gerne fragen stellen danach autonom weiterarbeiten“ delegiert die technische Weiterarbeit innerhalb der bestehenden Anforderungen.

Die CODEOWNER-Antwort ernennt Timo Neumeier nicht zum Rechteinhaber oder unabhängigen Fachprüfer.
Die frühere Auswahl `1A, 2A, 3A, 4A` bleibt gültig. ADR-014 bis ADR-017 werden nicht erneut abgefragt.
Die unabhängige Prüfung des Wahlprotokolls bleibt vor dessen Implementierung verpflichtend.

## Erledigt

1. AGENTS, CLAUDE, Checkpoint, ADRs, relevante Spezifikation sowie Issues #37/#38 gelesen.
2. Sauberen Arbeitsbaum und erfolgreichen CI-Stand des KC-003-Ausgangs geprüft.
3. [Entscheidungsregister](../17_P0_Entscheidungsregister.md) mit O-001 bis O-008 und spätesten Prüfterminen erstellt.
4. Bestätigtes Lastprofil und reproduzierbares Messverfahren dokumentiert.
5. [ADR-019](../architecture/ADR-019_Dienstidentitaeten.md) konkretisiert Diensttokens, mTLS, Widerruf und aktuelle Rechteprüfung.
6. [ADR-020](../architecture/ADR-020_Hintergrundauftraege.md) konkretisiert Redis Streams, dauerhafte Zustellnachweise, Wiederholungen und Fehlerablage.
7. Primärquellen für PostgreSQL, Redis, Helm und Kubernetes recherchiert. Planungsstände sind ausdrücklich keine getestete Installationsfreigabe.
8. Vorläufigen CODEOWNER eingetragen. Unabhängige Reviews und tatsächlicher Branchschutz bleiben offen.
9. Architektur, Testkonzept, Betriebshinweise, Lizenzprüfpunkte, Entscheidungsliste und P0-Plan abgeglichen.

## Bestätigtes Lastprofil

Eine Einrichtung, 50 gleichzeitig angemeldete Benutzer und insgesamt 10 Anfragen pro Sekunde.
80 % Lesen und 20 % Schreiben. p95 höchstens 500 ms beim Lesen und 1.000 ms beim Schreiben.
Technische Fehlerrate unter 1 %. Referenz: 4 vCPU, 8 GB RAM und 100 GB SSD.
5 Minuten Aufwärmzeit und 30 Minuten Messung. Medienverarbeitung wird gesondert gemessen.
Die Zielwerte sind bestätigt. Es gibt noch keine repräsentativen Lastmessergebnisse.
Der spätere synthetische Testbestand und das genaue Verfahren stehen im Register.

## Technische Quellen und Versionen

Stand vom 18.10.2026: PostgreSQL 18.6, Redis 8.10.2, Helm 4.3.1 und Kubernetes 1.37.1.
Die Primärquellen sind im Register verknüpft. Versionen, Images, Sicherheitsmeldungen,
Lizenzwahl und tatsächliche Kompatibilität MÜSSEN vor Infrastrukturimplementierung in KC-032 erneut geprüft werden.
Compose-/Portainer-Patchstände und konkrete S3-Anbieter sind bewusst noch nicht festgelegt.

## Prüfungen

| Prüfung | Ergebnis |
|---|---|
| `node scripts/check-project.mjs` | 38 Issues, 27 Dokumente, lokale Verweise und synthetische Daten geprüft |
| `node scripts/fixtures.mjs --check` | Bytegenau erfolgreich |
| `node --test scripts/*.test.mjs` | 13 von 13 Tests erfolgreich |
| `git diff --check` | Erfolgreich |
| Ausgangscommit `ec907b8` | [P0 CI](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39121874227) und [P0 Security](https://github.com/itsh-neumeier/kitaconnect/actions/runs/39121874219) erfolgreich |

Nur Dokumentation und CODEOWNERS geändert. Lokale Frontend-/Rust-Builds werden nicht wiederholt.
Secret-Scan und CI-Nachweise des neuen Commits werden im PR nachgeführt.
Keine bekannten neuen Testfehler. Kein Nachweis implementierter Dienst-, Last- oder Wahlsicherheit behauptet.

## Noch offene Freigaben

- Projektleitung benennt Rechteinhaber und juristische Fachperson vor Nutzungsrechtseinräumung oder Softwareveröffentlichung.
- Projektleitung benennt unabhängige Wahlprüfer; ohne geprüften Entwurf keine Implementierung des Wahlprotokolls.
- Rechtliche Datenschutzfristen und regionale Personalregelwerke bleiben vor betroffener Umsetzung fachlich zu bestätigen.
- CODEOWNERS ist jetzt zugeordnet. Ein unabhängiger Prüfer und wirksame Branchschutzregeln sind dadurch nicht nachgewiesen.
- Die frühere Branchschutzabfrage lieferte HTTP 403. Aus der leeren Regelsatzliste wurde kein fehlender Schutz abgeleitet.
- Juristische Lizenzwahl und geprüfte Versions-/Installationsmatrix bleiben vor tatsächlicher Verwendung erforderlich.

Die Zurückstellung der drei angefragten Rollen ist ausdrücklich gewünscht.
Die gesperrten Aktionen und spätesten Termine sind im Register dokumentiert.
Sie ist keine Fachfreigabe und keine P0-Gesamtabnahme.

## Nächste drei Aktionen

1. KC-037-PR mit aktuellem CI- und Review-Stand prüfen. Issue #37 erst nach Review und Integration schließen.
2. [KC-038 / #38](https://github.com/itsh-neumeier/kitaconnect/issues/38) weiterführen:
   unabhängige Review-Rollen und nachweisbaren Branchschutz klären. Bestätigten CODEOWNER nicht erneut abfragen.
3. KC-036 für die sprachliche Bestandsprüfung bearbeiten. Die dokumentierten späteren Fachprüfungen vor ihren Sperrterminen veranlassen.

Die Integrationsreihenfolge ist #39, #40, #41, danach der KC-037-PR.
Nach Integration einer Basis den nächsten PR auf `main` umstellen und erneut prüfen.
Die Sitzung erteilt keine eigenständige Merge-Freigabe.

## Sicherheitsumfang und Fortsetzung

Nur Planung und Repository-Zuständigkeit. Keine Echtdaten oder produktiven Secrets.
Die bestehende Erlaubnis für npm audit lokal und in GitHub Actions bleibt gültig.
Lesen Sie beim Neustart AGENTS, CLAUDE, diesen Status, die Entscheidungsliste und den aktuellen GitHub-Stand.
Wiederholen Sie bestätigte Fragen nicht. Arbeiten Sie weiter in P0; P1 benötigt Abnahme und eigenen Auftrag.
