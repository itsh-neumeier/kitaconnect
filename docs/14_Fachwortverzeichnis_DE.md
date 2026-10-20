# Fachwortverzeichnis Deutsch – KitaConnect

**Version v5.9 · verbindliche Terminologie für deutschsprachige Dokumentation**

| Bevorzugter Begriff | Bedeutung / Verwendung | Vermeiden, wenn gleichbedeutend |
|---|---|---|
| Einrichtung | Einzelne Kindertageseinrichtung | Kita-Standort, Mandant (wenn fachlich Einrichtung gemeint ist) |
| Träger | Organisation, die Einrichtungen verantwortet | Betreiber (bei fachlicher Trägerrolle) |
| Installation | Technisch eigenständig betriebene KitaConnect-Umgebung | Instanz, wenn unklar |
| Mandant | Technisch und rechtlich abgegrenzter Datenbereich | Tenant in deutschem Fließtext |
| PostgreSQL-Schema | Getrennter Namensraum in einer Datenbank | Datenbank, wenn nur Schema gemeint ist |
| Schema-Registry | Geschütztes Verzeichnis der Zuordnung zwischen Einrichtung und geprüftem PostgreSQL-Schema; ein Eintrag erteilt kein Zugriffsrecht | Schemaname als Berechtigungsnachweis |
| Provisionierung | Geprüftes Anlegen technischer Ressourcen durch eine gesonderte Betriebsidentität | Ungeprüfte automatische Anlage beim Anwendungsstart |
| Berechtigung | Erlaubnis für eine definierte Aktion auf Daten | Freigabe, wenn technische Autorisierung gemeint ist |
| Rolle | Bündel definierter Berechtigungen | Profil, wenn Rolle gemeint ist |
| Sorgeberechtigte Person | Person mit den jeweils geltenden Sorge- und Zugriffsrechten | Elternteil als pauschale Rechtsannahme |
| Elternbeirat | Gremium der Elternvertretung | Kita-Verwaltung |
| Kinderakte | Vertrauliche Informationen zu einem Kind | Portfolio (getrennte Datenkategorie) |
| Portfolio | Für berechtigte Personen freigegebene Entwicklungs- und Lerndokumentation | Kinderakte |
| Einwilligung | Nachweisbare Zustimmung für einen bestimmten Zweck | pauschale Freigabe |
| Freigabe | Dokumentierte Genehmigung einer Handlung oder Veröffentlichung | Berechtigung, wenn dauerhaftes Recht gemeint ist |
| Hintergrunddienst | Dienst für zeitversetzte Aufgaben | Worker, außer in Code/API |
| Datensicherung | Gesicherte Kopie zur Wiederherstellung | Backup im deutschen Fließtext |
| Wiederherstellung | Rückführung gesicherter Daten in einen geprüften Zustand | Restore im deutschen Fließtext |
| Stabile Version | Freigegebene, vollständig geprüfte Veröffentlichung | Release-Kandidat |
| Prüfschritt | Nachweisbare Qualitäts- oder Sicherheitskontrolle | Gate ohne Erklärung |
| Software-Stückliste | Verzeichnis der Softwarebestandteile; englisch Software Bill of Materials (SBOM) | SBOM ohne Erklärung |
| Dienstidentität | Technisch überprüfbare Identität eines Dienstes | Service-Identität im deutschen Fließtext |
| Diensttoken | Zeitlich begrenzter Nachweis für einen geprüften Dienstaufruf | Service-Token im deutschen Fließtext |
| Verbrauchergruppe | Gruppe von Hintergrunddiensten, die Aufträge aus einem Redis Stream übernehmen | Consumer Group ohne Erklärung |
| Outbox | Dauerhafter Bestand von Ereignissen, die nach einer Fachänderung zugestellt werden müssen | Queue, wenn der dauerhafte Ausgangsbestand gemeint ist |
| Dienstvertrag | Versionierter Vertrag für eine Dienstschnittstelle oder ein Ereignis | Contract ohne Erklärung |
| Zugriffsprotokoll | Nachvollziehbarer Nachweis von Datenzugriffen | Log ohne Erklärung |
| Mandantenwechsel | Wechsel des aktiven organisatorischen Kontexts | Login-Wechsel, wenn kein neuer Login erfolgt |
| Arbeitsablauf | Fachliche Folge von Bearbeitungs- und Freigabeschritten | Workflow im deutschen Fließtext; technische Bezeichner bleiben erhalten |
| Betriebsüberwachung | Beobachtung von Zustand, Fehlern und Ressourcen einer Installation | Monitoring im deutschen Fließtext |
| Nachrichtenübersicht | Zusammenstellung freigegebener Nachrichten für eine Zielgruppe | Newsfeed, Feed im deutschen Fließtext |
| Notfallzugriff | Begründeter und zeitbegrenzter Zugriff im ausdrücklich definierten Notfall | Break-glass im deutschen Fließtext |
| Berechtigungsumfang | Explizit begrenzter Geltungsbereich eines Zugriffsrechts | Scope im deutschen Fließtext; API-Felder bleiben erhalten |
| Dienstvertragstest | Prüfung der vereinbarten Schnittstelle zwischen Diensten | Contract-Test im deutschen Fließtext |
| Idempotenz | Eigenschaft, bei wiederholter Verarbeitung keine doppelte fachliche Wirkung zu erzeugen | Genau-einmal-Zusage ohne Nachweis |
| Fehlerablage | Nachvollziehbarer Bestand nicht erfolgreich verarbeiteter Aufträge | Dead-Letter ohne Erklärung |
| Versionsmarke | Markierung eines Versionsstands in Git | Tag ohne Erklärung im deutschen Fließtext |
| Änderungsprotokoll | Übersicht der Änderungen einer Softwareversion | Changelog im deutschen Fließtext |
| Zugriffskontrolle | Prüfung, ob eine konkrete Aktion auf Daten zulässig ist | Audit als Synonym für Berechtigungsentscheidung |
| Audit | Nachvollziehbare Prüfung protokollierter Vorgänge; technische Bezeichner bleiben erhalten | Pauschaler Vollzugriff auf Fachinhalte |

## Technische Abkürzungen

| Kürzel | Bedeutung im Projekt |
|---|---|
| ADR | Dokumentierte Architekturentscheidung; Architecture Decision Record |
| API | Programmierschnittstelle; Application Programming Interface |
| CI | Automatisierte Integration; Continuous Integration |
| PWA | Progressive Web App; installierbare Webanwendung |
| RBAC / ABAC | Rollenbasierte beziehungsweise attributbasierte Zugriffskontrolle |
| RLS | Row-Level Security; Zugriffsschutz für Datenbankzeilen |
| RPO / RTO | Angestrebter maximaler Datenverlustzeitraum beziehungsweise angestrebte Wiederherstellungsdauer |
| PITR / WAL | Wiederherstellung zu einem Zeitpunkt beziehungsweise PostgreSQL-Schreibprotokoll |
| HA | Hochverfügbarkeit; High Availability |
| SAML / OIDC | Security Assertion Markup Language beziehungsweise OpenID Connect |
| MFA / TOTP | Mehrfaktor-Authentisierung beziehungsweise zeitbasierte Einmalpasswörter |
| TLS / mTLS | Geschützte Transportverbindung beziehungsweise TLS mit beidseitiger Zertifikatsprüfung |
| EXIF / GPS | Bildmetadaten beziehungsweise Positionsangaben des Global Positioning System |
| SMTP / SPF / DKIM / DMARC | Mailversand beziehungsweise Verfahren für Absender- und Domainprüfung |
| ICS / HMAC / VAPID | Kalenderformat, schlüsselgebundener Nachrichtencode beziehungsweise Identifikation des Web-Push-Anwendungsservers |
| CSRF / CSP / IDOR | Gefälschte browserbasierte Anfrage, Browser-Inhaltsrichtlinie beziehungsweise unberechtigter direkter Objektzugriff |
| WCAG / E2E / SBOM | Web Content Accessibility Guidelines, Ende-zu-Ende-Tests beziehungsweise Software-Stückliste |
| SAST / SCA | Statische Quelltextanalyse beziehungsweise Analyse von Softwareabhängigkeiten |
| GHCR / HR / UI | GitHub Container Registry, Personalverwaltung beziehungsweise Benutzeroberfläche |
| DSGVO / DSFA | Datenschutz-Grundverordnung beziehungsweise Datenschutz-Folgenabschätzung |

## Regeln
1. Ergänze neue Fachbegriffe vor ihrer ersten wiederholten Verwendung.
2. Verwende technische Eigennamen wie `Rust`, `SvelteKit`, `PostgreSQL`, `Docker Compose`, `OpenAPI` und `ASD-STE100` unverändert.
3. Definiere Akronyme beim ersten Auftreten je Dokument, wenn die Zielgruppe sie nicht voraussetzen kann.
4. Bei Konflikten zwischen Glossar und normativer Fachdefinition hat die rechtlich oder technisch korrekte Definition Vorrang; aktualisiere danach das Glossar.
