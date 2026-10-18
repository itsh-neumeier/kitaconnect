# Self-Hosting- und Betriebshandbuch

**KitaConnect Spezifikation v5.9 · 17.10.2026 · Konsolidierter Planungsstand**

> Status: Anforderungsspezifikation, keine implementierte oder rechtsverbindlich geprüfte Software. Die neuesten ausdrücklichen Interviewentscheidungen gelten.

## Unterstützte Installationen
(A) Docker Compose/Portainer für Einzel-/kleinere Installationen, (B) Kubernetes/Helm für skalierbare und hochverfügbare Installationen. GHCR versionierte Images; Rust Services, SvelteKit-Web, PostgreSQL, Redis, Worker, lokaler oder S3-kompatibler Speicher. Reverse Proxy Traefik/Nginx, HTTPS. Nur stabile Releases; Admin bestätigt Installation, Release-Publish nach CI automatisch.

## Installationsablauf (Soll-Prozess)
1. DNS, TLS, Serverressourcen, Mailversand, Speicher und Backupziel vorbereiten.
2. `.env` aus Beispiel erzeugen; Secrets über Docker/Kubernetes Secret Stores bereitstellen, nicht ins Git.
3. PostgreSQL/Redis/Storage starten, DB-Verbindung und Schema-Rechte prüfen.
4. Versionierte Images beziehen, Migrationen im Wartungsmodus ausführen.
5. Ersteinrichtungsassistent: Systemadmin, Träger/Einrichtung, Module, Domains, Datenschutz.
6. Healthchecks, SMTP, Push, Medienworker, Backup und Restore-Probe prüfen.
7. Monitoring, Alerts und Log-Retention aktivieren; Dokumentation der Verantwortlichen.

## Backup und Recovery
Konfigurierbare verschlüsselte Backups von Datenbank, Medien, Konfiguration und Schlüsselmaterial; WAL-Archiv/PITR; externe Sicherung; regelmäßige Test-Restores. RPO und RTO jeweils Ziel 24 Stunden, betriebsabhängig zu testen. Mandantenspezifischer Restore über isolierte Restore-Umgebung und geprüfte selektive Datenübernahme. Nach Restore Löschjournal und Consent-Widerrufe erneut durchsetzen.

## Updates
Stable-Version anzeigen, Kompatibilität/Backup prüfen, manueller Installationsfreigabeschritt, Wartungsmodus, Migrationscheck, Rolling/Blue-Green soweit kompatibel, Healthchecks und Wiederanlauf. Schema-Migrationen können irreversibel sein; Rollback ist Restore-/Forward-Fix-Strategie, kein blindes Zurücksetzen.

## Monitoring
Integrierte Admin-Diagnose plus Prometheus/Grafana, strukturierte Logs, Traces, Alerting, Queue-/Workerzustand, Speicher-/Mandantenquotas, Datenbank-/WAL-/Backupzustand, Login-/Audit-Anomalien. Keine Geheimnisse, Kinder-/Gesundheitsdaten oder Wahlkorrelationen in Logs.

## Betriebsmatrix
| Profil | Laufzeit | Speicher | HA |
|---|---|---|---|
| Kleine Einrichtung | Docker Compose/Portainer | Lokal oder S3 | optional |
| Mehrere Träger | Compose oder Kubernetes | bevorzugt S3 | empfohlen |
| Größere Installation | Kubernetes/Helm | S3, HA-Postgres | empfohlen |

Konkrete CPU/RAM-/Storage-Mindestwerte erst durch P0-Benchmarks belastbar festlegen, insbesondere für lokale Gesichtserkennung.


## Dokumentationssprache (Ergänzung v5.9)
Für dieses Dokument gilt die deutsche Projektrichtlinie `13_Dokumentationsrichtlinie_DE_ASD_STE100.md` mit dem verbindlichen Fachwortverzeichnis `14_Fachwortverzeichnis_DE.md`. Die Richtlinie orientiert sich an ASD-STE100, stellt aber **keine offizielle deutsche ASD-STE100-Konformität** dar.
