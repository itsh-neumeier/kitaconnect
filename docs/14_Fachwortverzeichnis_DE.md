# Fachwortverzeichnis Deutsch – KitaConnect

**Version v5.9 · verbindliche Terminologie für deutschsprachige Dokumentation**

| Bevorzugter Begriff | Bedeutung / Verwendung | Vermeiden, wenn gleichbedeutend |
|---|---|---|
| Einrichtung | Einzelne Kindertageseinrichtung | Kita-Standort, Mandant (wenn fachlich Einrichtung gemeint ist) |
| Träger | Organisation, die Einrichtungen verantwortet | Betreiber (bei fachlicher Trägerrolle) |
| Installation | Technisch eigenständig betriebene KitaConnect-Umgebung | Instanz, wenn unklar |
| Mandant | Technisch und rechtlich abgegrenzter Datenbereich | Tenant in deutschem Fließtext |
| PostgreSQL-Schema | Getrennter Namensraum in einer Datenbank | Datenbank, wenn nur Schema gemeint ist |
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
| Zugriffsprotokoll | Nachvollziehbarer Nachweis von Datenzugriffen | Log ohne Erklärung |
| Mandantenwechsel | Wechsel des aktiven organisatorischen Kontexts | Login-Wechsel, wenn kein neuer Login erfolgt |

## Regeln
1. Ergänze neue Fachbegriffe vor ihrer ersten wiederholten Verwendung.
2. Verwende technische Eigennamen wie `Rust`, `SvelteKit`, `PostgreSQL`, `Docker Compose`, `OpenAPI` und `ASD-STE100` unverändert.
3. Definiere Akronyme beim ersten Auftreten je Dokument, wenn die Zielgruppe sie nicht voraussetzen kann.
4. Bei Konflikten zwischen Glossar und normativer Fachdefinition hat die rechtlich oder technisch korrekte Definition Vorrang; aktualisiere danach das Glossar.
