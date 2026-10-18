# Test- und Release-Konzept

**KitaConnect Spezifikation v5.9 · 17.10.2026 · Konsolidierter Planungsstand**

> Status: Anforderungsspezifikation, keine implementierte oder rechtsverbindlich geprüfte Software. Die neuesten ausdrücklichen Interviewentscheidungen gelten.

## Testpyramide und verpflichtende Gates
Format/Lint, Rust clippy/test, TypeScript/Svelte checks, Unit-/Integrationstests, Datenbankmigrationstests pro Mandantenschema, Contract-Tests, E2E, RBAC/ABAC/Feldrechte, Sorgerechtsfälle, Cross-Tenant- und Cross-Carrier-Tests, Security-SAST/SCA/Secrets/Container, SBOM, Lizenz-Compliance, Performance/Load, WCAG 2.2 AA, PWA/Mobil, Backup/Restore/PITR und Rollback-Szenarien. Keine echten Kinderdaten in CI; synthetische Fixtures.

## Security-Kritikalität
Blockierende Tests: tenant isolation, Wahlgeheimnis, Gesundheits-/Personalaktenzugriff, Consent-Withdrawal, Notfallzugriff, Finanztransaktionen, Passwort-/Session-Sicherheit, unsignierte/verwundbare Images. Kritische Änderungen brauchen genehmigte PR-Reviews, bevor automatischer Release möglich ist.

## Automatische Stable-Pipeline
1. Geschützter Main-Branch, mindestens ein Review, CODEOWNERS für Security/Privacy.
2. CI-Checks, Integration, E2E, Contract, Mandantentests, Last-Schwellen, Security, Accessibility.
3. Migrationskompatibilität und Rücksicherungsprobe; Release-Checkliste maschinell geprüft.
4. Versionsberechnung/Tag, Changelog, SBOM, Image-Signierung, GHCR-Publish, Release Notes.
5. Bei einem fehlgeschlagenen Gate kein Tag und kein Publish. Keine öffentlichen Preview-/Nightly-Artefakte.

## Qualitätskennzahlen (initiale Zielwerte, vor Release festzulegen)
Keine kritischen bekannten Sicherheitslücken; keine unaufgelösten kritischen Bugs; alle verbindlichen Testfälle erfolgreich. Konkrete Performance-SLOs und Lastszenarien sind in P0 als messbare Werte festzulegen, statt unbelegte Zahlen vorzugeben.


## Dokumentationssprache (Ergänzung v5.9)
Für dieses Dokument gilt die deutsche Projektrichtlinie `13_Dokumentationsrichtlinie_DE_ASD_STE100.md` mit dem verbindlichen Fachwortverzeichnis `14_Fachwortverzeichnis_DE.md`. Die Richtlinie orientiert sich an ASD-STE100, stellt aber **keine offizielle deutsche ASD-STE100-Konformität** dar.
