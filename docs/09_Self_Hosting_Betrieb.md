# Self-Hosting- und Betriebshandbuch

**KitaConnect Spezifikation v5.9 · 17.10.2026 · Konsolidierter Planungsstand**

> Status: Anforderungsspezifikation, keine implementierte oder rechtsverbindlich geprüfte Software. Die neuesten ausdrücklichen Interviewentscheidungen gelten.

## Unterstützte Installationen
Für einzelne und kleinere Installationen ist Docker Compose mit Portainer vorgesehen.
Für skalierbare und hochverfügbare Installationen sind Kubernetes und Helm vorgesehen.
Versionierte Images liegen in der GitHub Container Registry (GHCR).
Die Installation umfasst Rust-Dienste, SvelteKit-Web, PostgreSQL, Redis und Hintergrunddienste.
Dateien liegen lokal oder in S3-kompatiblem Speicher.
Traefik oder Nginx dienen als vorgeschalteter Webserver. Der Zugang verwendet HTTPS.
Es werden nur stabile Versionen installiert.
Die Administration bestätigt die Installation. Die Veröffentlichung erfolgt nach erfolgreicher automatisierter Integration (CI) automatisch.

## Installationsablauf (Soll-Prozess)
DNS bezeichnet die Namensauflösung. TLS schützt die Transportverbindung.
SMTP bezeichnet das Verfahren für den Mailversand.
Der Ablauf beschreibt den späteren Sollzustand. Das P0-Grundgerüst ist noch nicht installierbar.

1. Bereiten Sie DNS, TLS, Serverressourcen, Mailversand, Speicher und das Ziel der Datensicherung vor.
2. Erzeugen Sie `.env` aus dem Beispiel. Stellen Sie Secrets über geschützte Docker- oder Kubernetes-Speicher bereit.
   Speichern Sie Secrets nicht in Git.
3. Starten Sie PostgreSQL, Redis und Dateispeicher. Prüfen Sie Datenbankverbindung und Schemarechte.
4. Beziehen Sie versionierte Images. Führen Sie Migrationen im Wartungsmodus aus.
5. Richten Sie Systemadministrator, Träger beziehungsweise Einrichtung, Module, Domains und Datenschutz über den Assistenten ein.
6. Prüfen Sie Zustandsprüfungen, SMTP, Push, Medien-Hintergrunddienst, Datensicherung und Wiederherstellungsprobe.
7. Aktivieren Sie Betriebsüberwachung, Warnmeldungen und Aufbewahrungsregeln für Protokolle.
   Dokumentieren Sie die Verantwortlichen.

## Datensicherung und Wiederherstellung

Datensicherungen von Datenbank, Medien, Konfiguration und Schlüsselmaterial sind konfigurierbar und verschlüsselt.
Das PostgreSQL-Schreibprotokoll (WAL) wird archiviert und ermöglicht die Wiederherstellung zu einem Zeitpunkt (PITR).
Externe Sicherung und regelmäßige Wiederherstellungstests sind vorgesehen.
Der angestrebte maximale Datenverlustzeitraum (RPO) beträgt 24 Stunden.
Die angestrebte Wiederherstellungsdauer (RTO) beträgt ebenfalls 24 Stunden.
Beide Ziele werden betriebsabhängig getestet.
Eine mandantenspezifische Wiederherstellung verwendet eine isolierte Umgebung und geprüfte selektive Datenübernahme.
Nach Wiederherstellung werden Löschjournal und widerrufene Einwilligungen erneut durchgesetzt.

## Updates
1. Zeigen Sie die verfügbare stabile Version an. Prüfen Sie Kompatibilität und Datensicherung.
2. Holen Sie die manuelle Installationsfreigabe ein. Aktivieren Sie den Wartungsmodus und prüfen Sie die Migration.
3. Verwenden Sie schrittweise Aktualisierung oder den Wechsel zwischen zwei Betriebsumgebungen, soweit kompatibel.
4. Prüfen Sie Zustand und Wiederanlauf.

Schema-Migrationen können unumkehrbar sein.
Der Fehlerplan verwendet Wiederherstellung oder eine nachträgliche Korrektur. Blindes Zurücksetzen ist ausgeschlossen.

## Betriebsüberwachung

Die integrierte Administrationsdiagnose wird durch Prometheus und Grafana ergänzt.
Strukturierte Protokolle, Ablaufspuren und Warnmeldungen unterstützen die Diagnose.
Überwacht werden Warteschlangen, Hintergrunddienste, Speichergrenzen und Mandantenkontingente.
Überwacht werden außerdem Datenbank, WAL, Datensicherungen sowie Auffälligkeiten bei Anmeldung und Datenzugriffen.
Protokolle enthalten keine Geheimnisse, Kinder- oder Gesundheitsdaten und keine Verknüpfung zwischen Wählern und Stimmen.

## Betriebsmatrix

Das [KC-037-Register](17_P0_Entscheidungsregister.md) enthält die am 18.10.2026 recherchierten Versionsstände.
Diese Planungsbasis ist noch keine getestete Installationsfreigabe.
KC-032 MUSS konkrete Versionen, Images, Lizenzen und Kompatibilität vor Verwendung nachweisen.

HA bezeichnet Hochverfügbarkeit.

| Profil | Laufzeit | Speicher | HA |
|---|---|---|---|
| Kleine Einrichtung | Docker Compose/Portainer | Lokal oder S3 | optional |
| Mehrere Träger | Compose oder Kubernetes | bevorzugt S3 | empfohlen |
| Größere Installation | Kubernetes/Helm | S3, HA-Postgres | empfohlen |

Die Projektleitung hat ein [Referenzprofil](17_P0_Entscheidungsregister.md) mit 4 vCPU,
8 GB RAM und 100 GB SSD für Lastmessungen bestätigt.
Es ist noch kein Nachweis allgemeiner Mindestressourcen.
Die tatsächlichen Messungen folgen nach Implementierung repräsentativer Abläufe in KC-034.
Lokale Medienverarbeitung benötigt eine gesonderte Messung.


## Dokumentationssprache (Ergänzung v5.9)
Für dieses Dokument gilt die deutsche Projektrichtlinie `13_Dokumentationsrichtlinie_DE_ASD_STE100.md` mit dem verbindlichen Fachwortverzeichnis `14_Fachwortverzeichnis_DE.md`. Die Richtlinie orientiert sich an ASD-STE100, stellt aber **keine offizielle deutsche ASD-STE100-Konformität** dar.
